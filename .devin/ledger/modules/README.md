# Ledgers por módulo do motor

Um ledger **append-only por módulo** para rastreabilidade de manutenção, melhoria e problema ao longo
do tempo (lei F11, o análogo por módulo do Trust Ledger). Aqui vive o **HISTÓRICO**; o **canon** de
cada módulo (o que é/faz, do/don't) vive em `.devin/rules/nemesis-global-defender.md` §4.

- **Quem preenche:** a `nemesis-doc-sync` (Skill 4.6), quando um ciclo do SDD pipeline toca o módulo.
  O roteador de módulo do pré-flight (ver `nemesis-sdd-pipeline-auto.md`) identifica o módulo tocado.
- **Escopo:** engine-only (perfil motor). **Não espelhado** para a dashboard: é `.devin/ledger/`,
  dados locais de processo (ver manifest em `nemesis-harness-integrity.md`).
- **Append-only:** nunca reescrever histórico; só anexar linha. Números copiados da saída literal.
- **Formato:** cabeçalho do módulo (path, camada, jóia) + tabela de histórico.

## Índice (um arquivo por módulo do §4 do canon)

`hooks` · `nemesis-defender` · `ebpf-kernel` · `nemesis-doctor` · `nemesis-publisher` ·
`ast-linters` · `denylist` · `denylist-customers` · `quarantine` · `forensics` · `install` ·
`scripts` · `lsp` · `runtime` · `telemetry` · `target` · `logs` · `pentest-nemesis-control`

Módulos-jóia (elevados à camada MAIOR pelo roteador): **`hooks`** e **`pentest-nemesis-control`**.
