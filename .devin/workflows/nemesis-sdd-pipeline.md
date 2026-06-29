# Nemesis SDD Pipeline (Specification-Driven Development)

## Overview

O Nemesis SDD Pipeline e um workflow sequencial de 7 skills que governa o desenvolvimento
do Nemesis Framework v2.0 Rust de forma deterministica e auditavel.

Cada skill tem um proposito claro, um HARD-GATE de aprovacao, e integra-se ao proximo skill.

## Workflow Sequencial

```
┌─────────────────────────────────────────────────────────────┐
│ USER REQUEST (Fernando descreve necessidade informal)       │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 1: nemesis-specification-design                       │
│ OUTPUT: Especificacao tecnica gerada (nao gravada ainda)    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 0: nemesis-critical-analysis (PONTO 1: Pre-Spec)      │
│ VALIDACAO: Analise critica da spec antes de gravar          │
│ OUTPUT: PROSSEGUIR (gravar spec) ou REJEITAR (ajustar)      │
│ HARD-GATE: Veredito PROSSEGUIR para gravar spec             │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 1: nemesis-specification-design (gravacao)            │
│ OUTPUT: SPEC aprovada em Feature-Documentation/SPECS/       │
│ HARD-GATE: Fernando aprova especificacao                    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 2: pre-writing-rule-control                           │
│ VALIDACAO: Spec contra 6 regras Nemesis Rust               │
│ OUTPUT: PASS (prosseguir) ou FAIL (ajustar)                 │
│ HARD-GATE: Validacao passa ou propoe ajustes               │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 3: nemesis-writing-plans                              │
│ INPUT: SPEC aprovada + validada                            │
│ OUTPUT: PLAN com tarefas atomicas                           │
│ HARD-GATE: Fernando aprova plano                            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 0: nemesis-critical-analysis (PONTO 2: Pre-Execution) │
│ VALIDACAO: Analise critica do plano antes de executar       │
│ OUTPUT: PROSSEGUIR (executar) ou REJEITAR (ajustar)         │
│ HARD-GATE: Veredito PROSSEGUIR para executar plano          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 4: nemesis-subagent-driven-development                │
│ EXECUCAO: Tarefa por tarefa (Agent nemesis-implementer)     │
│ VALIDACAO: Two-stage review (spec compliance + code quality)│
│ OUTPUT: Todas as tarefas completadas                        │
│ EXECUCAO CONTINUA: Sem pause entre tarefas                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 4.5: nemesis-tests                                    │
│ VALIDACAO: cargo check + cargo test + run-pentest.sh        │
│ SE PASS: cargo build --release + nemesis-doctor + pentest   │
│ SE FAIL: investigar causa raiz, aprovar fix, retestar       │
│ OUTPUT: Todos os testes PASS + binarios recompilados        │
│ HARD-GATE: Fernando aprova cargo build --release            │
│ HARD-GATE: Fernando aprova reconexao do pretool             │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 5: nemesis-finishing-branch                           │
│ VALIDACAO FINAL: cargo check --workspace + cargo test       │
│ OUTPUT: PR documentada em Feature-Documentation/PR/         │
│ HARD-GATE: Fernando aprova PR                               │
│ DISPOSICAO: Fernando escolhe merge/keep/discard             │
└─────────────────────────────────────────────────────────────┘
```

## Regras Fundamentais

1. **NUNCA escrever codigo antes do design ser aprovado**
   - Skill 1 tem HARD-GATE — nao prosseguir sem aprovacao explícita
   - Skill 0 (Ponto 1) valida a spec ANTES de gravar — veredito REJEITAR bloqueia gravacao

2. **NUNCA executar antes do plano ser aprovado e validado**
   - Skill 3 tem HARD-GATE — nao disparar Skill 4 sem aprovacao
   - Skill 0 (Ponto 2) valida o plano APOS aprovacao e ANTES de executar — veredito REJEITAR bloqueia execucao

3. **Execucao continua entre tarefas** (Skill 4)
   - Nao pause para perguntar "posso continuar?"
   - PARE somente se bloqueado

4. **Usar git diff real para PRs** (Skill 5)
   - NUNCA fabricar evidencias
   - Sempre `git diff`, `git log`, dados reais

5. **Validacao obrigatoria em cada fase**
   - Skill 2: validacao contra 6 regras
   - Skill 4: two-stage review apos cada tarefa
   - Skill 4.5: cargo check + cargo test + run-pentest.sh + pentest full
   - Skill 5: cargo check + cargo test final

6. **Fernando governa decisoes humanas**
   - HARD-GATEs requerem aprovacao de Fernando
   - Skill 5 Final disposition: Fernando escolhe (merge/keep/discard)

## Entradas e Saidas

| Skill | Entrada | Saida | Aprovacao |
|-------|---------|-------|-----------|
| 0: nemesis-critical-analysis (Ponto 1) | Spec gerada + request | PROSSEGUIR/REJEITAR | (automatica) |
| 1: nemesis-specification-design | Request informal + analise critica | SPEC_NNN.md | Fernando |
| 2: pre-writing-rule-control | SPEC_NNN.md | PASS/FAIL + mensagem | (automatica) |
| 3: nemesis-writing-plans | SPEC_NNN.md (validada) | PLAN_NNN.md | Fernando |
| 0: nemesis-critical-analysis (Ponto 2) | SPEC + PLAN aprovados | PROSSEGUIR/REJEITAR | (automatica) |
| 4: nemesis-subagent-driven-development | PLAN_NNN.md (validado) | Todas tarefas ✅ | (continuo) |
| 4.5: nemesis-tests | Workspace atualizado | Testes PASS + binarios recompilados | Fernando |
| 5: nemesis-finishing-branch | Workspace validado | PR_NNN.md | Fernando |

## Como Usar

### Iniciar Pipeline

Fernando descreve necessidade:
```
"Preciso de um novo visitor tree-sitter para detectar unsafe blocks em eBPF hooks"
```

Invocar: `/nemesis-specification-design`

### Fluxo Tipico

1. **Skill 1**: Fernando descreve → Gera especificacao tecnica
2. **Skill 0 (Ponto 1)**: Analise critica da spec → PROSSEGUIR
3. **Skill 1**: Grava SPEC → Fernando aprova
4. **Skill 2**: Valida SPEC contra regras → PASS
5. **Skill 3**: Cria PLAN com tarefas → Fernando aprova
6. **Skill 0 (Ponto 2)**: Analise critica do plano → PROSSEGUIR
7. **Skill 4**: Executa TODAS as tarefas (sem pause) → COMPLETA
8. **Skill 4.5**: Testes (cargo check + cargo test + pentest) → PASS → cargo build --release → nemesis-doctor → reconectar pretool → pentest full
9. **Skill 5**: Gera PR → Fernando aprova → Choose disposition

### Parar No Meio

Se bloqueado em qualquer skill:
- STOP a execucao
- Reportar bloqueador exato a Fernando
- Aguardar instrucoes antes de continuar

## Regras do Pipeline

### Regra 1: Somente Rust (.rs files)
Nenhum .ts, .js, .py, .sh em .nemesis/

### Regra 2: Build via Cargo Workspace
Usar `cargo check -p <crate>` por tarefa. Nao rustc avulso.

### Regra 3: Maintenance Mode para Hooks
Se modificar .nemesis/hooks/, ativar maintenance mode primeiro.

### Regra 4: Scope da Spec
Nao sair do scope de arquivos listados. Nao modificar files aleatorios.

### Regra 5: Git Operations — Fernando Apenas
IA NUNCA executa git add, commit, push. Fernando faz manualmente.

### Regra 6: Sem Binarios Fora de .nemesis/target/
Nao copiar binarios para outro lugar.

## Comandos de Validacao

```bash
# Apos Skill 4 (Skill 4.5 executa estes)
cd .nemesis && cargo check --workspace
cd .nemesis && cargo test -p nemesis-defender
bash .nemesis/pentest-nemesis-control/nemesis-defender/run-pentest.sh

# Apos Skill 4.5 passar (com aprovacao)
cd .nemesis && cargo build --release --workspace
.nemesis/target/release/nemesis-doctor
# Reconectar pretool + executar pentest full

# Apos Skill 5
git diff --stat
git log --oneline -5
```

## Convencoes de Nomenclatura

- **Specs**: SPEC_NNN_nome-descritivo.md (em Feature-Documentation/SPECS/)
- **Plans**: PLAN_NNN_nome-descritivo.md (em Feature-Documentation/PLANS/)
- **PRs**: PR_NNN_nome-descritivo.md (em Feature-Documentation/PR/)
- **Numero**: auto-increment (001, 002, 003, ...)

## Permissoes Claude Code

- ✅ Read .nemesis/, Feature-Documentation/
- ✅ Write .nemesis/<crates>/, Feature-Documentation/
- ✅ Bash: cargo check, cargo test, grep, find, cat
- ❌ Write .nemesis/hooks/, .nemesis/target/
- ❌ git add, git commit, git push
- ❌ cargo build --release (requer aprovacao Fernando)

## Enforcement Ativo

Nemesis enforcement (AST + eBPF + pretool + deny-list) valida:
- Nenhum arquivo nao-.rs em .nemesis/
- Nenhum cargo build --release nao-autorizado
- Nenhum unsafe block improprio em eBPF hooks
- Nenhuma violacao de regras Nemesis

Confie nele para qualidade de codigo. Foco em fluxo e metodo.

## Suporte

Se bloqueado ou com duvidas sobre o pipeline:
1. Consultar CLAUDE.md (raiz do projeto)
2. Consultar skill especifica (.claude/skills/<skill>/SKILL.md)
3. Reportar bloqueador exato a Fernando
4. Aguardar instrucoes
