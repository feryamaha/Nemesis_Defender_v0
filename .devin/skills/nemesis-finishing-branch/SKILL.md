---
name: nemesis-finishing-branch
description: >
  Finaliza o desenvolvimento: verifica testes, coleta git diff real, gera PR estruturada,
  apresenta opcoes de merge/PR/descarte. HARD-GATE de revisao humana antes de salvar a PR.
---

# Nemesis Finishing Branch (Rust)

Verificar conclusao, coletar evidencias de git diff real, gerar documentacao de PR estruturada,
apresentar opcoes de disposicao da branch.

## AUTORIZACAO OBRIGATORIA (nunca auto-invocar)

Esta skill **NAO faz parte da fase autonoma do pipeline**. Ela so executa mediante
autorizacao explicita do Fernando, dada na PARADA UNICA (pos-validacao) ou depois dela.
Ficar "tudo verde" na validacao NAO autoriza o finishing. Invocar sem autorizacao explicita =
violacao do pipeline.

**Anuncio de inicio**: "Estou usando a skill nemesis-finishing-branch para finalizar o desenvolvimento."

**Pre-requisitos**: (1) todas as tarefas de implementacao completadas e validadas (Skill 4.5);
(2) autorizacao explicita do Fernando para esta skill.

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

Construir PR a partir de dados reais (git diff, stat, log). A PR DEVE conter todas as secoes
abaixo, na ordem exata, incluindo a **CLI Table** com comandos, resultados e observacoes:

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

## CLI Table

> **IMPORTANTE**: A CLI Table contem APENAS comandos de VALIDACAO do projeto (cargo check,
> cargo test, pentest, build, etc). NUNCA inclua comandos de analise como `git diff`,
> `git log`, `git status` ou similares. Esses sao ferramentas internas para coletar evidencias,
> nao validacoes do projeto.

| Command | Result (OK/FAIL) | Observations |
|---------|------------------|--------------|
| `cargo check --workspace` | OK | Compilacao Rust valida |
| `cargo test --workspace` | OK | Testes passam |
| Pentest estatico | OK | 224/224 PASS (100%) |
| Pentest full live | OK | 74/74, 0 gaps, AUTOSSUFICIENTE |
```

### Exemplo de PR Preenchida (stub de referencia)

```markdown
# Adicao do pacote widget-accessibility

## Objetivo
Corrigir erro de modulo nao encontrado no componente AccessibilityWidget adicionando a dependencia widget-accessibility que quebrou o deploy apos a ultima PR 105.

## Arquivos Afetados
- `package.json` [modificado]
- `bun.lock` [modificado]
- `next-env.d.ts` [modificado]

## Melhorias Implementadas
### Dependencias
- `package.json`: Adicionado widget-accessibility@^2.0.1 as dependencias do projeto
- `bun.lock`: Atualizado lockfile com a nova dependencia
- `next-env.d.ts`: Regenerado apos build

## Beneficios
- **Correcao de bug**: Resolve o erro "Module not found: Can't resolve 'widget-accessibility'" que impedia o build
- **Funcionalidade**: O componente AccessibilityWidget agora pode carregar o modulo de acessibilidade corretamente

## CLI Table

| Command | Result (OK/FAIL) | Observations |
|---------|------------------|--------------|
| bun build | OK | Build success |
| Pentest estatico | OK | 224/224 PASS (100%) |
| Pentest full live | OK | 74/74, 0 gaps, AUTOSSUFICIENTE |
```

> **IMPORTANTE**: O stub acima e um EXEMPLO de como a PR deve ficar preenchida. Substitua todos
> os valores pelos dados reais do seu git diff. NAO copie o stub literalmente. Siga a estrutura:
> titulo com colon, secoes na ordem exata, um `### Arquivo:` por arquivo significativo, output
> exato de `git diff --stat` em bloco de codigo, criterios de aceitacao com checkboxes marcados,
> e **sempre** inclua a **CLI Table** com comandos, resultados (OK/FAIL) e observacoes.

### Step 4: Apresentar PR para Review

**HARD-GATE**: Apresentar PR completa (incluindo CLI Table). BLOQUEAR ate resposta de Fernando.

Perguntar:
```
PR gerada com base em git diff real. Revise acima. Aprovado?
```

Respostas validas: "sim", "pode", "aprovado", "ok", "prossiga"

### Step 5: Salvar PR

Apos aprovacao, salvar em:
```
Feature-Documentation/PR/PR_NNN_nome-descritivo.md
```

**Antes de gravar**: listar o conteudo de `Feature-Documentation/PR/` para determinar o
proximo numero sequencial. NUNCA assumir um numero sem verificar. NUNCA criar numero
repetido ou errado. O numero deve ser exatamente o proximo na sequencia das PRs existentes.

Exemplo:
```bash
ls Feature-Documentation/PR/
```
Se existem PR_001 e PR_002, a proxima sera PR_003. Se a pasta esta vazia, a primeira sera PR_001.

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