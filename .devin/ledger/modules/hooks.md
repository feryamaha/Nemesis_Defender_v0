# Ledger do módulo: hooks

> Append-only. Canon: `.devin/rules/nemesis-global-defender.md` §4. Preenchido pela doc-sync (4.6).

**Módulo:** hooks pretool/posttool — interceptação write/exec-time, `exit 2`, tradução
ferramenta→intenção, fail-closed. Ponto de entrada de todo o sistema.
**Path:** `.nemesis/hooks/` (bins do pacote raiz `nemesis`)
**Camada:** 1 · **Jóia:** SIM (elevado à camada MAIOR)
**Guardas:** só em manutenção coordenada pelo Fernando (invariante 12); nunca abrir caminho que "passe" em erro (quebra fail-closed).

## Histórico

| Data | Ciclo/PR | Mudança | Veredito | Problema / próxima melhoria |
|---|---|---|---|---|
| 2026-07-22 | — | Ledger criado (bootstrap do harness agêntico) | — | — |
