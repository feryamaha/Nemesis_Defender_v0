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
    pub title_en: String,
    pub status: CheckStatus,
    pub lines: Vec<String>,
    pub lines_en: Vec<String>,
}

impl CheckResult {
    pub fn new(title: impl Into<String>, title_en: impl Into<String>) -> Self {
        CheckResult {
            title: title.into(),
            title_en: title_en.into(),
            status: CheckStatus::Ok,
            lines: Vec::new(),
            lines_en: Vec::new(),
        }
    }

    pub fn status(mut self, s: CheckStatus) -> Self {
        self.status = s;
        self
    }

    pub fn line(mut self, pt: impl Into<String>, en: impl Into<String>) -> Self {
        self.lines.push(pt.into());
        self.lines_en.push(en.into());
        self
    }

    pub fn push(&mut self, pt: impl Into<String>, en: impl Into<String>) {
        self.lines.push(pt.into());
        self.lines_en.push(en.into());
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
        if !r.title_en.is_empty() {
            println!("[EN] {}", r.title_en);
        }
        for i in 0..r.lines.len() {
            println!("        {}", r.lines[i]);
            if i < r.lines_en.len() && !r.lines_en[i].is_empty() {
                println!("        EN: {}", r.lines_en[i]);
            }
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
