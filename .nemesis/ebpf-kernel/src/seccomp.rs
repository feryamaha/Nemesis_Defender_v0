use libc;
use seccompiler::{BpfProgram, SeccompAction, SeccompFilter, SeccompRule};
use std::collections::BTreeMap;

/// Syscall numbers for x86_64 (Linux)
/// Used for newer syscalls that may not be available in all libc versions
#[allow(non_snake_case)]
mod SYS {
    pub const OPEN_TREE: i64 = 428;
    pub const MOVE_MOUNT: i64 = 429;
    pub const FSOPEN: i64 = 430;
    pub const FSCONFIG: i64 = 431;
    pub const FSMOUNT: i64 = 432;
    pub const FSPICK: i64 = 433;
    pub const MOUNT_SETATTR: i64 = 442;
    pub const USERFAULTFD: i64 = 323;
    pub const BPF: i64 = 321;
    pub const PERF_EVENT_OPEN: i64 = 298;
    pub const IO_URING_SETUP: i64 = 425;
    pub const IO_URING_ENTER: i64 = 426;
    pub const IO_URING_REGISTER: i64 = 427;
    pub const FINIT_MODULE: i64 = 313;
    pub const KEXEC_FILE_LOAD: i64 = 320;
    pub const REBOOT: i64 = 169;
    pub const ACCT: i64 = 163;
    pub const KCMP: i64 = 312;
    pub const LOOKUP_DCOOKIE: i64 = 212;
    pub const ADD_KEY: i64 = 248;
    pub const REQUEST_KEY: i64 = 249;
    pub const KEYCTL: i64 = 250;
}

/// Aplica filtro seccomp-bpf bloqueando syscalls perigosas.
/// Requer que no_new_privs já esteja ativo (landlock::apply_sandbox faz isso).
///
/// Bloqueados por categoria:
/// - Kernel module loading: init_module, finit_module, delete_module
/// - Kernel/system control: mount, kexec_load, kexec_file_load, reboot, acct
/// - New mount API (Linux 5.2+): open_tree, move_mount, fsopen, fsconfig, fsmount, fspick, mount_setattr
/// - Exploit primitives: userfaultfd, bpf, perf_event_open, io_uring_*
/// - Host information leak: kcmp, lookup_dcookie
/// - Keyring manipulation: add_key, request_key, keyctl
/// - Legacy: ptrace
pub fn apply_seccomp_filter() -> Result<(), String> {
    let mut rules: BTreeMap<i64, Vec<SeccompRule>> = BTreeMap::new();

    for syscall in &[
        // Kernel module loading (ring-0 execution)
        libc::SYS_init_module,
        libc::SYS_delete_module,
        SYS::FINIT_MODULE,

        // Kernel/system control
        libc::SYS_mount,
        libc::SYS_kexec_load,
        SYS::KEXEC_FILE_LOAD,
        SYS::REBOOT,
        SYS::ACCT,

        // New mount API (Linux 5.2+ — bypassa mount() clássico!)
        SYS::OPEN_TREE,
        SYS::MOVE_MOUNT,
        SYS::FSOPEN,
        SYS::FSCONFIG,
        SYS::FSMOUNT,
        SYS::FSPICK,
        SYS::MOUNT_SETATTR,

        // Exploit primitives
        SYS::USERFAULTFD,
        SYS::BPF,
        SYS::PERF_EVENT_OPEN,
        SYS::IO_URING_SETUP,
        SYS::IO_URING_ENTER,
        SYS::IO_URING_REGISTER,

        // Host information leak
        SYS::KCMP,
        SYS::LOOKUP_DCOOKIE,

        // Keyring manipulation
        SYS::ADD_KEY,
        SYS::REQUEST_KEY,
        SYS::KEYCTL,

        // Legacy
        libc::SYS_ptrace,
    ] {
        rules.insert(*syscall, vec![]);
    }

    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(libc::EPERM as u32),
        std::env::consts::ARCH.try_into().map_err(|e| format!("{e}"))?,
    )
    .map_err(|e| format!("filter build: {e}"))?;

    let prog: BpfProgram = filter.try_into().map_err(|e| format!("compile: {e}"))?;
    seccompiler::apply_filter(&prog).map_err(|e| format!("apply: {e}"))?;

    Ok(())
}
