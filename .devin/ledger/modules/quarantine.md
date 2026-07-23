# Ledger do módulo: quarantine

> Append-only. Canon: `.devin/rules/nemesis-global-defender.md` §4. Preenchido pela doc-sync (4.6).

**Módulo:** estado de runtime da quarentena — `PENDING.json` + arquivos movidos + `meta.json` (motivo).
O daemon MOVE (não deleta) o confirmado; enquanto houver pendência, o pretool trava a sessão.
Reversível por `restore` / `purge`.
**Path:** `.nemesis/quarantine/`
**Camada:** 2 · **Jóia:** não
**Guardas:** não editar à mão; resolver por CLI (`restore`/`purge`); não versionar conteúdo movido.

## Histórico

| Data | Ciclo/PR | Mudança | Veredito | Problema / próxima melhoria |
|---|---|---|---|---|
| 2026-07-22 | — | Ledger criado (bootstrap do harness agêntico) | — | — |
