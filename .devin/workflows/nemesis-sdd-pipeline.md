# Nemesis SDD Pipeline (Specification-Driven Development)

## Overview

O Nemesis SDD Pipeline governa o desenvolvimento do Nemesis Framework Rust de forma
deterministica e auditavel. A partir de um input informal do Fernando, o pipeline corre em
**modo autonomo** ate a implementacao validada, com **UMA parada obrigatoria** ao final da
validacao. As fases de documentacao (doc-sync) e finishing **nunca executam sem autorizacao
explicita do Fernando**, porque e nesse ponto que ele decide entre finalizar ou gerar novas
issues e reiniciar o ciclo.

Aplicar junto: `.devin/rules/nemesis-fable-method.md` (metodo de trabalho do modelo) e
`.devin/rules/nemesis-epistemic-safety.md` (disciplina epistemica).

## Os dois modos

- **MODO AUTONOMO (default)**: Fernando da o input e o pipeline executa sem pausas
  intermediarias ate a PARADA UNICA (pos-validacao). Specs e planos sao gravados sem aguardar
  aprovacao; o gate de qualidade deles e a analise critica (Skill 0) + rule control (Skill 2),
  ambos automaticos.
- **MODO SUPERVISIONADO**: o fluxo classico com aprovacao humana de spec e de plano. So e
  usado quando o Fernando pedir explicitamente ("modo supervisionado", "quero aprovar a spec").

## Workflow (modo autonomo)

```
┌─────────────────────────────────────────────────────────────┐
│ INPUT (Fernando descreve necessidade ou aponta uma ISSUE)   │
└────────────────────┬────────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 1: nemesis-specification-design                       │
│ Ler codigo real dos pontos de contato ANTES de especificar  │
│ OUTPUT: especificacao tecnica gerada                        │
└────────────────────┬────────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 0: nemesis-critical-analysis (PONTO 1: Pre-Spec)      │
│ GATE AUTOMATICO: PROSSEGUIR → gravar SPEC_NNN e seguir      │
│ REJEITAR → ajustar a spec e re-analisar (1 ciclo);          │
│ segundo REJEITAR → PARADA DE EMERGENCIA (reportar)          │
└────────────────────┬────────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 2: pre-writing-rule-control                           │
│ GATE AUTOMATICO: PASS → seguir; FAIL → ajustar e revalidar  │
│ (1 ciclo); segundo FAIL → PARADA DE EMERGENCIA              │
└────────────────────┬────────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 3: nemesis-writing-plans                              │
│ Tarefas atomicas, paths reais, codigo completo, verificacao │
│ OUTPUT: PLAN_NNN gravado (sem aguardar aprovacao)           │
└────────────────────┬────────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 0: nemesis-critical-analysis (PONTO 2: Pre-Execution) │
│ GATE AUTOMATICO: mesmo regime do Ponto 1                    │
└────────────────────┬────────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 4: nemesis-subagent-driven-development                │
│ Execucao continua tarefa a tarefa, two-stage review         │
│ Sem pausas; para somente em bloqueio irresoluvel            │
└────────────────────┬────────────────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ Skill 4.5: nemesis-tests (validacao completa)               │
│ cargo check + cargo test + build release (autorizado        │
│ intrinsecamente pelo pipeline) + pentest estatico + doctor  │
│ Falha → investigacao de causa raiz + fix cirurgico          │
│ autonomo (max 2 tentativas por falha); persistiu →          │
│ PARADA DE EMERGENCIA                                        │
└────────────────────┬────────────────────────────────────────┘
                     ▼
╔═════════════════════════════════════════════════════════════╗
║ ⛔ PARADA UNICA OBRIGATORIA (HARD-GATE humano)               ║
║ Apresentar: spec, plano, git diff real, tabela de           ║
║ validacao, decisoes tomadas, achados fora de escopo.        ║
║ AGUARDAR o Fernando. Opcoes dele:                           ║
║  (a) autorizar doc-sync e/ou finishing                      ║
║  (b) gerar issues a partir dos achados e reiniciar o ciclo  ║
║  (c) pedir ajustes na implementacao                         ║
║  (d) descartar                                              ║
╚════════════════════╤════════════════════════════════════════╝
                     ▼ (somente com autorizacao explicita)
┌─────────────────────────────────────────────────────────────┐
│ Skill 4.6: nemesis-doc-sync — SO COM AUTORIZACAO DO FERNANDO│
│ HARD-GATE: Fernando aprova as mudancas de doc               │
└────────────────────┬────────────────────────────────────────┘
                     ▼ (somente com autorizacao explicita)
┌─────────────────────────────────────────────────────────────┐
│ Skill 5: nemesis-finishing-branch — SO COM AUTORIZACAO      │
│ HARD-GATE: Fernando aprova PR e escolhe disposicao          │
└─────────────────────────────────────────────────────────────┘
```

## Regras Fundamentais

1. **Autonomia ate a validacao, nunca alem dela.** Entre o input e o fim da Skill 4.5 nao ha
   pausas para aprovacao. A PARADA UNICA e inegociavel: nenhuma skill pos-validacao (4.6, 5)
   e invocada automaticamente, nem "por conveniencia", nem porque "ja estava tudo verde".

2. **Gates automaticos nao sao decorativos.** A analise critica (Skill 0) e o rule control
   (Skill 2) substituem a aprovacao humana intermediaria; por isso os vereditos deles BLOQUEIAM
   de verdade. Veredito negativo permite UM ciclo de ajuste + re-analise; o segundo veredito
   negativo vira PARADA DE EMERGENCIA (reportar ao Fernando com o veredito e a evidencia).

3. **Paradas de emergencia** (alem da PARADA UNICA): bloqueio Nemesis persistente apos
   correcao; mesma tarefa/falha apos 2 tentativas de fix; escopo real materialmente maior que
   a spec; qualquer acao irreversivel ou externa nao prevista no plano (classe C do metodo
   Fable); ambiguidade que genuinamente impede progresso. Nesses casos: STOP, reportar o
   bloqueador exato com evidencia, aguardar o Fernando.

4. **Autorizacao intrinseca da validacao.** Dentro da Skill 4.5, `cargo build --release`,
   a restauracao de capabilities do eBPF e a reconexao do pretool (restaurar estado) estao
   autorizados pelo proprio pipeline. DESCONECTAR o pretool permanece exclusivo do Fernando
   (invariante 12 do AGENTS.md), sempre.

5. **Evidencia real sempre.** git diff/log reais nas PRs (nunca fabricar); numeros copiados
   da saida literal dos comandos desta sessao; falha reportada com a mesma proeminencia que
   sucesso (metodo Fable, secao 3).

6. **Fernando governa as decisoes humanas.** A PARADA UNICA, o doc-sync, o finishing e a
   disposicao da branch sao dele. Git de escrita e exclusivamente dele.

## Entradas e Saidas

| Fase | Entrada | Saida | Gate |
|------|---------|-------|------|
| 1: specification-design | Input informal / ISSUE | Spec gerada | automatico (Skill 0 P1) |
| 0 (P1): critical-analysis | Request + spec | PROSSEGUIR/REJEITAR | automatico |
| (gravacao) | Spec aprovada pela analise | SPEC_NNN.md | nenhum |
| 2: pre-writing-rule-control | SPEC_NNN.md | PASS/FAIL | automatico |
| 3: writing-plans | SPEC validada | PLAN_NNN.md gravado | automatico (Skill 0 P2) |
| 0 (P2): critical-analysis | SPEC + PLAN | PROSSEGUIR/REJEITAR | automatico |
| 4: subagent-driven-development | PLAN validado | Tarefas completas | continuo |
| 4.5: nemesis-tests | Workspace atualizado | Validacao completa | automatico |
| ⛔ PARADA UNICA | Tudo acima | Relatorio consolidado | **Fernando** |
| 4.6: doc-sync | Autorizacao explicita | Doc sincronizada | **Fernando** |
| 5: finishing-branch | Autorizacao explicita | PR_NNN.md | **Fernando** |

## Relatorio da PARADA UNICA (formato obrigatorio)

```
PIPELINE CONCLUIDO ATE A VALIDACAO — aguardando Fernando

Spec:  Feature-Documentation/SPECS/SPEC_NNN_nome.md
Plano: Feature-Documentation/PLANS/PLAN_NNN_nome.md

Diff real (git diff --stat):
[saida literal]

Validacao:
| Comando | Resultado | Observacao |
[tabela com saidas literais]

Decisoes tecnicas tomadas (com justificativa em 1 linha cada):
[lista]

Achados fora de escopo (estacionamento — sem acao tomada):
[lista com arquivo:linha, ou "nenhum"]

Proximos passos possiveis:
(a) autorizar doc-sync  (b) autorizar finishing  (c) gerar issues  (d) ajustar  (e) descartar
```

## Regras do Pipeline

### Regra 1: Rust como unica linguagem NOVA em .nemesis/
Nenhum codigo novo em .ts/.js/.py/.sh dentro de `.nemesis/`. Infra pre-existente nao-Rust
(o C do eBPF em `ebpf-kernel/`, os shell scripts herdados de `install/`, `scripts/` e
`pentest-nemesis-control/`) pode ser EDITADA quando a mudanca a exigir: herdar, nao introduzir
toolchain novo. Arquivos de configuracao e templates (.json, .toml, .service, .plist) sao
permitidos onde o design os preve.

### Regra 2: Build via Cargo Workspace
`cargo check -p <crate>` por tarefa; `cargo test -p <crate>` para validar. Nada de rustc avulso.

### Regra 3: Hooks somente em manutencao coordenada
Mexer em `.nemesis/hooks/` so com o pretool desconectado pelo Fernando (invariante 12).

### Regra 4: Scope da Spec
Nao sair dos arquivos listados na spec. Divergencia material = parada de emergencia.

### Regra 5: Git de escrita e exclusivo do Fernando
IA nunca executa git add/commit/push. Evidencia so com git read-only real.

### Regra 6: Sem binarios fora de .nemesis/target/
Nao copiar binarios para outro lugar.

## Convencoes de Nomenclatura

- **Specs**: `SPEC_NNN_nome-descritivo.md` em `Feature-Documentation/SPECS/`
- **Plans**: `PLAN_NNN_nome-descritivo.md` em `Feature-Documentation/PLANS/`
- **PRs**: `PR_NNN_nome-descritivo.md` em `Feature-Documentation/PR/`
- **Numero**: auto-increment verificado com `ls` antes de gravar (nunca assumir).

## Como Usar

Fernando descreve a necessidade (ou aponta uma issue em `Feature-Documentation/ISSUE/`):

```
"Preciso de um novo visitor para detectar X" | "Analise a ISSUE 010 e execute"
```

O pipeline inicia em modo autonomo e so para na PARADA UNICA (ou numa parada de emergencia).
Para o fluxo com aprovacoes intermediarias, pedir explicitamente o modo supervisionado.

## Suporte

Se bloqueado: (1) consultar `CLAUDE.md` / `AGENTS.md`; (2) consultar a skill especifica em
`.devin/skills/` (espelho em `.claude/skills/`); (3) reportar o bloqueador exato ao Fernando
e aguardar. Nemesis enforcement (AST + eBPF + pretool + denylists) valida a qualidade do
codigo; o pipeline governa fluxo e metodo.
