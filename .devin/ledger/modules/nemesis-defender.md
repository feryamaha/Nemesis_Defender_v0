# Ledger do módulo: nemesis-defender

> Append-only. Canon: `.devin/rules/nemesis-global-defender.md` §4. Preenchido pela doc-sync (4.6).

**Módulo:** motor `scan_content` + `compute_severity` + daemon Iron Dome + CLI + quarentena. Inclui
`scanner/` (8 arquivos), `visitors/` (15 arquivos) e `config/denylist-defender.json` (regras
EMBUTIDAS via `include_str!`, fonte única).
**Path:** `.nemesis/nemesis-defender/`
**Camada:** 1+2 (core) · **Jóia:** não (crítico — trust-critical no CODEOWNERS)
**Guardas:** daemon é security-only (qualidade nunca chega nele); não duplicar regra de conteúdo fora da fonte única; sem `unwrap()` em input não-confiável; visitor é método, não cobertura.

## Histórico

| Data | Ciclo/PR | Mudança | Veredito | Problema / próxima melhoria |
|---|---|---|---|---|
| 2026-07-22 | — | Ledger criado (bootstrap do harness agêntico) | — | — |
