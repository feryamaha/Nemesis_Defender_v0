# Ledger do módulo: scripts

> Append-only. Canon: `.devin/rules/nemesis-global-defender.md` §4. Preenchido pela doc-sync (4.6).

**Módulo:** shell herdado — `ensure-ebpf-caps.sh` (setcap por-inode após build release),
`nemesis-build.sh`.
**Path:** `.nemesis/scripts/`
**Camada:** — · **Jóia:** não
**Guardas:** script shell **não pode manipular path do harness em variável** (o Defender quarentena por design, visitor `nemesis_bypass`); herdar, não introduzir toolchain novo.

## Histórico

| Data | Ciclo/PR | Mudança | Veredito | Problema / próxima melhoria |
|---|---|---|---|---|
| 2026-07-22 | — | Ledger criado (bootstrap do harness agêntico) | — | — |
