//! Execucao e parse do nemesis-doctor. Portado de local-source.ts:196-272.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub title: String,
    pub title_en: String,
    pub status: String,
    pub lines: Vec<String>,
    pub lines_en: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorRun {
    pub run_at: String,
    pub verdict: String,
    pub exit_code: i32,
    pub quick: bool,
    pub checks: Vec<DoctorCheck>,
}

/// Grava o snapshot do doctor FULL em disco (escrita atomica: tmp + rename).
/// E a UNICA fonte da observabilidade (SPEC-001: full-only, nunca quick).
pub fn write_full_snapshot(path: &Path, run: &DoctorRun) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let bytes = serde_json::to_vec(run)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

/// Le o snapshot do doctor FULL. None se ausente ou corrompido.
pub fn read_full_snapshot(path: &Path) -> Option<DoctorRun> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Caso-borda da primeira execucao ever: snapshot ainda em construcao.
/// Payload honesto — a UI mostra que o full esta rodando, sem inventar dado.
pub fn building_snapshot_placeholder() -> DoctorRun {
    DoctorRun {
        run_at: chrono::Local::now().to_rfc3339(),
        verdict: "ATENCAO".to_string(),
        exit_code: 0,
        quick: false,
        checks: vec![DoctorCheck {
            title: "Snapshot do doctor full em construcao".to_string(),
            title_en: "Full doctor snapshot being built".to_string(),
            status: "warn".to_string(),
            lines: vec![
                "primeira execucao do doctor full em andamento (background)".to_string(),
                "recarregue em ~40s".to_string(),
            ],
            lines_en: vec![
                "first full doctor run in progress (background)".to_string(),
                "reload in ~40s".to_string(),
            ],
        }],
    }
}

/// Executa nemesis-doctor FULL e retorna DoctorRun.
/// NUNCA chamar dentro de handler HTTP (custo medido: 11-38s wall, ~51s CPU) —
/// somente na thread de background do --serve e no --sync (ISSUE-001/SPEC-001).
pub fn run_doctor(nemesis_path: &std::path::Path) -> DoctorRun {
    let bin_path = nemesis_path.join(".nemesis/target/release/nemesis-doctor");
    let output = Command::new(&bin_path)
        .current_dir(nemesis_path)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_doctor_output(&stdout, out.status.code().unwrap_or(1))
        }
        Err(_) => DoctorRun {
            run_at: chrono::Local::now().to_rfc3339(),
            verdict: "CRITICO".to_string(),
            exit_code: 1,
            quick: false,
            checks: vec![DoctorCheck {
                title: "Doctor binario".to_string(),
                title_en: "Doctor binary".to_string(),
                status: "fail".to_string(),
                lines: vec![
                    "nao foi possivel executar nemesis-doctor".to_string(),
                    format!("path: {}", bin_path.display()),
                ],
                lines_en: vec![
                    "could not run nemesis-doctor".to_string(),
                    format!("path: {}", bin_path.display()),
                ],
            }],
        },
    }
}

fn parse_doctor_output(output: &str, exit_code: i32) -> DoctorRun {
    let lines: Vec<&str> = output.split('\n').collect();
    let mut checks: Vec<DoctorCheck> = Vec::new();
    let mut verdict = "SAUDAVEL".to_string();
    let mut current_title = String::new();
    let mut current_title_en = String::new();
    let mut current_status = "pass".to_string();
    let mut current_lines: Vec<String> = Vec::new();
    let mut current_lines_en: Vec<String> = Vec::new();
    let mut expect_en_title = false;
    let mut expect_en_line = false;

    for line in &lines {
        let trimmed = line.trim();

        // Detecta linhas de grupo no formato real do doctor:
        //   [ OK ] G3 - Inventario de binarios
        //   [SKIP] G1 - Compilacao
        //   [FAIL] Gx - Titulo
        //   [ WARN ] Gx - Titulo
        let is_title = trimmed.starts_with("[ OK ]")
            || trimmed.starts_with("[SKIP]")
            || trimmed.starts_with("[FAIL]")
            || trimmed.starts_with("[ WARN ]");

        if is_title {
            if !current_title.is_empty() {
                checks.push(DoctorCheck {
                    title: current_title.clone(),
                    title_en: current_title_en.clone(),
                    status: current_status.clone(),
                    lines: current_lines.clone(),
                    lines_en: current_lines_en.clone(),
                });
            }
            // Remove o prefixo de status ([ OK ], [SKIP], etc.) para manter só o título
            current_title = trimmed
                .trim_start_matches("[ OK ]")
                .trim_start_matches("[SKIP]")
                .trim_start_matches("[FAIL]")
                .trim_start_matches("[ WARN ]")
                .trim_start_matches("[ NA ]")
                .trim()
                .to_string();
            current_title_en.clear();
            expect_en_title = true;
            if trimmed.starts_with("[SKIP]") {
                current_status = "warn".to_string();
            } else if trimmed.starts_with("[FAIL]") {
                current_status = "fail".to_string();
                verdict = "CRITICO".to_string();
            } else if trimmed.starts_with("[ WARN ]") {
                current_status = "warn".to_string();
                if verdict == "SAUDAVEL" {
                    verdict = "ATENCAO".to_string();
                }
            } else {
                current_status = "pass".to_string();
            }
            current_lines.clear();
            current_lines_en.clear();
            continue;
        }

        // Linha de titulo em ingles: [EN] G3 - Binary inventory
        if expect_en_title && trimmed.starts_with("[EN]") {
            current_title_en = trimmed.trim_start_matches("[EN]").trim().to_string();
            expect_en_title = false;
            continue;
        }

        // Linhas de conteudo (nao vazias, nao separadoras, nao cabecalho)
        if !trimmed.is_empty()
            && !trimmed.starts_with("===")
            && !trimmed.starts_with("NEMESIS DOCTOR")
            && !trimmed.starts_with("VEREDITO")
            && !trimmed.starts_with("[EN]")
        {
            // O status/veredito vem SOMENTE do prefixo do titulo do grupo
            // ([ OK ] / [ WARN ] / [FAIL] / [SKIP]), tratado acima. NAO inferir status a
            // partir do texto explicativo: prosa como "...otherwise FAILED..." (criterio do G7)
            // ou "'warning' does not block" contem as substrings FAIL/WARN e marcava o check
            // como falho por engano, forcando veredito CRITICO com o pentest APROVADO.

            // Linha em ingles: comeca com "EN: " apos a indentacao
            if trimmed.starts_with("EN: ") && expect_en_line {
                current_lines_en.push(trimmed.trim_start_matches("EN: ").to_string());
                expect_en_line = false;
                continue;
            }

            current_lines.push(trimmed.to_string());
            expect_en_line = true;
        }
    }

    if !current_title.is_empty() {
        checks.push(DoctorCheck {
            title: current_title,
            title_en: current_title_en,
            status: current_status,
            lines: current_lines,
            lines_en: current_lines_en,
        });
    }

    DoctorRun {
        run_at: chrono::Local::now().to_rfc3339(),
        verdict,
        exit_code: if exit_code == 0 && checks.iter().all(|c| c.status != "fail") {
            0
        } else {
            1
        },
        quick: false,
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_doctor_output;

    // Regressao: a prosa explicativa do G7 contem "FAILED"/"warning"; o parser NAO pode
    // inferir falha a partir do texto, so do prefixo do titulo. Antes deste fix o veredito
    // virava CRITICO com o pentest APROVADO ([ OK ]).
    #[test]
    fn prosa_com_failed_nao_reprova_check_ok() {
        let out = "\
[ OK ] G7 - Pentest Red-Team (run-pentest.sh)
[EN] G7 - Red-Team Pentest (run-pentest.sh)
        Total: 413 | Corretos: 413 | Taxa: 100.0%
        EN: Verdict: APPROVED if ZERO gaps; otherwise FAILED. No acceptable band.
        APROVADO
[ OK ] G8 - Telemetria e dashboard
[EN] G8 - Telemetry and dashboard
        OK    dashboard hidratada (summary: 30132 violations)

=============================================
 VEREDITO GLOBAL: SAUDAVEL
";
        let run = parse_doctor_output(out, 0);
        assert_eq!(run.verdict, "SAUDAVEL", "prosa com FAILED nao pode virar CRITICO");
        assert_eq!(run.exit_code, 0);
        let g7 = run.checks.iter().find(|c| c.title.starts_with("G7")).unwrap();
        assert_eq!(g7.status, "pass", "G7 [ OK ] deve ser pass mesmo com 'FAILED' na prosa");
    }

    // [FAIL] real no titulo continua reprovando.
    #[test]
    fn fail_no_titulo_reprova() {
        let out = "\
[FAIL] G6 - Daemon nemesis-defender
[EN] G6 - Nemesis-defender daemon
        PID file ausente - daemon nao esta rodando.
";
        let run = parse_doctor_output(out, 1);
        assert_eq!(run.verdict, "CRITICO");
        assert_eq!(run.exit_code, 1);
    }
}
