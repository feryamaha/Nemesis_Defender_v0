---
trigger: always_on
status: active
scope: repo-local
repo: Nemesis_Defender_v0
last_updated: 2026-07-09
---

# Perfil do repo: Nemesis_Defender_v0 (motor)

> Regra per-repo POR DESIGN (não espelhada; ver manifest em `nemesis-harness-integrity.md`).
> As skills e workflows espelhados são texto ÚNICO e leem deste arquivo tudo o que depende de
> stack: comandos de validação, paths, regras de linguagem e fases exclusivas. O repo irmão
> tem o perfil próprio dele em `../Dashboard-Nemesis-Defender/.devin/rules/nemesis-repo-profile.md`.

## Identidade

- **Papel:** motor de enforcement (runtime, pretool, daemon, eBPF, pentest, publisher).
- **Stack:** Rust (workspace Cargo em `.nemesis/`), C do eBPF (`ebpf-kernel/`, infra
  pré-existente), shell scripts herdados (`install/`, `scripts/`, `pentest-nemesis-control/`).

## Comandos de validação (por fase)

| Fase | Comando | Observação |
|---|---|---|
| Verificação por tarefa | `cd .nemesis && cargo check -p <crate>` | uma por tarefa do plano |
| Check do workspace | `cd .nemesis && cargo check --workspace` | |
| Testes unitários | `cd .nemesis && cargo test -p nemesis-defender` | `nemesis-ebpf-kernel` exige `--release` |
| Build release | `cd .nemesis && cargo build --release --workspace` | só com autorização (intrínseca na Skill 4.5) |
| Validação de segurança estática | `bash .nemesis/pentest-nemesis-control/nemesis-defender/run-pentest.sh` | contra binário release; `FAIL=0` + `STATUS: APROVADO` |
| Capabilities eBPF (Linux) | `sudo .nemesis/scripts/ensure-ebpf-caps.sh` | após todo build release (setcap é por-inode) |
| Diagnóstico | `.nemesis/target/release/nemesis-doctor` | ANTES de reconectar o pretool |
| Postura de proteção (pre-flight F1) | `.nemesis/target/release/nemesis-doctor --quick` | G4 pretool · G5 eBPF · G6 daemon |

## Fases exclusivas deste perfil (não existem no dashboard)

As Fases 3, 5, 6 e 7 da skill `nemesis-tests` (pentest estático, capabilities eBPF,
nemesis-doctor, reconexão do pretool + pentest full) aplicam-se APENAS a este repo.

## Paths de processo

- Specs: `Feature-Documentation/SPECS/SPEC_NNN_nome.md`
- Plans: `Feature-Documentation/PLANS/PLAN_NNN_nome.md`
- PRs: `Feature-Documentation/PR/PR_NNN_nome.md`
- Issues: `Feature-Documentation/ISSUE/`
- Trust Ledger: `.devin/ledger/trust-ledger.md`

## Regras de linguagem (as 6 regras do rule-control, versão deste perfil)

1. **Rust como única linguagem NOVA em `.nemesis/`.** Proibido introduzir código novo
   .ts/.js/.py/.sh dentro de `.nemesis/`. Permitido (herdar, não introduzir toolchain novo):
   EDITAR infra pré-existente não-Rust quando a mudança exigir (C do eBPF, shell herdado de
   `install/`, `scripts/`, `pentest-nemesis-control/`). Config e templates (.json, .toml,
   .service, .plist) onde o design os prevê. Arquivo não-Rust NOVO fora dessas categorias = FAIL.
2. **Build via Cargo workspace.** `cargo check -p <crate>` por tarefa; `cargo test -p <crate>`
   para validação. Proibido rustc direto e `cargo build --release` sem autorização.
3. **Hooks exigem manutenção coordenada.** Tarefa que toca `.nemesis/hooks/` requer flag
   `maintenance_mode_required` (quem desconecta o pretool é o Fernando, invariante 12).
4. **Escopo da spec.** Nenhuma tarefa modifica arquivo fora de FILES INVOLVED.
5. **Git é exclusivamente do Fernando.** Nenhuma tarefa executa git de escrita; exceção
   única: relatórios em `Feature-Documentation/` (sem git).
6. **Sem binários fora de `.nemesis/target/`.** Nenhuma cópia de binário para outro path.

## Guardas específicas do perfil

- Nunca subir o daemon durante install/manutenção; validação pontual só com
  `nemesis-defender --scan` (AGENTS.md invariante 3).
- Resolução de caminho pelo ancestral `.nemesis` (nunca profundidade fixa).
- Nada de `unsafe` novo no Rust de userspace; o `unsafe` legítimo vive no C do eBPF.
- Script shell NÃO pode manipular paths do harness em variável (o Defender quarentena por
  design; ver origem em `nemesis-harness-integrity.md`).
