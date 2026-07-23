# Ledger do módulo: denylist

> Append-only. Canon: `.devin/rules/nemesis-global-defender.md` §4. Preenchido pela doc-sync (4.6).

**Módulo:** denylists de **comando EDITÁVEIS pelo usuário** (`deny-list.json`, `deny-list-base.json`,
`deny-list-generic.json`, `deny-list-quality.json`, `denylist-folder-files.json`). Isentas do scan do daemon.
**Path:** `.nemesis/denylist/`
**Camada:** 1 · **Jóia:** não
**Guardas:** NÃO confundir com as regras EMBUTIDAS (`denylist-defender.json`, em `nemesis-defender/config/`); estas ficam no disco por design e o usuário pode ajustar.

## Histórico

| Data | Ciclo/PR | Mudança | Veredito | Problema / próxima melhoria |
|---|---|---|---|---|
| 2026-07-22 | — | Ledger criado (bootstrap do harness agêntico) | — | — |
