//! Servidor HTTP local para modo --serve.
//!
//! SPEC-001 (ISSUE-001): o doctor FULL nao roda mais no request path.
//! - Cache por RECURSO (nao por URL), com invalidacao por mtime:
//!     violations  <- mtime do ledger
//!     doctor      <- mtime do snapshot .nemesis/telemetry/doctor-full.json
//!     pentest     <- mtime do pentest-results.md
//! - O doctor FULL (11-38s wall, ~51s CPU medidos) roda numa thread de background:
//!   no boot (se snapshot ausente) e a cada NEMESIS_DOCTOR_FULL_INTERVAL (default 30min).
//!   A observabilidade serve SEMPRE o full (ressalva de Fernando) — via snapshot.
//! - Single-thread mantido por decisao explicita (SPEC-001 R1.3): com o full fora do
//!   path, o pior request e o re-parse do ledger (~0,15s) em mudanca de mtime.

use crate::config;
use crate::identity;
use crate::sources;
use crate::sources::doctor::DoctorRun;
use crate::sources::pentest::PentestRun;
use crate::sources::violations::{LedgerState, Violation};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tiny_http::{Header, Method, Response, Server};

#[derive(Default)]
struct ResourceCache {
    /// (mtime do ledger, estado incremental, lista ordenada ts-desc pronta para servir)
    violations: Option<(SystemTime, LedgerState, Arc<Vec<Violation>>)>,
    doctor: Option<(SystemTime, Arc<DoctorRun>)>,
    pentest: Option<(SystemTime, Arc<PentestRun>)>,
}

fn mtime(path: &std::path::Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

pub fn run(port: u16, nemesis_path: &std::path::Path) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("bind {}: {}", addr, e))?;

    println!("[nemesis-publisher] Servindo em http://{}", addr);

    // Registro best-effort no startup: se o install gerou o token mas nao conseguiu registrar
    // (rede/dashboard fora do ar), tenta de novo aqui. O systemd/launchd roda --serve no boot,
    // entao o install acaba contado assim que houver rede. Em thread para nao bloquear o serve.
    std::thread::spawn(try_register_if_needed);

    // Doctor FULL em background: boot (se snapshot ausente) + intervalo configuravel.
    let np = nemesis_path.to_path_buf();
    std::thread::spawn(move || background_doctor_full(np.clone()));

    // Sync Neon em background: se DATABASE_URL estiver definida, sincroniza a cada
    // NEMESIS_SYNC_INTERVAL (default 30min). Caso contrario, nao faz nada.
    let np2 = nemesis_path.to_path_buf();
    std::thread::spawn(move || background_neon_sync(np2));

    let cache: Mutex<ResourceCache> = Mutex::new(ResourceCache::default());

    // Warm-up: paga o parse inicial do ledger (~1s) no boot, antes de servir —
    // nenhum cliente enxerga esse custo (SPEC-001: frio < 500ms).
    let t0 = Instant::now();
    let n = get_violations(&cache).len();
    eprintln!(
        "[nemesis-publisher] warm-up do ledger: {} violations em {:.2}s",
        n,
        t0.elapsed().as_secs_f32()
    );

    for request in server.incoming_requests() {
        // Anti DNS-rebinding: o socket faz bind so em 127.0.0.1, mas um site hostil visitado
        // pelo usuario pode religar seu dominio para 127.0.0.1 e emitir requests same-origin.
        // A defesa canonica de servico localhost e validar o Host: so loopback e aceito.
        let host = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("Host"))
            .map(|h| h.value.as_str().to_string());
        if !is_loopback_host(host.as_deref()) {
            let _ = request.respond(Response::empty(403));
            continue;
        }

        let url = request.url().to_string();
        let method = request.method().clone();

        if method != Method::Get {
            let _ = request.respond(Response::empty(405));
            continue;
        }

        let json = match url.as_str() {
            "/data/summary" => handle_summary(&cache, nemesis_path),
            "/data/doctor" => handle_doctor(&cache),
            "/data/pentest" => handle_pentest(&cache, nemesis_path),
            "/data/sessions" => handle_sessions(&cache),
            p if p.starts_with("/data/violations") => handle_violations(&cache, p),
            _ => {
                let _ = request.respond(Response::empty(404));
                continue;
            }
        };

        let mut response = Response::from_string(&json);
        response = response.with_header(
            Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
        );
        let _ = request.respond(response);
    }

    Ok(())
}

/// Loop da thread de background: mantem o snapshot do doctor FULL atualizado.
/// Roda imediatamente se o snapshot nao existe; depois re-executa quando a idade
/// do arquivo ultrapassa o intervalo. Intervalo 0 = so o boot-run (e o --sync).
fn background_doctor_full(nemesis_path: PathBuf) {
    let interval = config::doctor_full_interval();
    let snap = config::doctor_full_snapshot_path();

    loop {
        let age = mtime(&snap).and_then(|t| t.elapsed().ok());
        let needs = match age {
            None => true, // snapshot ausente: primeira execucao ever
            Some(a) => interval > 0 && a.as_secs() >= interval,
        };

        if needs {
            eprintln!("[nemesis-publisher] doctor FULL em background (snapshot)...");
            let t0 = Instant::now();
            let run = sources::doctor::run_doctor(&nemesis_path);
            match sources::doctor::write_full_snapshot(&snap, &run) {
                Ok(()) => eprintln!(
                    "[nemesis-publisher] snapshot doctor-full atualizado em {:.1}s (verdict: {})",
                    t0.elapsed().as_secs_f32(),
                    run.verdict
                ),
                Err(e) => eprintln!("[nemesis-publisher] ERRO ao gravar snapshot: {}", e),
            }
        }

        if interval == 0 {
            break;
        }
        std::thread::sleep(Duration::from_secs(interval.min(30)));
    }
}

/// Loop da thread de background: sincroniza com o Neon periodicamente.
/// Se DATABASE_URL nao estiver definida, loga uma vez e encerra.
/// Se NEMESIS_SYNC_INTERVAL for 0, nao sincroniza (modo manual apenas).
fn background_neon_sync(nemesis_path: PathBuf) {
    let interval = config::sync_interval();

    if interval == 0 {
        eprintln!("[nemesis-publisher] sync Neon desativado (NEMESIS_SYNC_INTERVAL=0). Use --sync manualmente.");
        return;
    }

    if config::database_url().is_none() {
        eprintln!("[nemesis-publisher] sync Neon: DATABASE_URL nao definida. Sync automatico pulado. Use --sync com DATABASE_URL.");
        return;
    }

    eprintln!("[nemesis-publisher] sync Neon automatico ativo (intervalo: {}s)", interval);

    loop {
        let t0 = Instant::now();
        match crate::neon::sync_all(&nemesis_path) {
            Ok(()) => eprintln!(
                "[nemesis-publisher] sync Neon concluido em {:.1}s",
                t0.elapsed().as_secs_f32()
            ),
            Err(e) => eprintln!(
                "[nemesis-publisher] sync Neon falhou: {:#}", e
            ),
        }
        std::thread::sleep(Duration::from_secs(interval));
    }
}

/// Registro best-effort: so age se ha identidade opt-in ainda nao registrada e o bootstrap
/// secret foi embutido no build. Nunca propaga erro (o --serve nao pode falhar por isto).
fn try_register_if_needed() {
    let Ok(id) = identity::load() else {
        return;
    };
    if !id.opt_in || id.registered_at.is_some() {
        return;
    }
    let Some(secret) = config::bootstrap_secret() else {
        return; // build sem NEMESIS_BOOTSTRAP_SECRET: registro desabilitado (fail-closed)
    };
    let url = config::dashboard_url();
    match crate::publisher::register(&id, secret, &url) {
        Ok(()) => {
            let _ = identity::update(|i| {
                i.registered_at = Some(chrono::Local::now().to_rfc3339());
            });
            eprintln!("[nemesis-publisher] registro concluido no startup do --serve.");
        }
        Err(e) => eprintln!("[nemesis-publisher] registro no startup pulado: {:#}", e),
    }
}

// ---- recursos cacheados (invalidacao por mtime) ----

fn get_violations(cache: &Mutex<ResourceCache>) -> Arc<Vec<Violation>> {
    let path = config::ledger_path();
    let m = mtime(&path);

    let mut guard = cache.lock().unwrap();
    if let (Some(m), Some((cached_m, _, sorted))) = (m, &guard.violations) {
        if *cached_m == m {
            return sorted.clone();
        }
    }

    // mtime mudou (ou primeira leitura): parse INCREMENTAL a partir do estado anterior —
    // custo proporcional ao delta, nao ao ledger inteiro (SPEC-001).
    let prev = guard.violations.take().map(|(_, state, _)| state);
    let state = sources::violations::parse_ledger_incremental(&path, prev);

    let mut sorted_vec = state.items.clone();
    sorted_vec.sort_by(|a, b| b.ts.cmp(&a.ts));
    let sorted = Arc::new(sorted_vec);

    guard.violations = Some((m.unwrap_or(SystemTime::UNIX_EPOCH), state, sorted.clone()));
    sorted
}

/// Doctor da observabilidade = SEMPRE o snapshot FULL (SPEC-001, ressalva full-only).
fn get_doctor(cache: &Mutex<ResourceCache>) -> Arc<DoctorRun> {
    let path = config::doctor_full_snapshot_path();
    let m = mtime(&path);

    match m {
        None => {
            // primeira execucao ever: o boot-run da thread ainda nao concluiu
            Arc::new(sources::doctor::building_snapshot_placeholder())
        }
        Some(m) => {
            {
                let guard = cache.lock().unwrap();
                if let Some((cached_m, data)) = &guard.doctor {
                    if *cached_m == m {
                        return data.clone();
                    }
                }
            }
            let run = sources::doctor::read_full_snapshot(&path)
                .unwrap_or_else(sources::doctor::building_snapshot_placeholder);
            let run = Arc::new(run);
            let mut guard = cache.lock().unwrap();
            guard.doctor = Some((m, run.clone()));
            run
        }
    }
}

fn get_pentest(cache: &Mutex<ResourceCache>, nemesis_path: &std::path::Path) -> Arc<PentestRun> {
    let md_path =
        nemesis_path.join(".nemesis/pentest-nemesis-control/nemesis-defender/pentest-results.md");
    let m = mtime(&md_path);

    if let Some(m) = m {
        let guard = cache.lock().unwrap();
        if let Some((cached_m, data)) = &guard.pentest {
            if *cached_m == m {
                return data.clone();
            }
        }
    }

    let parsed = Arc::new(sources::pentest::read_pentest(&md_path));
    let mut guard = cache.lock().unwrap();
    guard.pentest = Some((m.unwrap_or(SystemTime::UNIX_EPOCH), parsed.clone()));
    parsed
}

// ---- handlers (compostos dos recursos; ZERO subprocesso no request path) ----

fn handle_summary(cache: &Mutex<ResourceCache>, nemesis_path: &std::path::Path) -> String {
    let violations = get_violations(cache);
    let doctor = get_doctor(cache);
    let pentest = get_pentest(cache, nemesis_path);
    let summary = sources::summary::build_summary(&violations, &doctor, &pentest);
    serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string())
}

fn handle_doctor(cache: &Mutex<ResourceCache>) -> String {
    let doctor = get_doctor(cache);
    serde_json::to_string(&*doctor).unwrap_or_else(|_| "{}".to_string())
}

fn handle_pentest(cache: &Mutex<ResourceCache>, nemesis_path: &std::path::Path) -> String {
    let pentest = get_pentest(cache, nemesis_path);
    serde_json::to_string(&*pentest).unwrap_or_else(|_| "{}".to_string())
}

fn handle_sessions(cache: &Mutex<ResourceCache>) -> String {
    let violations = get_violations(cache);
    let env = config::environment();
    let sessions = sources::sessions::derive_sessions(&violations, &env);
    serde_json::to_string(&sessions).unwrap_or_else(|_| "[]".to_string())
}

fn handle_violations(cache: &Mutex<ResourceCache>, path: &str) -> String {
    let all = get_violations(cache);

    let query_str = path.split('?').nth(1).unwrap_or("");
    let params = parse_query(query_str);

    let layer_filter = params.get("layer").cloned();
    let kind_filter = params.get("kind").cloned();
    let search = params.get("search").cloned();
    let page: usize = params.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let page_size: usize = params
        .get("pageSize")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    let mut filtered: Vec<&Violation> = all.iter().collect();
    if let Some(ref layer) = layer_filter {
        filtered.retain(|v| v.layer == *layer);
    }
    if let Some(ref kind) = kind_filter {
        filtered.retain(|v| v.kind == *kind);
    }
    if let Some(ref s) = search {
        let lower = s.to_lowercase();
        filtered.retain(|v| {
            v.message.to_lowercase().contains(&lower)
                || v
                    .target
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&lower)
        });
    }

    let total = filtered.len();
    let start = page.saturating_sub(1) * page_size;
    let items: Vec<_> = filtered.into_iter().skip(start).take(page_size).cloned().collect();

    serde_json::json!({
        "items": items,
        "total": total,
        "page": page,
        "pageSize": page_size,
    })
    .to_string()
}

fn parse_query(q: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in q.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            map.insert(k.to_string(), v.to_string());
        }
    }
    map
}

/// Aceita apenas Host loopback (com ou sem porta), case-insensitive. Defesa anti
/// DNS-rebinding: o bind so em 127.0.0.1 nao basta, porque um dominio hostil pode ser
/// religado para o loopback; a validacao do Host no servidor e a defesa canonica.
/// `None` (Host ausente) e qualquer host nao-loopback = rejeitado.
fn is_loopback_host(host: Option<&str>) -> bool {
    let Some(h) = host else { return false };
    let h = h.trim().to_ascii_lowercase();
    // Extrai o hostname descartando a porta. IPv6 vem entre colchetes: [::1]:8080.
    let hostname = if let Some(rest) = h.strip_prefix('[') {
        match rest.split_once(']') {
            Some((inner, _)) => format!("[{}]", inner),
            None => h.clone(),
        }
    } else {
        h.split(':').next().unwrap_or("").to_string()
    };
    matches!(hostname.as_str(), "127.0.0.1" | "localhost" | "[::1]")
}

#[cfg(test)]
mod tests {
    use super::is_loopback_host;

    #[test]
    fn aceita_loopback_com_e_sem_porta() {
        for h in [
            "127.0.0.1",
            "127.0.0.1:8080",
            "localhost",
            "localhost:8080",
            "LOCALHOST:8080",
            "[::1]",
            "[::1]:8080",
        ] {
            assert!(is_loopback_host(Some(h)), "deveria aceitar: {h}");
        }
    }

    #[test]
    fn rejeita_nao_loopback_e_ausencia() {
        for h in [
            "evil.com",
            "evil.com:8080",
            "attacker.example:8080",
            "0.0.0.0:8080",
            "10.0.0.5",
            "nemesis.local",
            "",
        ] {
            assert!(!is_loopback_host(Some(h)), "deveria rejeitar: {h}");
        }
        assert!(!is_loopback_host(None), "Host ausente deve ser rejeitado");
    }
}
