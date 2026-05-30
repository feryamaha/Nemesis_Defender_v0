use std::env;
use std::path::PathBuf;

fn main() {
    println!("\n🛡️  Nemesis Install Hooks\n");

    let project_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let hooks_dir = project_dir.join(".nemesis").join("hooks");
    let shell_script = hooks_dir.join("nemesis-pretool-check.sh");
    let ps_script = hooks_dir.join("nemesis-pretool-check.ps1");

    // Verificar se diretório de hooks existe
    if !hooks_dir.exists() {
        println!("📁 Criando diretório de hooks...");
        std::fs::create_dir_all(&hooks_dir).expect("Failed to create hooks directory");
    }

    // Verificar scripts de hook
    let check_script = |script_path: &PathBuf, name: &str| -> bool {
        if script_path.exists() {
            println!("✅ {} encontrado", name);
            true
        } else {
            println!("❌ {} NÃO encontrado em: {}", name, script_path.display());
            false
        }
    };

    let shell_ok = check_script(&shell_script, "Shell script (Bash)");
    let ps_ok = check_script(&ps_script, "PowerShell script");

    if shell_ok && ps_ok {
        println!("\n✅ Todos os hooks estão instalados corretamente!\n");
        println!("Os workflows agora usarão PreToolUse hooks para enforcement determinístico.");
        std::process::exit(0);
    } else {
        println!("\n⚠️  Alguns hooks não foram encontrados.");
        println!("Verifique se os arquivos existem em:");
        println!("  - {}", shell_script.display());
        println!("  - {}\n", ps_script.display());
        std::process::exit(1);
    }
}
