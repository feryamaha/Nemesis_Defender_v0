use regex::Regex;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CommandMapping {
    pub original: String,
    pub adapted: String,
    pub package_manager: String,
}

#[derive(Debug, Clone)]
pub struct AdaptedCommand {
    pub command: String,
    pub args: Vec<String>,
    pub package_manager: String,
    pub original_command: String,
    pub blocked: bool,
    pub block_reason: Option<String>,
}

// ============================================================
// WHITELIST DE COMANDOS BUN PERMITIDOS
// ============================================================
lazy_static::lazy_static! {
    static ref ALLOWED_BUN_COMMANDS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("install");
        set.insert("add");
        set.insert("add -d");
        set.insert("run");
        set.insert("dev");
        set.insert("build");
        set.insert("test");
        set.insert("lint");
        set.insert("type-check");
        set.insert("tsc --noEmit");
        set.insert("generate:component");
        set.insert("format");
        set.insert("check");
        set.insert("x"); // bunx
        set
    };

    static ref ALLOWED_BUN_PREFIXES: Vec<&'static str> = vec![
        "add ", "add -d ", "run ", "x ",
    ];

    static ref SHELL_METACHAR_PATTERN: Regex = Regex::new(r"[;&|`$(){}[\]<>\\!#*?~]").unwrap();
}

fn sanitize_args(args: &[String]) -> (bool, Vec<String>, Option<String>) {
    for arg in args {
        if SHELL_METACHAR_PATTERN.is_match(arg) {
            return (false, Vec::new(), Some(arg.clone()));
        }
        if arg.len() > 256 {
            return (false, Vec::new(), Some(format!("[arg too long: {}...]", &arg[..32.min(arg.len())])));
        }
    }
    (true, args.to_vec(), None)
}

fn blocked_command(original_command: &str, reason: &str) -> AdaptedCommand {
    AdaptedCommand {
        command: String::new(),
        args: Vec::new(),
        package_manager: "blocked".to_string(),
        original_command: original_command.to_string(),
        blocked: true,
        block_reason: Some(reason.to_string()),
    }
}

pub struct PackageManagerAdapter {
    host_package_manager: String,
}

impl PackageManagerAdapter {
    pub fn new(host_package_manager: &str) -> Self {
        Self {
            host_package_manager: host_package_manager.to_string(),
        }
    }

    pub fn adapt_command(&self, bun_command: &str) -> AdaptedCommand {
        if bun_command.is_empty() {
            return blocked_command(bun_command, "Comando inválido ou vazio");
        }

        // Limite de comprimento total
        if bun_command.len() > 512 {
            return blocked_command(bun_command, "Comando excede comprimento máximo permitido");
        }

        let normalized_command = bun_command.trim();
        let command_without_bun = normalized_command.trim_start_matches("bun ").trim();

        // Verificar whitelist de comandos permitidos
        if !self.is_allowed_bun_command(command_without_bun) {
            let allowed: Vec<_> = ALLOWED_BUN_COMMANDS.iter().map(|&s| s).collect();
            return blocked_command(
                bun_command,
                &format!(
                    "Comando bun não está na whitelist de comandos permitidos: \"{}\". Comandos permitidos: {}",
                    command_without_bun,
                    allowed.join(", ")
                ),
            );
        }

        match self.host_package_manager.as_str() {
            "bun" => {
                let raw_args: Vec<_> = command_without_bun.split_whitespace().map(|s| s.to_string()).collect();
                let (safe, args, rejected) = sanitize_args(&raw_args);
                if !safe {
                    return blocked_command(
                        bun_command,
                        &format!("Argumento contém metacaracteres de shell: {}", rejected.unwrap_or_default()),
                    );
                }
                AdaptedCommand {
                    command: "bun".to_string(),
                    args,
                    package_manager: "bun".to_string(),
                    original_command: bun_command.to_string(),
                    blocked: false,
                    block_reason: None,
                }
            }
            "yarn" => self.adapt_bun_to_yarn(bun_command, command_without_bun),
            "npm" => self.adapt_bun_to_npm(bun_command, command_without_bun),
            "pnpm" => self.adapt_bun_to_pnpm(bun_command, command_without_bun),
            _ => {
                eprintln!("[PackageManagerAdapter] Package manager não suportado: {}", self.host_package_manager);
                blocked_command(
                    bun_command,
                    &format!("Gerenciador de pacotes não suportado: {}. Suportados: bun, yarn, npm, pnpm", self.host_package_manager),
                )
            }
        }
    }

    fn is_allowed_bun_command(&self, command: &str) -> bool {
        // Comando exato na whitelist
        if ALLOWED_BUN_COMMANDS.contains(command) {
            return true;
        }

        // Prefixo permitido com argumentos
        ALLOWED_BUN_PREFIXES.iter().any(|prefix| command.starts_with(prefix))
    }

    fn build_adapted_command(
        &self,
        command: &str,
        raw_args: &[String],
        package_manager: &str,
        original_command: &str,
    ) -> AdaptedCommand {
        let (safe, args, rejected) = sanitize_args(raw_args);
        if !safe {
            return blocked_command(
                original_command,
                &format!("Argumento contém metacaracteres de shell: {}", rejected.unwrap_or_default()),
            );
        }
        AdaptedCommand {
            command: command.to_string(),
            args,
            package_manager: package_manager.to_string(),
            original_command: original_command.to_string(),
            blocked: false,
            block_reason: None,
        }
    }

    fn adapt_bun_to_yarn(&self, bun_command: &str, command: &str) -> AdaptedCommand {
        let mappings: std::collections::HashMap<&str, (&str, Vec<&str>)> = [
            ("install", ("yarn", vec!["install"])),
            ("add", ("yarn", vec!["add"])),
            ("add -d", ("yarn", vec!["add", "-D"])),
            ("run", ("yarn", vec!["run"])),
            ("dev", ("yarn", vec!["dev"])),
            ("build", ("yarn", vec!["build"])),
            ("test", ("yarn", vec!["test"])),
            ("lint", ("yarn", vec!["lint"])),
            ("type-check", ("yarn", vec!["type-check"])),
            ("tsc --noEmit", ("yarn", vec!["tsc", "--noEmit"])),
            ("generate:component", ("yarn", vec!["generate:component"])),
        ].iter().cloned().collect();

        if let Some((cmd, args)) = mappings.get(command) {
            let args_owned: Vec<String> = args.iter().map(|&s| s.to_string()).collect();
            return self.build_adapted_command(cmd, &args_owned, "yarn", bun_command);
        }

        if let Some(script_name) = command.strip_prefix("run ") {
            return self.build_adapted_command(
                "yarn",
                &["run".to_string(), script_name.to_string()],
                "yarn",
                bun_command,
            );
        }

        if let Some(packages_str) = command.strip_prefix("add -d ") {
            let packages: Vec<String> = packages_str.split_whitespace().map(|s| s.to_string()).collect();
            let mut args = vec!["add".to_string(), "-D".to_string()];
            args.extend(packages);
            return self.build_adapted_command("yarn", &args, "yarn", bun_command);
        }

        if let Some(packages_str) = command.strip_prefix("add ") {
            let packages: Vec<String> = packages_str.split_whitespace().map(|s| s.to_string()).collect();
            let mut args = vec!["add".to_string()];
            args.extend(packages);
            return self.build_adapted_command("yarn", &args, "yarn", bun_command);
        }

        // Fallback
        let fallback_args: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
        self.build_adapted_command("yarn", &fallback_args, "yarn", bun_command)
    }

    fn adapt_bun_to_npm(&self, bun_command: &str, command: &str) -> AdaptedCommand {
        let mappings: std::collections::HashMap<&str, (&str, Vec<&str>)> = [
            ("install", ("npm", vec!["install"])),
            ("add", ("npm", vec!["install"])),
            ("add -d", ("npm", vec!["install", "--save-dev"])),
            ("run", ("npm", vec!["run"])),
            ("dev", ("npm", vec!["run", "dev"])),
            ("build", ("npm", vec!["run", "build"])),
            ("test", ("npm", vec!["test"])),
            ("lint", ("npm", vec!["run", "lint"])),
            ("type-check", ("npm", vec!["run", "type-check"])),
            ("tsc --noEmit", ("npm", vec!["run", "tsc", "--noEmit"])),
            ("generate:component", ("npm", vec!["run", "generate:component"])),
        ].iter().cloned().collect();

        if let Some((cmd, args)) = mappings.get(command) {
            let args_owned: Vec<String> = args.iter().map(|&s| s.to_string()).collect();
            return self.build_adapted_command(cmd, &args_owned, "npm", bun_command);
        }

        if let Some(script_name) = command.strip_prefix("run ") {
            return self.build_adapted_command(
                "npm",
                &["run".to_string(), script_name.to_string()],
                "npm",
                bun_command,
            );
        }

        if let Some(packages_str) = command.strip_prefix("add -d ") {
            let packages: Vec<String> = packages_str.split_whitespace().map(|s| s.to_string()).collect();
            let mut args = vec!["install".to_string(), "--save-dev".to_string()];
            args.extend(packages);
            return self.build_adapted_command("npm", &args, "npm", bun_command);
        }

        if let Some(packages_str) = command.strip_prefix("add ") {
            let packages: Vec<String> = packages_str.split_whitespace().map(|s| s.to_string()).collect();
            let mut args = vec!["install".to_string()];
            args.extend(packages);
            return self.build_adapted_command("npm", &args, "npm", bun_command);
        }

        let fallback_args: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
        self.build_adapted_command("npm", &fallback_args, "npm", bun_command)
    }

    fn adapt_bun_to_pnpm(&self, bun_command: &str, command: &str) -> AdaptedCommand {
        let mappings: std::collections::HashMap<&str, (&str, Vec<&str>)> = [
            ("install", ("pnpm", vec!["install"])),
            ("add", ("pnpm", vec!["add"])),
            ("add -d", ("pnpm", vec!["add", "-D"])),
            ("run", ("pnpm", vec!["run"])),
            ("dev", ("pnpm", vec!["dev"])),
            ("build", ("pnpm", vec!["build"])),
            ("test", ("pnpm", vec!["test"])),
            ("lint", ("pnpm", vec!["lint"])),
            ("type-check", ("pnpm", vec!["type-check"])),
            ("tsc --noEmit", ("pnpm", vec!["tsc", "--noEmit"])),
            ("generate:component", ("pnpm", vec!["generate:component"])),
        ].iter().cloned().collect();

        if let Some((cmd, args)) = mappings.get(command) {
            let args_owned: Vec<String> = args.iter().map(|&s| s.to_string()).collect();
            return self.build_adapted_command(cmd, &args_owned, "pnpm", bun_command);
        }

        if let Some(script_name) = command.strip_prefix("run ") {
            return self.build_adapted_command(
                "pnpm",
                &["run".to_string(), script_name.to_string()],
                "pnpm",
                bun_command,
            );
        }

        if let Some(packages_str) = command.strip_prefix("add -d ") {
            let packages: Vec<String> = packages_str.split_whitespace().map(|s| s.to_string()).collect();
            let mut args = vec!["add".to_string(), "-D".to_string()];
            args.extend(packages);
            return self.build_adapted_command("pnpm", &args, "pnpm", bun_command);
        }

        if let Some(packages_str) = command.strip_prefix("add ") {
            let packages: Vec<String> = packages_str.split_whitespace().map(|s| s.to_string()).collect();
            let mut args = vec!["add".to_string()];
            args.extend(packages);
            return self.build_adapted_command("pnpm", &args, "pnpm", bun_command);
        }

        let fallback_args: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
        self.build_adapted_command("pnpm", &fallback_args, "pnpm", bun_command)
    }

    pub fn adapt_commands(&self, bun_commands: &[String]) -> Vec<AdaptedCommand> {
        bun_commands.iter().map(|cmd| self.adapt_command(cmd)).collect()
    }

    pub fn needs_adaptation(&self, command: &str) -> bool {
        command.starts_with("bun") && self.host_package_manager != "bun"
    }

    pub fn format_original_command(adapted_command: &AdaptedCommand) -> String {
        if adapted_command.blocked {
            format!("[BLOCKED: {}]", adapted_command.block_reason.as_ref().unwrap_or(&"Unknown".to_string()))
        } else {
            format!("{} {}", adapted_command.package_manager, adapted_command.args.join(" "))
        }
    }
}
