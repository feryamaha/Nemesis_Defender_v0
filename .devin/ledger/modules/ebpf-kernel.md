# Ledger do módulo: ebpf-kernel

> Append-only. Canon: `.devin/rules/nemesis-global-defender.md` §4. Preenchido pela doc-sync (4.6).

**Módulo:** backstop de kernel (Linux, opt-in). `bprm_check_security` → `-EPERM` no exec bloqueado;
allowlist de egress em `socket_connect`; escopo por cgroup do agente; Landlock sem root
(`no_new_privs`). Independente do pretool.
**Path:** `.nemesis/ebpf-kernel/` (crate) + `ebpf/nemesis-block.bpf.c` + `denylist-ebpf/commands.toml`
**Camada:** 3 · **Jóia:** não
**Guardas:** `unsafe` legítimo vive no C do eBPF, não no Rust de userspace; `cargo test` exige `--release`; nunca sobrepor decisão anterior da cadeia LSM (`ret != 0`); não commitar `.bpf.o`.

## Histórico

| Data | Ciclo/PR | Mudança | Veredito | Problema / próxima melhoria |
|---|---|---|---|---|
| 2026-07-22 | — | Ledger criado (bootstrap do harness agêntico) | — | — |
