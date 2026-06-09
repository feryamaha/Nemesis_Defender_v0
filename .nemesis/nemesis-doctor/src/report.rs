//! Modelo de status e renderizacao do relatorio do nemesis-doctor.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Ok,
    Warn,
    Fail,
    Skip,
    /// Usada apenas em plataformas nao-Linux (eBPF nao se aplica).
    #[allow(dead_code)]
    Na,
}

impl CheckStatus {
    pub fn symbol(&self) -> &'static str {
        match self {
            CheckStatus::Ok => "[ OK ]",
            CheckStatus::Warn => "[WARN]",
            CheckStatus::Fail => "[FAIL]",
            CheckStatus::Skip => "[SKIP]",
            CheckStatus::Na => "[ NA ]",
        }
    }
}

pub struct CheckResult {
    pub title: String,
    pub status: CheckStatus,
    pub lines: Vec<String>,
}

impl CheckResult {
    pub fn new(title: impl Into<String>) -> Self {
        CheckResult {
            title: title.into(),
            status: CheckStatus::Ok,
            lines: Vec::new(),
        }
    }

    pub fn status(mut self, s: CheckStatus) -> Self {
        self.status = s;
        self
    }

    pub fn line(mut self, l: impl Into<String>) -> Self {
        self.lines.push(l.into());
        self
    }

    pub fn push(&mut self, l: impl Into<String>) {
        self.lines.push(l.into());
    }
}

/// Imprime o relatorio e retorna o exit code (0 = ok/warn, 1 = critico).
pub fn render(results: &[CheckResult]) -> i32 {
    println!("\n=============================================");
    println!("        NEMESIS DOCTOR - RELATORIO");
    println!("=============================================\n");

    let mut has_fail = false;
    let mut has_warn = false;

    for r in results {
        println!("{} {}", r.status.symbol(), r.title);
        for l in &r.lines {
            println!("        {}", l);
        }
        println!();
        match r.status {
            CheckStatus::Fail => has_fail = true,
            CheckStatus::Warn => has_warn = true,
            _ => {}
        }
    }

    let (verdict, code) = if has_fail {
        ("CRITICO", 1)
    } else if has_warn {
        ("ATENCAO", 0)
    } else {
        ("SAUDAVEL", 0)
    };

    println!("=============================================");
    println!(" VEREDITO GLOBAL: {}", verdict);
    println!("=============================================");
    code
}
