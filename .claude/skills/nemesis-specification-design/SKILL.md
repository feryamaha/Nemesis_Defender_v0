---
name: nemesis-specification-design
description: >
  Converte request informal em especificacao tecnica estruturada para o Nemesis (motor Rust
  ou dashboard Next.js, conforme o perfil do repo). Auto-ativa quando Fernando descreve uma
  necessidade. NUNCA escreva codigo antes do design ser validado. No modo autonomo (default)
  o gate da spec e a analise critica (Skill 0, Ponto 1); a spec e gravada sem aguardar
  aprovacao humana.
---

# Nemesis Specification Design

Converter requests informais em especificacoes tecnicas estruturadas.

> **Texto unico espelhado nos dois repos.** Stack, comandos e paths vem do perfil do repo
> (`.devin/rules/nemesis-repo-profile.md`).

**Anuncio de inicio**: "Estou usando a skill nemesis-specification-design para gerar uma especificacao tecnica."

## GATE (por modo)

NAO execute qualquer skill de implementacao e NAO escreva codigo antes do design validado.

- **MODO AUTONOMO (default)**: o gate e AUTOMATICO. Apos gerar a spec, invocar
  `nemesis-critical-analysis` (Ponto 1). Veredito PROSSEGUIR = gravar a spec e seguir o
  pipeline sem pausa. Veredito REJEITAR = ajustar e re-analisar (1 ciclo); segundo REJEITAR =
  parada de emergencia (reportar ao Fernando).
- **MODO SUPERVISIONADO** (so quando o Fernando pedir): apresentar a spec e BLOQUEAR ate
  aprovacao explicita ("sim", "pode", "aprovado", "ok", "prossiga", "continua").

## Processo

### Step 1: Entender Contexto do Projeto

Ler a paisagem do repo para grounding da analise, conforme o perfil:

```bash
# Motor (Nemesis_Defender_v0):
cat CLAUDE.md | head -50
ls .nemesis/*/Cargo.toml
cat .nemesis/Cargo.toml | grep -A 20 "\[workspace\]"

# Dashboard (Dashboard-Nemesis-Defender):
cat AGENTS.md | head -60
cat package.json
ls src/app src/lib src/schema
```

Identificar: stack do perfil, modulos afetados, padroes existentes, regras vigentes.

### Step 2: Analisar Request e Gerar Especificacao

Analisar o request de Fernando e gerar a especificacao **COMPLETA** em uma unica passagem.
NAO fazer perguntas socraticas.

**Mapa de traducao (informal → tecnica), exemplos:**
- "nao compila" → "Erro de compilacao em <crate/modulo> X"
- "nao roda o teste" → "suite do perfil falha em X::Y::test_Z"
- "ta lento" → "Latencia/performance acima de threshold"
- "nao valida corretamente" → "Regra semantica nao e enforced / schema nao valida"

**Disciplina epistemica:**
- NUNCA tratar framing do usuario como verdade absoluta
- Quando evidencia e ambigua: declarar incerteza na spec
- Fazer assumpcoes razoaveis quando necessario, documentadas em CONTEXT

**Estrutura de especificacao (gerar completa de uma vez):**

1. **REQUEST** — Traducao tecnica da necessidade
2. **CATEGORY** — Bugfix | Feature | Refactor | Infra | Docs
3. **PROBLEM** — Sintomas observaveis somente, SEM hipoteses causais
4. **CONTEXT** — Modulos afetados, sintomas, comportamento esperado, assumpcoes
5. **REQUIREMENTS** — O que deve ser feito (tecnico, na stack do perfil)
6. **FILES INVOLVED** — Paths exatos
7. **RESTRICTIONS** — Limites nao-negociaveis (regras do perfil, compatibilidade)
8. **EXPECTED DELIVERY** — Resultado concreto e verificavel, com os comandos de validacao
   do perfil e o resultado esperado de cada um

**Exemplo (perfil motor; no dashboard a mesma estrutura com stack TS/bun):**
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
- Regras 1-6 do perfil do repo (linguagem, toolchain, areas sensiveis, escopo, git, artefatos)
- Nao quebrar testes existentes

EXPECTED DELIVERY:
- Novo visitor compilado e integrado
- cargo check -p ast-linters: PASS
- cargo test -p ast-linters: PASS (incluindo novos testes)
- Detecta unsafe blocks em arquivo de teste Rust

VERIFICATION:
$ cd .nemesis && cargo check -p ast-linters
$ cd .nemesis && cargo test -p ast-linters
```

### Step 2.5: Ler o codigo real dos pontos de contato (obrigatorio, lei F1/F6)

Antes de fechar FILES INVOLVED, ler os arquivos que a spec cita (metodo Fable, F1:
nenhum path citado sem confirmacao no disco; F6: nenhuma assinatura sem grep).
Assumpcoes que nao puderam ser verificadas ficam DECLARADAS na secao CONTEXT.

### Step 3: Validar (gate automatico no modo autonomo)

Invocar `nemesis-critical-analysis` (Ponto 1) sobre a spec gerada.

- **PROSSEGUIR**: seguir para Step 4 (gravar) sem pausa.
- **REJEITAR**: ajustar a spec conforme a justificativa e re-analisar (maximo 1 ciclo).
  Segundo REJEITAR = parada de emergencia: reportar veredito + evidencia ao Fernando.

No modo supervisionado: apresentar a spec ao Fernando e bloquear ate aprovacao.

### Step 4: Salvar Especificacao

Apos veredito PROSSEGUIR (ou aprovacao, no modo supervisionado), salvar no path de specs do
perfil (motor: `Feature-Documentation/SPECS/`; dashboard: `.devin/specs/`), nome
`SPEC_NNN_nome-descritivo.md`, numero auto-increment verificado com `ls` antes de gravar
(nunca assumir).

## Lembrar

- NUNCA escrever codigo antes de design validado (analise critica no autonomo; Fernando no supervisionado)
- Ler o codigo real dos pontos de contato ANTES de fechar a spec (nenhum path inventado)
- Gerar especificacao completa SEM fazer perguntas
- Somente sintomas observaveis, NUNCA hipoteses causais
- Documentar assumpcoes quando fizer inferencias razoaveis
- Responder SEMPRE em PT-BR, escrever specs em PT-BR

## Proxima Skill

**Apos spec gravada** (modo autonomo, sem pausa):
1. Invocar `pre-writing-rule-control` para validacao de regras
2. Se validacao PASS: invocar `nemesis-writing-plans`
