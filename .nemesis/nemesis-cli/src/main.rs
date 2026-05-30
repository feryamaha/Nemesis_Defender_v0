mod detect;
mod download;
mod hooks;
mod init;
mod status;
mod templates;
mod uninstall;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nemesis-cli")]
#[command(about = "Nemesis Framework installer CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Instalar Nemesis neste projeto
    Init {
        /// Caminho dos binarios Nemesis compilados
        #[arg(long)]
        from: Option<String>,
    },
    /// Verificar status de instalacao
    Status,
    /// Desinstalar Nemesis deste projeto
    Uninstall,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let target_dir = std::env::current_dir()?;

    match cli.command {
        Commands::Init { from } => {
            // Stack detection is now handled internally by init::init()
            init::init(&target_dir, from)?;
        },
        Commands::Status => {
            status::status(&target_dir)?;
        },
        Commands::Uninstall => {
            uninstall::uninstall(&target_dir)?;
        },
    }

    Ok(())
}
