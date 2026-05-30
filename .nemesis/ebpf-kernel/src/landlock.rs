use landlock::{
    ABI, AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr,
    RulesetStatus,
};

#[derive(Debug)]
pub enum LandlockStatus {
    Active { abi: u32 },
    Unsupported,
    Error(String),
}

/// Verifica se o kernel atual suporta Landlock (>= 5.13).
pub fn landlock_in_lsm_list() -> bool {
    std::fs::read_to_string("/sys/kernel/security/lsm")
        .map(|s| s.split(',').any(|l| l.trim() == "landlock"))
        .unwrap_or(false)
}

/// Aplica um ruleset Landlock ao processo atual e a toda a sua árvore de filhos.
/// Não requer root — usa no_new_privs via prctl antes de restrict_self.
/// `allowed_exec_paths`: paths que o processo sandboxed pode executar.
pub fn apply_sandbox(allowed_exec_paths: &[&str]) -> LandlockStatus {
    // no_new_privs: obrigatório para landlock_restrict_self sem CAP_SYS_ADMIN
    let r = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if r != 0 {
        return LandlockStatus::Error("prctl PR_SET_NO_NEW_PRIVS failed".into());
    }

    // ABI V5 — kernel 6.8; degradação automática em kernels mais antigos
    let _abi = ABI::V5;

    let ruleset_result = Ruleset::default()
        .handle_access(AccessFs::Execute)
        .and_then(|r| r.handle_access(AccessFs::WriteFile))
        .and_then(|r| r.handle_access(AccessFs::ReadFile))
        .and_then(|r| r.create());

    let mut ruleset = match ruleset_result {
        Ok(r) => r,
        Err(e) => return LandlockStatus::Error(format!("ruleset create: {e}")),
    };

    for path in allowed_exec_paths {
        let fd = match PathFd::new(path) {
            Ok(fd) => fd,
            Err(_) => continue, // path inexistente — skip silencioso
        };
        ruleset = match ruleset.add_rule(PathBeneath::new(fd, AccessFs::Execute)) {
            Ok(r) => r,
            Err(e) => return LandlockStatus::Error(format!("add_rule {path}: {e}")),
        };
    }

    match ruleset.restrict_self() {
        Ok(status) => match status.ruleset {
            RulesetStatus::FullyEnforced => LandlockStatus::Active { abi: 5 },
            RulesetStatus::PartiallyEnforced => LandlockStatus::Active { abi: 1 },
            RulesetStatus::NotEnforced => LandlockStatus::Unsupported,
        },
        Err(e) => LandlockStatus::Error(format!("restrict_self: {e}")),
    }
}
