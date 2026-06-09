mod checks;
mod report;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("nemesis-doctor - diagnostico de saude do Nemesis");
        println!("Uso: nemesis-doctor [--quick]");
        println!("  --quick   Pula grupos pesados (G1 compile, G2 testes, G7 pentest)");
        return;
    }

    let quick = args.iter().any(|a| a == "--quick");
    eprintln!(
        "[nemesis-doctor] Executando diagnostico{}...",
        if quick { " (modo rapido)" } else { "" }
    );

    let results = checks::run_all(quick);
    let code = report::render(&results);
    std::process::exit(code);
}
