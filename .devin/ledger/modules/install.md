# Ledger do módulo: install

> Append-only. Canon: `.devin/rules/nemesis-global-defender.md` §4. Preenchido pela doc-sync (4.6).

**Módulo:** instalador/desinstalador — `nemesis-install.sh`, `nemesis-uninstall.sh`,
`com.nemesis.publisher.plist`, `nemesis-publisher.service`, `info-install.txt`. Isento por nome.
**Path:** `.nemesis/install/`
**Camada:** — · **Jóia:** não
**Guardas:** **NUNCA subir o daemon durante o install** (invariante 3) — ele quarentenaria o próprio instalador; validar por `--scan`, não pelo pretool; verificar checksum antes de extrair.

## Histórico

| Data | Ciclo/PR | Mudança | Veredito | Problema / próxima melhoria |
|---|---|---|---|---|
| 2026-07-22 | — | Ledger criado (bootstrap do harness agêntico) | — | — |
