//! Cliente Postgres para escrita direta no Neon (modo --sync).

use crate::config;
use crate::identity;
use crate::sources;
use crate::sync_state;
use anyhow::{Context, Result};
use postgres::Client;

/// Remove o parametro `channel_binding=...` da connection string (query e/ou libpq).
/// native-tls nao fornece tls-server-end-point; manter `require` quebraria o SCRAM-PLUS.
fn strip_channel_binding(url: &str) -> String {
    url.split('&')
        .filter(|seg| !seg.starts_with("channel_binding="))
        .collect::<Vec<_>>()
        .join("&")
}

/// Insere violations em lotes (multi-row VALUES), idempotente por PK(id, install_id).
fn insert_violations_batched(
    client: &mut Client,
    install_id: &str,
    rows: &[&sources::violations::Violation],
) -> Result<()> {
    use postgres::types::ToSql;
    const COLS: usize = 10;
    const CHUNK: usize = 1000; // 1000 * 10 = 10_000 params (< limite 65_535 do protocolo)

    for chunk in rows.chunks(CHUNK) {
        let mut stmt = String::from(
            "INSERT INTO violations (id, install_id, ts, date, time, layer, kind, message, target, raw_ok) VALUES ",
        );
        let mut params: Vec<&(dyn ToSql + Sync)> = Vec::with_capacity(chunk.len() * COLS);
        for (i, v) in chunk.iter().enumerate() {
            if i > 0 {
                stmt.push(',');
            }
            let b = i * COLS;
            stmt.push_str(&format!(
                "(${},${},${},${},${},${},${},${},${},${})",
                b + 1, b + 2, b + 3, b + 4, b + 5, b + 6, b + 7, b + 8, b + 9, b + 10
            ));
            params.push(&v.id);
            params.push(&install_id);
            params.push(&v.ts);
            params.push(&v.date);
            params.push(&v.time);
            params.push(&v.layer);
            params.push(&v.kind);
            params.push(&v.message);
            params.push(&v.target);
            params.push(&v.raw_ok);
        }
        stmt.push_str(" ON CONFLICT (id, install_id) DO NOTHING");
        client.execute(stmt.as_str(), &params).context("insert violations em lote")?;
    }
    Ok(())
}

/// Cria tabelas se nao existirem (DDL idempotente).
fn ensure_tables(client: &mut Client) -> Result<()> {
    let stmts = [
        "CREATE TABLE IF NOT EXISTS violations (
            id TEXT NOT NULL,
            install_id TEXT NOT NULL,
            ts TEXT NOT NULL,
            date TEXT NOT NULL,
            time TEXT NOT NULL,
            layer TEXT NOT NULL,
            kind TEXT NOT NULL,
            message TEXT NOT NULL,
            target TEXT,
            raw_ok BOOLEAN NOT NULL,
            PRIMARY KEY(id, install_id)
        )",
        "CREATE TABLE IF NOT EXISTS doctor_runs (
            id SERIAL PRIMARY KEY,
            install_id TEXT NOT NULL,
            run_at TEXT NOT NULL,
            verdict TEXT NOT NULL,
            exit_code INT NOT NULL,
            quick BOOLEAN NOT NULL,
            checks JSONB NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS pentest_runs (
            id SERIAL PRIMARY KEY,
            install_id TEXT NOT NULL,
            run_at TEXT NOT NULL,
            total INT NOT NULL,
            blocked INT NOT NULL,
            results JSONB NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT NOT NULL,
            install_id TEXT NOT NULL,
            type TEXT NOT NULL,
            environment TEXT NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT NOT NULL,
            total_blocks INT NOT NULL,
            by_layer JSONB NOT NULL,
            by_kind JSONB NOT NULL,
            PRIMARY KEY(id, install_id)
        )",
    ];

    for stmt in &stmts {
        client.execute(*stmt, &[]).context("criar tabela")?;
    }
    Ok(())
}

/// Sincroniza todas as fontes com o Neon.
pub fn sync_all(nemesis_path: &std::path::Path) -> Result<()> {
    let database_url = config::database_url()
        .ok_or_else(|| anyhow::anyhow!("DATABASE_URL nao definida"))?;

    // Neon exige TLS (sslmode=require). O `postgres` sync usa NoTls por padrao — por isso
    // o --sync falhava com "no TLS implementation configured". Usamos native-tls (OpenSSL).
    // `channel_binding=require` e removido: native-tls nao expoe o binding (tls-server-end-point),
    // e a exigencia faria o SCRAM-PLUS falhar; sslmode=require ja garante o canal cifrado.
    let database_url = strip_channel_binding(&database_url);
    let connector = native_tls::TlsConnector::builder()
        .build()
        .context("construir TLS connector (native-tls/OpenSSL)")?;
    let connector = postgres_native_tls::MakeTlsConnector::new(connector);
    let mut client = Client::connect(&database_url, connector)
        .context("conectar ao Neon")?;

    ensure_tables(&mut client)?;

    let id = identity::load()?;
    let install_id = &id.install_id;
    let env = config::environment();

    let mut state = sync_state::load()?;

    // 1. Violations
    let ledger_path = config::ledger_path();
    let violations = sources::violations::parse_ledger(&ledger_path);
    let new_violations: Vec<&sources::violations::Violation> = violations
        .iter()
        .filter(|v| {
            if let Some(ref last_ts) = state.violations_last_ts {
                v.ts.as_str() > last_ts.as_str()
            } else {
                true
            }
        })
        .collect();

    // INSERT em LOTE (multi-row): 19k linhas 1-a-1 sobre a rede ao Neon leva ~40min.
    // Chunks de 1000 reduzem para ~20 round-trips (SPEC: sync nao pode ser inviavel).
    insert_violations_batched(&mut client, install_id, &new_violations)?;

    if let Some(last) = violations.first() {
        state.violations_last_ts = Some(last.ts.clone());
    }

    // 2. Doctor (FULL — e aproveita o run para atualizar o snapshot local do --serve)
    let doctor = sources::doctor::run_doctor(nemesis_path);
    if let Err(e) =
        sources::doctor::write_full_snapshot(&config::doctor_full_snapshot_path(), &doctor)
    {
        eprintln!("[nemesis-publisher] aviso: falha ao gravar doctor-full.json: {}", e);
    }
    let checks_json = serde_json::to_string(&doctor.checks)?;
    client.execute(
        "INSERT INTO doctor_runs (install_id, run_at, verdict, exit_code, quick, checks)
         VALUES ($1, $2, $3, $4, $5, $6::text::jsonb)",
        &[install_id, &doctor.run_at, &doctor.verdict, &doctor.exit_code, &doctor.quick, &checks_json],
    )?;
    state.doctor_last_run_at = Some(doctor.run_at.clone());

    // 3. Pentest
    let md_path = nemesis_path.join(".nemesis/pentest-nemesis-control/nemesis-defender/pentest-results.md");
    let pentest = sources::pentest::read_pentest(&md_path);
    let results_json = serde_json::to_string(&pentest.results)?;
    client.execute(
        "INSERT INTO pentest_runs (install_id, run_at, total, blocked, results)
         VALUES ($1, $2, $3, $4, $5::text::jsonb)",
        &[install_id, &pentest.run_at, &(pentest.total as i32), &(pentest.blocked as i32), &results_json],
    )?;
    state.pentest_last_run_at = Some(pentest.run_at.clone());

    // 4. Sessions
    let sessions = sources::sessions::derive_sessions(&violations, &env);
    for sess in &sessions {
        let by_layer_json = serde_json::to_string(&sess.by_layer)?;
        let by_kind_json = serde_json::to_string(&sess.by_kind)?;
        client.execute(
            "INSERT INTO sessions (id, install_id, type, environment, started_at, ended_at, total_blocks, by_layer, by_kind)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8::text::jsonb, $9::text::jsonb)
             ON CONFLICT (id, install_id) DO UPDATE SET
               type = EXCLUDED.type, environment = EXCLUDED.environment,
               started_at = EXCLUDED.started_at, ended_at = EXCLUDED.ended_at,
               total_blocks = EXCLUDED.total_blocks,
               by_layer = EXCLUDED.by_layer, by_kind = EXCLUDED.by_kind",
            &[
                &sess.id, install_id, &sess.session_type, &sess.environment,
                &sess.started_at, &sess.ended_at, &(sess.total_blocks as i32),
                &by_layer_json, &by_kind_json,
            ],
        )?;
    }

    sync_state::save(&state)?;

    println!(
        "[nemesis-publisher] Sincronizacao com Neon concluida: {} violations, {} sessions, doctor={}, pentest={} ({} blocked)",
        new_violations.len(),
        sessions.len(),
        doctor.verdict,
        pentest.total,
        pentest.blocked,
    );

    Ok(())
}
