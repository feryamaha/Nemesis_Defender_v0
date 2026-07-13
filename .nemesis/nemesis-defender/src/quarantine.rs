//! Quarentena — em vez de DELETAR (`rm`) arquivos maliciosos confirmados, o nemesis-defender
//! os MOVE para `.nemesis/quarantine/` e os retém para revisão humana.
//!
//! Cada item vira uma pasta `.nemesis/quarantine/<id>/` contendo:
//!   - o arquivo original (preservado, inerte — fora do projeto, não executa)
//!   - `meta.json` (por que foi retido: path original, severidade, violations, hora)
//! Um índice `.nemesis/quarantine/PENDING.json` lista os itens não-resolvidos. Enquanto
//! houver itens pendentes, o pretool bloqueia a sessão (exit 2) pedindo revisão humana.
//!
//! O humano decide: `restore` (falso-positivo, volta ao lugar) ou `purge` (expurga de vez).
//! Tudo 100% local — nada é exfiltrado.

use crate::DefenderResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const PENDING_FILE: &str = "PENDING.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QViolation {
    pub visitor: String,
    pub line: u32,
    pub message: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineEntry {
    pub id: String,
    pub original_path: String,
    pub quarantined_at: String,
    pub severity: String,
    pub violations: Vec<QViolation>,
}

/// Resolve `.nemesis/quarantine` subindo do executável até `.nemesis/` (CWD-independente).
pub fn quarantine_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors() {
            if anc.file_name().map(|n| n == ".nemesis").unwrap_or(false) {
                return anc.join("quarantine");
            }
        }
    }
    PathBuf::from(".nemesis/quarantine")
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn held_file_path(entry: &QuarantineEntry) -> PathBuf {
    let basename = Path::new(&entry.original_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    quarantine_dir().join(&entry.id).join(basename)
}

/// Move o arquivo malicioso para a quarentena. Retorna o `id` criado.
pub fn quarantine_file(original: &Path, result: &DefenderResult) -> std::io::Result<String> {
    let qdir = quarantine_dir();
    let basename = original.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let orig_str = original.display().to_string();

    // DEDUP por path: se já existe um item PENDENTE para este mesmo caminho, NÃO cria uma
    // nova entrada. Cobre 3 casos: (a) rajada de eventos do inotify para uma única escrita;
    // (b) múltiplas instâncias transitórias do daemon; (c) o loop editor-vs-daemon — um
    // editor com o arquivo aberto re-salva o buffer assim que o daemon move o arquivo,
    // recriando-o. Move o duplicado para a pasta do item já existente (mantém src/ limpo,
    // sem poluir PENDING) e retorna o id existente.
    {
        let pending = load_pending();
        if let Some(existing) = pending.iter().find(|e| e.original_path == orig_str) {
            let dest = qdir.join(&existing.id).join(basename);
            if fs::rename(original, &dest).is_err() {
                let _ = fs::copy(original, &dest).and_then(|_| fs::remove_file(original));
            }
            return Ok(existing.id.clone());
        }
    }

    let id = format!(
        "{}__{}",
        chrono::Local::now().format("%Y%m%d-%H%M%S"),
        sanitize(basename)
    );
    let item_dir = qdir.join(&id);
    fs::create_dir_all(&item_dir)?;

    // Mover (rename; fallback copy+remove entre filesystems diferentes).
    let dest_file = item_dir.join(basename);
    if fs::rename(original, &dest_file).is_err() {
        fs::copy(original, &dest_file)?;
        let _ = fs::remove_file(original);
    }

    let entry = QuarantineEntry {
        id: id.clone(),
        original_path: original.display().to_string(),
        quarantined_at: chrono::Local::now().to_rfc3339(),
        severity: format!("{:?}", result.severity),
        violations: result
            .violations
            .iter()
            .map(|v| QViolation {
                visitor: v.visitor.clone(),
                line: v.line,
                message: v.message.clone(),
                evidence: v.evidence.clone(),
            })
            .collect(),
    };

    if let Ok(j) = serde_json::to_string_pretty(&entry) {
        let _ = fs::write(item_dir.join("meta.json"), j);
    }

    let mut pending = load_pending();
    pending.push(entry);
    save_pending(&pending);

    Ok(id)
}

pub fn load_pending() -> Vec<QuarantineEntry> {
    let p = quarantine_dir().join(PENDING_FILE);
    fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_pending(entries: &[QuarantineEntry]) {
    let qdir = quarantine_dir();
    let _ = fs::create_dir_all(&qdir);
    if let Ok(j) = serde_json::to_string_pretty(entries) {
        let _ = fs::write(qdir.join(PENDING_FILE), j);
    }
}

/// Há itens não-resolvidos? (o pretool usa isto para bloquear a sessão.)
pub fn has_pending() -> bool {
    !load_pending().is_empty()
}

// ── RESTORED-ALLOWLIST (decisão humana de restore, lida pelo daemon) ──────────────────────────
//
// Quando o humano faz `restore <id>` (falso-positivo), a decisão é persistida aqui como
// (original_path, sha256). O daemon consulta antes de quarentenar: se o par casar, NÃO
// re-quarentena (respeita o restore). Só a camada daemon é relaxada; pretool e eBPF seguem
// bloqueando. Arquivo em .nemesis/quarantine/ (absolute_block; o daemon pula essa subárvore).
// Match por path E hash (conservador): conteúdo alterado no mesmo path volta a ser escaneado.

const RESTORED_ALLOWLIST_FILE: &str = "restored-allowlist.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoredEntry {
    pub original_path: String,
    pub sha256: String,
    pub restored_at: String,
}

fn restored_allowlist_path() -> PathBuf {
    quarantine_dir().join(RESTORED_ALLOWLIST_FILE)
}

/// SHA-256 hex de um buffer (mesmo esquema do checksum do install).
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Lê a restored-allowlist. Fail-safe: qualquer erro (ausente/vazia/inválida) => vazia.
pub fn load_restored_allowlist() -> Vec<RestoredEntry> {
    fs::read_to_string(restored_allowlist_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_restored_allowlist(entries: &[RestoredEntry]) -> std::io::Result<()> {
    fs::create_dir_all(quarantine_dir())?;
    let j = serde_json::to_string_pretty(entries)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    fs::write(restored_allowlist_path(), j)
}

/// Registra a decisão humana de restore (append idempotente por (original_path, sha256)).
/// Retorna Err se não conseguiu persistir; o chamador (cli_restore) apenas reporta, o restore
/// em si não é desfeito.
pub fn record_restored(original_path: &str, sha256: &str) -> std::io::Result<()> {
    let mut entries = load_restored_allowlist();
    if entries
        .iter()
        .any(|e| e.original_path == original_path && e.sha256 == sha256)
    {
        return Ok(()); // já registrado
    }
    entries.push(RestoredEntry {
        original_path: original_path.to_string(),
        sha256: sha256.to_string(),
        restored_at: chrono::Local::now().to_rfc3339(),
    });
    save_restored_allowlist(&entries)
}

/// Lógica pura de match (testável sem disco): o par (path, conteúdo) está na lista?
pub fn entry_matches(entries: &[RestoredEntry], original_path: &Path, content: &[u8]) -> bool {
    let path_str = original_path.display().to_string();
    let hash = sha256_hex(content);
    entries
        .iter()
        .any(|e| e.original_path == path_str && e.sha256 == hash)
}

/// true se este (path, conteúdo) foi restaurado como falso-positivo por decisão humana.
/// Fail-safe: lista vazia => false (nenhuma isenção). Usada pelo daemon.
pub fn is_restored_fp(original_path: &Path, content: &[u8]) -> bool {
    entry_matches(&load_restored_allowlist(), original_path, content)
}

// ── CLI ──────────────────────────────────────────────────────────────────────

pub fn cli_list() {
    let pending = load_pending();
    if pending.is_empty() {
        println!("Quarentena vazia.");
        return;
    }
    println!(
        "{} item(ns) em quarentena ({}/):",
        pending.len(),
        quarantine_dir().display()
    );
    for e in &pending {
        println!(
            "  [{}]  {}  — {} — {} violacao(oes)",
            e.id,
            e.original_path,
            e.severity,
            e.violations.len()
        );
    }
    println!("\nUse o ID da coluna [ID] acima (sem os colchetes):");
    println!("  nemesis-defender --quarantine show <ID>   (inspecionar)");
    println!("  nemesis-defender --quarantine restore <ID> (falso-positivo, volta ao lugar)");
    println!("  nemesis-defender --quarantine purge <ID>   (expurgar definitivamente)");
}

pub fn cli_show(id: &str) {
    match load_pending().into_iter().find(|e| e.id == id) {
        Some(e) => {
            println!("ID          : {}", e.id);
            println!("Origem      : {}", e.original_path);
            println!("Quarentenado: {}", e.quarantined_at);
            println!("Severidade  : {}", e.severity);
            println!("Arquivo retido (inerte): {}", held_file_path(&e).display());
            println!("Violacoes:");
            for v in &e.violations {
                println!("  ├─ [{}] L{} — {}", v.visitor, v.line, v.message);
                println!("  │   evidencia: {}", v.evidence);
            }
        }
        None => println!("ID nao encontrado: {}", id),
    }
}

/// Restaura o arquivo para o path original (falso-positivo) e remove da quarentena.
pub fn cli_restore(id: &str) {
    let mut pending = load_pending();
    let Some(pos) = pending.iter().position(|e| e.id == id) else {
        println!("ID nao encontrado: {}", id);
        return;
    };
    let entry = pending[pos].clone();
    let held = held_file_path(&entry);
    let orig = PathBuf::from(&entry.original_path);

    // Persistir a decisão humana de FP ANTES de mover o arquivo de volta: quando o daemon
    // receber o evento inotify de criação em `orig`, a restored-allowlist já terá
    // (original_path, sha256) e is_restored_fp isenta a re-quarentena (fecha a corrida
    // restore-vs-daemon). Falha aqui NÃO impede o restore, apenas alerta.
    match fs::read(&held) {
        Ok(bytes) => {
            let hash = sha256_hex(&bytes);
            if let Err(e) = record_restored(&entry.original_path, &hash) {
                eprintln!(
                    "  ⚠️  Falha ao registrar a decisão na restored-allowlist ({}). O daemon pode re-quarentenar.",
                    e
                );
            }
        }
        Err(e) => {
            eprintln!(
                "  ⚠️  Não consegui ler o arquivo retido para registrar o hash ({}). O daemon pode re-quarentenar.",
                e
            );
        }
    }

    if let Some(parent) = orig.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let moved = fs::rename(&held, &orig)
        .or_else(|_| fs::copy(&held, &orig).map(|_| ()))
        .is_ok();
    if moved {
        let _ = fs::remove_dir_all(quarantine_dir().join(&entry.id));
        pending.remove(pos);
        save_pending(&pending);
        println!("Restaurado: {} -> {}", entry.id, entry.original_path);
    } else {
        println!("Falha ao restaurar {} (arquivo retido em {})", entry.id, held.display());
    }
}

/// Expurga (deleta definitivamente) o item em quarentena.
pub fn cli_purge(id: &str) {
    let mut pending = load_pending();
    let Some(pos) = pending.iter().position(|e| e.id == id) else {
        println!("ID nao encontrado: {}", id);
        return;
    };
    let _ = fs::remove_dir_all(quarantine_dir().join(id));
    pending.remove(pos);
    save_pending(&pending);
    println!("Expurgado definitivamente: {}", id);
}
