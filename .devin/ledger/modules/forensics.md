# Ledger do módulo: forensics

> Append-only. Canon: `.devin/rules/nemesis-global-defender.md` §4. Preenchido pela doc-sync (4.6).

**Módulo:** auditoria de conteúdo externo (issue/PR) — `incoming/` + `scan-incoming.sh` → veredito
APROVADO/REPROVADO. **Isenta da quarentena do daemon** (`daemon_quarantine_exempt`): escaneia/loga
mas não move nem trava.
**Path:** `.nemesis/forensics/`
**Camada:** — (triagem) · **Jóia:** não
**Guardas:** `incoming/` e o relatório não são versionados; é triagem, não garantia — sempre ler + revisar; conteúdo de terceiros com nome de marca é não-confiável por padrão (anti-masquerading).

## Histórico

| Data | Ciclo/PR | Mudança | Veredito | Problema / próxima melhoria |
|---|---|---|---|---|
| 2026-07-22 | — | Ledger criado (bootstrap do harness agêntico) | — | — |
