---
name: nemesis-finishing-branch
description: >
  Finaliza o desenvolvimento: verifica testes, coleta git diff real, gera PR estruturada,
  apresenta opcoes de merge/PR/descarte. HARD-GATE de revisao humana antes de salvar a PR.
---

# Nemesis Finishing Branch (Rust)

Verificar conclusao, coletar evidencias de git diff real, gerar documentacao de PR estruturada,
apresentar opcoes de disposicao da branch.

**Anuncio de inicio**: "Estou usando a skill nemesis-finishing-branch para finalizar o desenvolvimento."

**Pre-requisito**: Todas as tarefas de implementacao completadas e verificadas.

## Processo

### Step 1: Verificar Conclusao

```bash
cd .nemesis && cargo check --workspace
cd .nemesis && cargo test --workspace
```

Ambos devem sair com exit code 0. Se qualquer um falhar:
- Retornar para `nemesis-subagent-driven-development`
- Reportar qual comando falhou e por que
- NAO prosseguir

### Step 2: Coletar Evidencias Reais

**IMPORTANTE**: Usar APENAS evidencias reais de git diff. NUNCA fabricar.

```bash
# Branch atual
git branch --show-current

# Mudancas por arquivo
git diff --stat

# Lista exata de arquivos modificados
git diff HEAD --name-only

# Log recente
git log --oneline -5

# Diff de conteudo (se necessario detalhe)
git diff HEAD
```

### Step 3: Gerar Conteudo da PR

Construir PR a partir de dados reais (git diff, stat, log):

```markdown
# PR_NNN: [Titulo Descritivo da Entrega]

## Objetivo
[1-4 linhas: o que foi feito e por que. Referencia a spec/plano.]

## Arquivos Afetados
- `.nemesis/crate/src/path/to/file.rs` [new|modified]
- `.nemesis/crate/src/another/file.rs` [modified]

(Usar output EXATO de `git diff --stat`)

## Implementacoes Realizadas

### Arquivo: `.nemesis/crate/src/path/to/file.rs` (new|modified)
[O que foi criado/modificado em detalhe. Decisao tecnica. Padrão Rust seguido.]

[Repetir para cada arquivo significativo]

## Criterios de Aceitacao
- [x] cargo check --workspace: PASS
- [x] cargo test --workspace: PASS
- [x] Sem violacoes Nemesis
- [x] Codigo Rust idiomatico

## Beneficios
[Reuso, desacoplamento, seguranca, performance, enforcebilidade]

## Notas Adicionais
[Contexto adicional se relevante]
```

### Step 4: Apresentar PR para Review

**HARD-GATE**: Apresentar PR completa. BLOQUEAR ate resposta de Fernando.

Tabela de validacao:
| Comando | Resultado | Observacoes |
|---------|-----------|-------------|
| `cargo check --workspace` | PASS | Compilacao Rust valida |
| `cargo test --workspace` | PASS | Testes passam |
| `git diff --stat` | [Numero de files] | Escopo correto |

Perguntar:
```
PR gerada com base em git diff real. Revise acima. Aprovado?
```

Respostas validas: "sim", "pode", "aprovado", "ok", "prossiga"

### Step 5: Salvar PR

Apos aprovacao, salvar em:
```
Feature-Documentation/PR/PR_NNN_nome-descritivo.md
Numero: auto-increment a partir de PRs existentes (001, 002, ...)
```

Arquivo markdown contendo PR completa aprovada.

### Step 6: Apresentar Opcoes de Disposicao

**FERNANDO DECIDE** o que fazer com a branch:

```
OPCOES DE DISPOSICAO:

1. MERGE NA BRANCH PRINCIPAL
   Fernando executa: git merge <branch-name>

2. MANTER BRANCH PARA PR REVIEW EXTERNO
   Branch fica aberta para pull request externo
   
3. DESCARTAR BRANCH
   Fernando executa: git branch -D <branch-name>

Aguardando decisao de Fernando...
```

## Red Flags

- `cargo check --workspace` NAO PASS
- `cargo test --workspace` NAO PASS
- Arquivos modificados fora do scope original
- Violacoes de regras Nemesis detectadas
- git diff contem arquivos secretos (.env, credentials, etc)

## Lembrar

- NUNCA fabricar git diff — usar dados reais SOMENTE
- NUNCA pular pipeline de validacao
- PR segue conVencoes deste projeto (PT-BR, paths exatos, evidencias reais)
- Nemesis enforcement ja validou qualidade de codigo
- Fernando faz commit/push manualmente — skill NAO faz git write

## Integracao

**Skill anterior**: `nemesis-subagent-driven-development`
**Final skill no pipeline Nemesis SDD**