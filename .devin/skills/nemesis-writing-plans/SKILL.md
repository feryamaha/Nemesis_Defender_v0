---
name: nemesis-writing-plans
description: >
  Converte especificacao aprovada em plano de implementacao
  com tarefas atomicas (2-5 min cada). Cada tarefa tem paths exatos,
  codigo Rust completo, comandos de verificacao cargo.
---

# Nemesis Writing Plans (Rust)

Converter especificacao aprovada em plano de implementacao abrangente com tarefas atomicas.

**Anuncio de inicio**: "Estou usando a skill nemesis-writing-plans para gerar o plano de implementacao."

**Pre-requisito**: Uma especificacao aprovada existe em `Feature-Documentation/SPECS/`.

## Processo

### Step 1: Carregar e Revisar Spec

Ler a especificacao aprovada. Identificar:
- O que deve ser construido (REQUEST/REQUIREMENTS)
- Qual(is) crate(s) serao afetado(s) (FILES INVOLVED)
- Quais restricoes se aplicam (RESTRICTIONS)
- Quais sao os criterios de aceitacao (EXPECTED DELIVERY)

```bash
cat Feature-Documentation/SPECS/SPEC_*.md | tail -1
```

### Step 2: Ler Codigo Fonte Obrigatorio

**OBRIGATORIO**: Ler TODOS os arquivos listados na secao FILES INVOLVED da spec.
Nao gerar plano sem ter lido o codigo real.

```bash
# Ler Cargo.toml do(s) crate(s)
cat .nemesis/<crate>/Cargo.toml

# Ler lib.rs/main.rs
cat .nemesis/<crate>/src/lib.rs
cat .nemesis/<crate>/src/main.rs

# Ler arquivos existentes a serem modificados
cat .nemesis/<crate>/src/path/to/existing/file.rs
```

### Step 3: Mapear Estrutura de Arquivos

Antes de definir tarefas, confirmar quais arquivos serao criados ou modificados:

```
CREATE: .nemesis/crate/src/path/to/new_file.rs
MODIFY: .nemesis/crate/src/path/to/existing.rs
TEST:   .nemesis/crate/tests/path/to/test.rs
```

Cada arquivo tem uma responsabilidade clara. Arquivos que mudam juntos vivem juntos.

### Step 4: Decompor em Tarefas Atomicas

**Cada tarefa = 1 arquivo, 1 mudanca, 1 verificacao. Tempo: 2-5 minutos.**

Granularidade:
- Ler arquivo existente = um passo
- Analisar linha de mudanca = um passo
- Implementar mudanca em .rs = um passo
- Executar `cargo check -p <crate>` = um passo

**Exemplo bom**:
```
TASK 1: Adicionar nova funcao validar_unsafe_block
  FILE: .nemesis/ast-linters/src/visitors/ebpf_checker.rs (MODIFY)
  VERIFICACAO: cargo check -p ast-linters

TASK 2: Integrar visitor novo em lib.rs
  FILE: .nemesis/ast-linters/src/lib.rs (MODIFY)
  VERIFICACAO: cargo check -p ast-linters

TASK 3: Escrever testes
  FILE: .nemesis/ast-linters/tests/ebpf_checker_test.rs (CREATE)
  VERIFICACAO: cargo test -p ast-linters
```

**Exemplo ruim**:
```
TASK 1: "Implementar tudo no novo visitor"  ← Muito grande
TASK 2: "TBD — adicionar testes depois"     ← Placeholder
TASK 3: "Similar a TASK 1"                  ← Referencia indireta
```

### Step 5: Escrever o Plano

**Header obrigatorio**:

```markdown
# [Nome da Feature] — Plano de Implementacao

> **Para agentes**: Use nemesis-subagent-driven-development para executar este plano.

**Objetivo**: [Uma sentenca clara]

**Spec**: Feature-Documentation/SPECS/SPEC_NNN_nome.md

**Crates Afetadas**: ast-linters, workflow-enforcement, (lista)

**Arquitetura**: [2-3 sentencas sobre abordagem tecnica]

**Tech Stack Rust**: tree-sitter, serde, regex, (dependencias relevantes)

---
```

**Estrutura de tarefa (obrigatoria para TODA tarefa)**:

```markdown
## TASK N: [Descricao curta]

**Crate**: ast-linters

**Arquivos**:
- CREATE: `.nemesis/ast-linters/src/visitors/novo_visitor.rs`
- MODIFY: `.nemesis/ast-linters/src/lib.rs` (linhas XXX-YYY)
- TEST:   `.nemesis/ast-linters/tests/novo_visitor_test.rs`

**Verificacao**:
```bash
cd .nemesis && cargo check -p ast-linters
```

**Descricao Detalhada**:
[O que fazer, contexto tecnico, padroes Rust a seguir]

**Implementacao**:
[Codigo Rust completo a ser escrito — NAO deixar placeholders]
```

### Step 6: Sem Placeholders

NUNCA escrever:
- "TBD", "TODO", "implementar depois", "fill in details"
- "Adicionar error handling apropriado" sem codigo real
- "Escrever testes para o acima" sem codigo de teste real
- "Similar a TASK N" — repetir o codigo, tarefas podem ser lidas fora de ordem

Cada tarefa deve ter:
- Codigo Rust **completo** e **exato**
- Comando de verificacao **exato** (com expected output)
- Assumpcoes **documentadas**

### Step 7: Auto-Review (checklist obrigatorio)

Checklist interno (reprovou em algum item = corrigir o plano antes de seguir):
- [ ] Todos os paths sao exatos e confirmados no disco? (.nemesis/crate/src/...)
- [ ] Codigo Rust completo em cada tarefa? (sem placeholders)
- [ ] Cada tarefa tem cargo check/test para verificacao?
- [ ] Ordem faz sentido? (tipos antes de funcoes, lib.rs antes de main.rs, testes por ultimo;
      a suposicao mais arriscada do plano e verificada nas PRIMEIRAS tarefas, nao nas ultimas)
- [ ] Nenhuma tarefa executa git write operations?
- [ ] Verificacao final inclui cargo check --workspace + cargo test --workspace?

**MODO AUTONOMO (default)**: nao ha aprovacao humana aqui. O gate do plano e a
`nemesis-critical-analysis` (Ponto 2), invocada apos a gravacao (Step 8). Veredito
PROSSEGUIR = disparar a execucao sem pausa; REJEITAR = ajustar e re-analisar (1 ciclo);
segundo REJEITAR = parada de emergencia.

**MODO SUPERVISIONADO** (so quando o Fernando pedir): apresentar o plano e bloquear ate
aprovacao explicita ("sim", "pode", "aprovado", "ok", "prossiga", "execute").

### Step 8: Salvar Plano

Salvar em:
```
Feature-Documentation/PLANS/PLAN_NNN_nome-descritivo.md
Numero: auto-increment a partir de planos existentes (001, 002, ...)
```

## Lembrar

- Ler codigo ANTES de gerar plano — obrigatorio
- Paths exatos SEMPRE (.nemesis/crate/src/file.rs)
- Codigo Rust completo em cada passo
- Comandos exatos com output esperado
- Tarefas atomicas, sequenciais, verificaveis
- Nemesis enforcement valida qualidade — foco em completude
- Responder SEMPRE em PT-BR

## Integracao

**Skill anterior**: `pre-writing-rule-control`
**Proxima skill apos aprovacao**: `nemesis-subagent-driven-development`