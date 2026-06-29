## Objetivo
[1-4 linhas: o que foi feito e por que. Referencia a spec/plano.]

## Tipo de mudanca
- [ ] Correcao de bug
- [ ] Nova feature
- [ ] Teste de regressao (cobrindo um bug/bypass ja corrigido)
- [ ] Documentacao
- [ ] Refatoracao / performance

## Arquivos Afetados
- `caminho/do/arquivo.rs` [new|modified]
- `caminho/do/outro/arquivo.rs` [modified]

## Implementacoes Realizadas

### Arquivo: `caminho/do/arquivo.rs` (new|modified)
- [O que foi criado/modificado em detalhe. Decisao tecnica. Padrao Rust seguido.]

### Arquivo: `caminho/do/outro/arquivo.rs` (modified)
- [O que foi modificado. Decisao tecnica.]

## Criterios de Aceitacao
- [x] `cargo check --workspace`: PASS
- [x] `cargo test --workspace`: PASS
- [x] Sem violacoes Nemesis
- [x] Codigo Rust idiomatico

## Beneficios
[Reuso, desacoplamento, seguranca, performance, enforcebilidade]

## Notas Adicionais
[Contexto adicional se relevante]

> ⚠️ Se este PR está relacionado a uma **falha de segurança ou bypass explorável**, pare. Não descreva o exploit aqui — siga o [SECURITY.md](SECURITY.md) e reporte em privado primeiro.

## CLI Table

| Command | Result (OK/FAIL) | Observations |
|---------|------------------|--------------|
| `cargo check --workspace` | OK | Compilação válida |
| `cargo test --workspace` | OK | Testes passam |
| Pentest estático | OK | 224/224 PASS (100%) |
| Pentest full live | OK | 74/74, 0 gaps, AUTOSSUFICIENTE |

## DCO
Ao abrir este PR, confirmo que minha contribuição será licenciada sob a **GNU AGPL v3.0** (e concedo ao mantenedor o direito de licenciamento dual, conforme o CONTRIBUTING.md) e certifico a origem do código conforme o **Developer Certificate of Origin**.