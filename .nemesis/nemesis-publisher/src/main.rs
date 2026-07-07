//! nemesis-publisher — publisher de telemetria opt-in para dashboard.

use nemesis_publisher::config;
use nemesis_publisher::identity;
use nemesis_publisher::ledger;
use nemesis_publisher::publisher;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    let result = match args.get(1).map(|s| s.as_str()) {
        Some("--init") => cmd_init(
            args.iter().any(|a| a == "--opt-in"),
            args.iter().any(|a| a == "--official"),
        ),
        Some("--opt-in") => cmd_opt_in(),
        Some("--opt-out") => cmd_opt_out(),
        Some("--register") => cmd_register(),
        Some("--publish") => cmd_publish(),
        Some("--serve") => cmd_serve(),
        Some("--sync") => cmd_sync(),
        Some("--status") => cmd_status(),
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        _ => {
            print_help();
            std::process::exit(1);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[nemesis-publisher] ERRO: {:#}", e);
            ExitCode::FAILURE
        }
    }
}

fn cmd_init(opt_in: bool, official: bool) -> anyhow::Result<()> {
    if !opt_in {
        println!("[nemesis-publisher] Telemetria desativada. Use --opt-in para ativar.");
        return Ok(());
    }
    if identity::exists() {
        println!("[nemesis-publisher] Identidade ja existe. Use --status para ver.");
        return Ok(());
    }
    let mut id = identity::create_identity();
    if official {
        id.environment = "official".to_string();
    }
    identity::save(&id)?;
    println!("[nemesis-publisher] Identidade criada (environment={}).", id.environment);
    Ok(())
}

fn cmd_opt_in() -> anyhow::Result<()> {
    if !identity::exists() {
        let id = identity::create_identity();
        identity::save(&id)?;
        println!("[nemesis-publisher] Identidade criada. Use --register para registrar na dashboard.");
        return Ok(());
    }
    let mut id = identity::load()?;
    if id.opt_in {
        println!("[nemesis-publisher] Opt-in ja ativado.");
        return Ok(());
    }
    id.opt_in = true;
    identity::save(&id)?;
    println!("[nemesis-publisher] Opt-in ativado. Use --register para registrar na dashboard.");
    Ok(())
}

fn cmd_opt_out() -> anyhow::Result<()> {
    if !identity::exists() {
        println!("[nemesis-publisher] Telemetria ja desativada.");
        return Ok(());
    }
    identity::update(|id| {
        id.opt_in = false;
    })?;
    println!("[nemesis-publisher] Opt-out aplicado. Nada sera enviado.");
    Ok(())
}

fn cmd_register() -> anyhow::Result<()> {
    let id = identity::load()?;
    if !id.opt_in {
        anyhow::bail!("Opt-in nao ativado.");
    }
    if id.registered_at.is_some() {
        println!("[nemesis-publisher] Ja registrado em {}.", id.registered_at.unwrap());
        return Ok(());
    }
    let secret = config::bootstrap_secret()
        .ok_or_else(|| anyhow::anyhow!("Bootstrap secret nao configurado no build. Nao pode registrar."))?;
    let url = config::dashboard_url();
    publisher::register(&id, secret, &url)?;
    identity::update(|identity| {
        identity.registered_at = Some(chrono::Local::now().to_rfc3339());
    })?;
    Ok(())
}

fn cmd_publish() -> anyhow::Result<()> {
    let id = identity::load()?;
    if !id.opt_in {
        println!("[nemesis-publisher] Opt-in nao ativado. Nada enviado.");
        return Ok(());
    }
    if id.registered_at.is_none() {
        anyhow::bail!("Nao registrado. Use --register primeiro.");
    }
    let ledger_path = config::ledger_path();
    if !ledger_path.exists() {
        println!("[nemesis-publisher] Nenhum bloqueio registrado.");
        return Ok(());
    }
    let agg = ledger::aggregate(&ledger_path);
    if agg.total_blocks == 0 {
        println!("[nemesis-publisher] Nenhum bloqueio registrado.");
        return Ok(());
    }
    let url = config::dashboard_url();
    publisher::publish(&id, &agg, &url)?;
    Ok(())
}

fn cmd_serve() -> anyhow::Result<()> {
    let port = config::publisher_port();
    let nemesis_path = config::nemesis_repo_root();
    nemesis_publisher::server::run(port, &nemesis_path)
}

fn cmd_sync() -> anyhow::Result<()> {
    let nemesis_path = config::nemesis_repo_root();
    nemesis_publisher::neon::sync_all(&nemesis_path)
}

fn cmd_status() -> anyhow::Result<()> {
    if !identity::exists() {
        println!("[nemesis-publisher] Telemetria desativada (sem identidade).");
        return Ok(());
    }
    let id = identity::load()?;
    println!("==============================================================");
    println!(" NEMESIS — Publisher de Telemetria");
    println!("==============================================================");
    println!("Install ID    : {}", id.install_id);
    println!("Alias         : {}", id.alias);
    println!("Opt-in        : {}", id.opt_in);
    println!("Criado em     : {}", id.created_at);
    match &id.registered_at {
        Some(ts) => println!("Registrado em : {}", ts),
        None => println!("Registrado em : (nao registrado)"),
    }
    println!();
    if id.opt_in {
        let ledger_path = config::ledger_path();
        let agg = ledger::aggregate(&ledger_path);
        println!("Total de bloqueios: {}", agg.total_blocks);
        println!();
        println!("-- Por CAMADA --");
        for (layer, count) in &agg.by_layer {
            println!("  {:<18} {:>7}", layer.as_str(), count);
        }
        println!();
        println!("-- Por NATUREZA --");
        for (nature, count) in &agg.by_nature {
            println!("  {:<18} {:>7}", nature.as_str(), count);
        }
    } else {
        println!("Telemetria desativada (opt_in: false).");
    }
    Ok(())
}

fn print_help() {
    eprintln!("[nemesis-publisher] Publisher de telemetria opt-in para dashboard");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  nemesis-publisher --init [--opt-in] [--official]  Criar identidade (market ou official)");
    eprintln!("  nemesis-publisher --opt-in            Ativar opt-in (gera identidade se necessario)");
    eprintln!("  nemesis-publisher --opt-out           Desativar opt-in");
    eprintln!("  nemesis-publisher --register          Registrar install na dashboard");
    eprintln!("  nemesis-publisher --publish           Enviar contadores agregados para a dashboard");
    eprintln!("  nemesis-publisher --serve             Iniciar servidor HTTP local (dashboard local)");
    eprintln!("  nemesis-publisher --sync              Sincronizar dados com Neon Postgres");
    eprintln!("  nemesis-publisher --status            Mostrar estado da telemetria");
    eprintln!("  nemesis-publisher --help              Ajuda");
    eprintln!();
    eprintln!("Environment variables:");
    eprintln!("  NEMESIS_DASHBOARD_URL                 URL base da dashboard (default: https://nemesis-defender.vercel.app)");
    eprintln!("  NEMESIS_PUBLISHER_PORT                Porta do servidor --serve (default: 8080)");
    eprintln!("  NEMESIS_ENVIRONMENT                   Environment: official ou market (default: official)");
    eprintln!("  DATABASE_URL                          URL de conexao Neon Postgres para --sync");
}
