//! G8 - Telemetria e dashboard (publisher opt-in).
//!
//! Check ADITIVO: telemetria e opt-in, portanto este gate nunca reprova o doctor.
//! Sem opt-in = Skip (por design, nao e problema); problema em maquina com opt-in = Warn.
//! Sub-checks (issue 010): snapshot doctor-full.json, sync Neon (sync-state.json),
//! processo `--serve` e hidratacao da dashboard (/data/summary).

use crate::checks::nemesis_dir;
use crate::report::{CheckResult, CheckStatus};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

/// Limite de "fresco" para snapshot e sync (issue 010: stale > 1h = ATENCAO).
const STALE_SECS: u64 = 3600;

/// Porta do publisher --serve. Espelho de `nemesis-publisher/src/config.rs`
/// (`publisher_port`): o doctor nao depende do crate do publisher; mesma env var,
/// mesmo default. Mudanca la exige mudanca aqui.
fn publisher_port() -> u16 {
    std::env::var("NEMESIS_PUBLISHER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
}

fn age_of(path: &Path) -> Option<Duration> {
    let mtime = std::fs::metadata(path).ok()?.modified().ok()?;
    std::time::SystemTime::now().duration_since(mtime).ok()
}

fn fmt_age(d: Duration) -> String {
    let s = d.as_secs();
    if s < 3600 {
        format!("{}min", s / 60)
    } else {
        format!("{:.1}h", s as f64 / 3600.0)
    }
}

/// PID do processo `nemesis-publisher --serve`, se houver.
#[cfg(target_os = "linux")]
fn serve_pid() -> Option<u32> {
    for entry in std::fs::read_dir("/proc").ok()?.flatten() {
        let name = entry.file_name();
        let Ok(pid) = name.to_string_lossy().parse::<u32>() else {
            continue;
        };
        if let Ok(raw) = std::fs::read(format!("/proc/{}/cmdline", pid)) {
            let cmd = String::from_utf8_lossy(&raw);
            if cmd.contains("nemesis-publisher") && cmd.contains("--serve") {
                return Some(pid);
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn serve_pid() -> Option<u32> {
    // macOS/BSD: sem /proc; mesmo metodo `ps` do daemon.rs.
    let out = std::process::Command::new("ps")
        .args(["-axo", "pid=,command="])
        .output()
        .ok()?;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if line.contains("nemesis-publisher") && line.contains("--serve") {
            return line.trim().split_whitespace().next()?.parse().ok();
        }
    }
    None
}

/// GET http://127.0.0.1:{port}/data/summary com HTTP/1.1 manual (sem dependencia nova).
/// Retorna o body se a resposta foi 200. O server usa Connection: close + Content-Length
/// (tiny_http), entao read_to_string ate EOF e suficiente.
fn get_summary(port: u16) -> Option<String> {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let req = format!(
        "GET /data/summary HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        port
    );
    stream.write_all(req.as_bytes()).ok()?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf).ok()?;
    let ok = buf.starts_with("HTTP/1.1 200") || buf.starts_with("HTTP/1.0 200");
    if !ok {
        return None;
    }
    buf.split_once("\r\n\r\n").map(|(_, body)| body.to_string())
}

/// DATABASE_URL disponivel para o sync? No ambiente do doctor OU no publisher.env
/// (o publisher carrega esse arquivo no startup; o doctor so precisa saber se existe).
fn database_url_configured(telemetry: &Path) -> bool {
    if std::env::var_os("DATABASE_URL").is_some() {
        return true;
    }
    std::fs::read_to_string(telemetry.join("publisher.env"))
        .map(|c| {
            c.lines()
                .map(str::trim)
                .any(|l| l.starts_with("DATABASE_URL=") && l.len() > "DATABASE_URL=".len())
        })
        .unwrap_or(false)
}

pub fn run() -> CheckResult {
    let mut res = CheckResult::new(
        "G8 - Telemetria e dashboard",
        "G8 - Telemetry and dashboard",
    );
    let telemetry = nemesis_dir().join("telemetry");

    // Opt-in e manual (nemesis-publisher --opt-in). Sem ele, nada a diagnosticar.
    let opt_in = std::fs::read_to_string(telemetry.join("identity.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("opt_in").and_then(|b| b.as_bool()))
        .unwrap_or(false);
    if !opt_in {
        res.push(
            "Telemetria desativada (opt-in e manual: nemesis-publisher --opt-in).",
            "Telemetry disabled (opt-in is manual: nemesis-publisher --opt-in).",
        );
        return res.status(CheckStatus::Skip);
    }

    let mut warn = false;

    // 1) Snapshot local doctor-full.json (fonte da observabilidade do doctor).
    let snapshot = telemetry.join("doctor-full.json");
    match age_of(&snapshot) {
        Some(age) if age.as_secs() <= STALE_SECS => {
            res.push(
                format!("OK    snapshot doctor-full.json (idade: {})", fmt_age(age)),
                format!("OK    doctor-full.json snapshot (age: {})", fmt_age(age)),
            );
        }
        Some(age) => {
            warn = true;
            res.push(
                format!(
                    "ATENCAO snapshot doctor-full.json stale (idade: {}, limite 1h).",
                    fmt_age(age)
                ),
                format!(
                    "WARNING doctor-full.json snapshot stale (age: {}, limit 1h).",
                    fmt_age(age)
                ),
            );
        }
        None => {
            warn = true;
            res.push(
                "ATENCAO snapshot doctor-full.json ausente (o --serve o gera em background).",
                "WARNING doctor-full.json snapshot missing (--serve generates it in background).",
            );
        }
    }

    // 2) Sync Neon (sync-state.json e salvo a cada sync; o mtime e o ultimo sync).
    let sync_state = telemetry.join("sync-state.json");
    if !database_url_configured(&telemetry) {
        res.push(
            "INFO  sync Neon manual/desativado (DATABASE_URL nao configurada).",
            "INFO  Neon sync manual/disabled (DATABASE_URL not configured).",
        );
    } else {
        match age_of(&sync_state) {
            Some(age) if age.as_secs() <= STALE_SECS => {
                res.push(
                    format!("OK    sync Neon (ultimo: ha {})", fmt_age(age)),
                    format!("OK    Neon sync (last: {} ago)", fmt_age(age)),
                );
            }
            Some(age) => {
                warn = true;
                res.push(
                    format!(
                        "ATENCAO sync Neon stale (ultimo: ha {}, limite 1h).",
                        fmt_age(age)
                    ),
                    format!("WARNING Neon sync stale (last: {} ago, limit 1h).", fmt_age(age)),
                );
            }
            None => {
                warn = true;
                res.push(
                    "ATENCAO sync Neon nunca executado (sync-state.json ausente).",
                    "WARNING Neon sync never ran (sync-state.json missing).",
                );
            }
        }
    }

    // 3) Processo --serve (comportamento: TCP na porta; identidade: PID).
    let port = publisher_port();
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let serve_up = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).is_ok();
    if serve_up {
        match serve_pid() {
            Some(pid) => res.push(
                format!("OK    publisher --serve (PID {}, porta {})", pid, port),
                format!("OK    publisher --serve (PID {}, port {})", pid, port),
            ),
            None => res.push(
                format!("OK    porta {} respondendo (PID nao identificado).", port),
                format!("OK    port {} responding (PID not identified).", port),
            ),
        }
    } else {
        warn = true;
        res.push(
            format!(
                "ATENCAO publisher --serve nao encontrado (porta {} sem resposta). Acao: nemesis-publisher --install-service",
                port
            ),
            format!(
                "WARNING publisher --serve not found (port {} not responding). Action: nemesis-publisher --install-service",
                port
            ),
        );
    }

    // 4) Hidratacao da dashboard (so faz sentido com o serve no ar).
    if !serve_up {
        res.push(
            "PULAR dashboard hidratada (serve offline).",
            "SKIP  dashboard hydrated (serve offline).",
        );
    } else {
        match get_summary(port) {
            Some(body) => {
                // Campo verificado na resposta real do /data/summary: "total_violations"
                // (server.rs, handle_summary). "total" fica como fallback defensivo.
                let total = serde_json::from_str::<serde_json::Value>(&body)
                    .ok()
                    .and_then(|v| {
                        v.get("total_violations")
                            .or_else(|| v.get("total"))
                            .and_then(|t| t.as_u64())
                    });
                match total {
                    Some(t) => res.push(
                        format!("OK    dashboard hidratada (summary: {} violations)", t),
                        format!("OK    dashboard hydrated (summary: {} violations)", t),
                    ),
                    None => res.push(
                        "OK    dashboard respondeu 200 (summary sem campo total).",
                        "OK    dashboard responded 200 (summary without total field).",
                    ),
                }
            }
            None => {
                warn = true;
                res.push(
                    "ATENCAO /data/summary offline ou invalido com serve no ar.",
                    "WARNING /data/summary offline or invalid while serve is up.",
                );
            }
        }
    }

    if warn {
        res.status(CheckStatus::Warn)
    } else {
        res.status(CheckStatus::Ok)
    }
}
