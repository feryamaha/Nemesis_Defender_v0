---
name: nemesis-specification-design
description: >
  Converte request informal em especificacao tecnica estruturada para Nemesis Framework Rust.
  Auto-ativa quando Fernando descreve uma necessidade. NUNCA escreva codigo antes do design ser aprovado.
---

# Nemesis Specification Design

Converter requests informais em especificacoes tecnicas estruturadas para o Nemesis Framework v2.0 Rust.

**Anuncio de inicio**: "Estou usando a skill nemesis-specification-design para gerar uma especificacao tecnica."

## HARD-GATE

NAO execute qualquer skill de implementacao, NAO escreva codigo Rust, NAO use Agent nemesis-implementer
ate que o design seja APRESENTADO e **APROVADO** por Fernando explicitamente.

Respostas validas de aprovacao: "sim", "pode", "aprovado", "ok", "prossiga", "continua"

## Processo

### Step 1: Entender Contexto do Projeto

Ler a paisagem do projeto para grounding da analise:

```bash
pwd
# Esperado: .../Nemesis_Rust_v2.0

cat CLAUDE.md | head -50
ls -la .nemesis/ | head -20

# Identificar qual crate(s) sera afetado
ls .nemesis/*/Cargo.toml
cat .nemesis/Cargo.toml | grep -A 20 "\[workspace\]"
```

Identificar: stack Rust, crates do workspace, padroes existentes, regras Nemesis.

### Step 2: Analisar Request e Gerar Especificacao

Analisar o request de Fernando e gerar a especificacao **COMPLETA** em uma unica passagem.
NAO fazer perguntas socraticas.

**Mapa de traducao (informal → tecnica):**
- "nao compila" → "Erro de compilacao em crate X"
- "nao roda o teste" → "cargo test falha em X::Y::test_Z"
- "ta lento" → "Latencia/performance acima de threshold"
- "nao valida corretamente" → "Regra semantica nao e enforced"

**Disciplina epistemica:**
- NUNCA tratar framing do usuario como verdade absoluta
- Quando evidencia e ambigua: declarar incerteza na spec
- Fazer assumpcoes razoaveis quando necessario, documental em CONTEXT

**Estrutura de especificacao (gerar completa de uma vez):**

1. **REQUEST** — Traducao tecnica da necessidade
2. **CATEGORY** — Bugfix | Feature | Refactor | Infra | Docs
3. **PROBLEM** — Sintomas observaveis somente, SEM hipoteses causais
4. **CONTEXT** — Crates afetadas, sintomas, comportamento esperado, assumpcoes
5. **REQUIREMENTS** — O que deve ser feito (tecnico, em Rust)
6. **FILES INVOLVED** — Paths exatos (.nemesis/crate/src/...)
7. **RESTRICTIONS** — Limites nao-negociaveis (regras Nemesis, compatibilidade)
8. **EXPECTED DELIVERY** — Resultado concreto e verificavel
   - Exemplo: "cargo check -p ast-linters PASS, cargo test -p ast-linters PASS"

**Exemplo de REQUEST:**
```
REQUEST: Implementar novo visitor tree-sitter para deteccao de unsafe blocks em eBPF hooks

CATEGORY: Feature

PROBLEM: 
- Atual: AST linter nao detecta unsafe blocks em contexto eBPF
- Esperado: Detectar unsafe blocks e classificar como violation da regra ebpf-no-unsafe

CONTEXT:
- Crate afetada: ast-linters
- Modulo: visitors/ebpf_unsafe_checker.rs (novo)
- Dependencia: tree-sitter crate ja existe e configurada
- Assumpcao: regra "ebpf-no-unsafe" ja existe em .nemesis/ast-linters/src/rules.rs

REQUIREMENTS:
1. Criar novo visitor `EbpfUnsafeChecker` que herda de `TreeSitterVisitor`
2. Implementar metodo `visit_unsafe_block(&self, node: Node) -> Vec<Violation>`
3. Retornar lista de violations com severity=HIGH, rule_id="ebpf-no-unsafe"
4. Integrar visitor em pipeline de validacao em lib.rs
5. Adicionar testes em tests/ebpf_unsafe_test.rs

FILES INVOLVED:
- .nemesis/ast-linters/src/lib.rs (modify, adicionar visitor ao pipeline)
- .nemesis/ast-linters/src/visitors/ebpf_unsafe_checker.rs (create)
- .nemesis/ast-linters/tests/ebpf_unsafe_test.rs (create, testes)
- .nemesis/ast-linters/Cargo.toml (verify dependencias)

RESTRICTIONS:
- Somente Rust, nenhum arquivo .ts/.js
- Deve compilar: cargo check -p ast-linters PASS
- Nao quebrar testes existentes: cargo test -p ast-linters PASS
- Seguir naming convention: snake_case para funcoes, PascalCase para structs
- Nao modificar .nemesis/hooks/ (requer maintenance mode)

EXPECTED DELIVERY:
- Novo visitor compilado e integrado
- cargo check -p ast-linters: PASS
- cargo test -p ast-linters: PASS (incluindo novos testes)
- Detecta unsafe blocks em arquivo de teste Rust

VERIFICATION:
$ cd .nemesis && cargo check -p ast-linters
$ cd .nemesis && cargo test -p ast-linters
```

### Step 3: Apresentar e Obter Aprovacao

**HARD-GATE**: Apresentar a especificacao completa. BLOQUEAR todas as acoes ate resposta de Fernando.

Perguntar: "A especificacao reflete sua intencao? Quer ajustar algum ponto?"

**Se Fernando pedir ajustes**: Revisar especificacao e re-apresentar.

**Se Fernando aprovar** (sim, pode, ok, etc): Prosseguir para Step 4.

### Step 4: Salvar Especificacao

Apos aprovacao, salvar em arquivo:

```
Path: Feature-Documentation/SPECS/SPEC_NNN_nome-descritivo.md
Numero: auto-increment a partir de specs existentes (001, 002, 003, ...)
```

Comando de verificacao:
```bash
ls Feature-Documentation/SPECS/SPEC_*.md | sort | tail -3
```

Arquivo markdown contendo a especificacao completa aprovada.

## Lembrar

- NUNCA escrever codigo antes de design aprovado
- Gerar especificacao completa SEM fazer perguntas
- Somente sintomas observaveis, NUNCA hipoteses causais
- Documentar assumpcoes quando fazer inferencias razoaveis
- Nemesis enforcement (AST + eBPF + pretool) valida qualidade — foco em clareza de intencao
- Responder SEMPRE em PT-BR, escrever specs em PT-BR

## Proxima Skill Apos Aprovacao

**Apos especificacao aprovada por Fernando**:
1. Invocar `pre-writing-rule-control` para validacao de regras
2. Se validacao PASS: invocar `nemesis-writing-plans`