---
name: nemesis-subagent-driven-development
description: >
  Executa plano de implementacao tarefa por tarefa usando subagentes com contexto isolado.
  Pre-flight de postura na entrada. Two-stage review apos cada tarefa com revisor
  INDEPENDENTE (spec compliance depois code quality, verificacao rodada pelo revisor).
  Execucao continua sem pausas entre tarefas.
---

# Nemesis Subagent-Driven Development

Executar plano de implementacao enviando um agente fresco por tarefa, com two-stage review
independente apos cada um: validacao de spec compliance primeiro, depois validacao de
qualidade de codigo.

> **Texto unico espelhado nos dois repos.** Comandos de verificacao e regras de stack vem do
> perfil do repo (`.devin/rules/nemesis-repo-profile.md`).

**Principio core**: Agente fresco por tarefa + two-stage review INDEPENDENTE (spec depois
quality) = qualidade alta, iteracao rapida. Trabalho delegado se verifica de forma
independente antes de integrar (lei F9); julgamento nao se delega.

**Execucao continua**: NAO pause para check-in entre tarefas. Execute TODAS as tarefas do
plano sem parar. As unicas razoes para parar sao: BLOQUEADO que voce nao consegue resolver,
ambiguidade que genuinamente impede progresso, ou todas as tarefas completadas.

**Anuncio de inicio**: "Estou usando a skill nemesis-subagent-driven-development para executar o plano."

**Pre-requisito**: Um plano aprovado existe no path de plans do perfil.

## Processo

### Step 0: Pre-flight de postura (lei F1, obrigatorio)

Antes da primeira tarefa, declarar a postura observada por comando (nao por suposicao),
conforme o perfil:
- **Motor:** `.nemesis/target/release/nemesis-doctor --quick` (G4 pretool, G5 eBPF, G6
  daemon) + `git branch --show-current` + `git status --short`.
- **Dashboard:** `git branch --show-current` + `git status --short`.

Registrar a postura no rastreamento. Postura inesperada (ex.: working tree sujo com mudancas
que nao sao suas, branch errada) = parar e reportar antes de tocar em qualquer arquivo.

### Step 1: Carregar e Revisar Plano

Ler o arquivo do plano. Revisar criticamente. Se ha preocupacoes, levanta-las ANTES de
iniciar. Se nenhuma preocupacao, criar rastreamento de tarefas e prosseguir.

### Step 2: Registrar Tarefas

Criar lista de rastreamento (pode ser texto ou interno):

```
[ ] TASK 1: [descricao]
[ ] TASK 2: [descricao]
...
[ ] TASK N: [descricao]
```

### Step 3: Executar Tarefas (Continuo, sem Pausas)

Para CADA tarefa no plano:

#### Phase 3a: Marcar como in_progress
```
[IN] TASK N: [descricao]
```

#### Phase 3b: Disparar subagente implementador (contrato de handoff COMPLETO, lei F9)

O subagente nasce sem memoria da conversa: o contrato contem tudo.

```
OBJETIVO: [descricao completa da tarefa atomica]
MODULO: [crate/diretorio afetado]
ARQUIVOS (paths exatos): [lista exata de FILES INVOLVED desta tarefa]
CODIGO ESPERADO: [snippet/pseudocodigo do plano, se aplicavel]
INVARIANTES: [regras do perfil que se aplicam: linguagem, areas sensiveis, escopo]
O QUE NAO FAZER: [nao tocar arquivos fora da lista; nao introduzir dependencia nova;
  nao executar git de escrita; nao "aproveitar e melhorar" nada adjacente]
COMANDO DE VERIFICACAO: [comando por tarefa do perfil]
FORMATO DO RESULTADO: [diff dos arquivos tocados + saida literal da verificacao]
PLANO ORIGINAL: [path do plano]
```

#### Phase 3c: Two-Stage Review INDEPENDENTE

O review e feito por um **subagente revisor distinto do implementador**, com contexto
isolado, recebendo: a tarefa do plano, o diff produzido e o contrato acima. O revisor
**RODA ele proprio o comando de verificacao do perfil** (nao confia no relato do
implementador) antes de emitir o parecer.

**Stage 1 — Spec Compliance**:
- A implementacao de fato faz o que a spec/tarefa requer?
- Todos os modulos afetados?
- Nenhum arquivo fora do scope?

**Stage 2 — Code Quality**:
- Codigo seguro e idiomatico na stack do perfil? (motor: sem unsafe improprio, sem unwrap
  desnecessario; dashboard: TS estrito, Zod safeParse, sem any gratuito)
- Segue convencoes do codigo ao redor?
- Comando de verificacao do perfil PASS (rodado pelo revisor)?
- Nenhuma violacao das regras do perfil?

#### Phase 3d: Resultado

- **Se PASS**: Marcar [✅], prosseguir proxima tarefa (sem pause)
- **Se FAIL**: [❌] Disparar follow-up subagent com contexto de erro, tentar ate 2 vezes
- **Se BLOCKED**: [🚫] STOP, reportar a Fernando exatamente o que bloqueou

### Step 4: Procedimento de Bloqueio Nemesis

Se encontrar bloqueio Nemesis (via terminal ou interface), **OBRIGATORIO**:

1. Ler o motivo do bloqueio emitido:
   ```
   ========================================
   NEMESIS BLOCKED: Violacao detectada
   ========================================
   [Regra]: ...
   [Mensagem]: ...
   [Sugestao]: ...
   ```

2. Verificar a regra em CLAUDE.md/AGENTS.md e no perfil
3. Entender o anti-padrao sendo bloqueado
4. Corrigir a implementacao (NUNCA desabilitar ou contornar o Nemesis)
5. Re-executar a verificacao

Se bloqueio persiste apos correcao:
- STOP execucao
- Reportar a Fernando: "Bloqueio Nemesis em TASK N: [descricao exata]"
- Aguardar instrucoes

### Step 5: Verificacao Final (Apos TODAS as Tarefas)

Rodar a suite completa do perfil:

```bash
# Motor:
cd .nemesis && cargo check --workspace
cd .nemesis && cargo test --workspace

# Dashboard:
bun run lint
bunx tsc --noEmit
bun run build
```

Todos devem PASS.

### Step 6: Report de Execucao

Apos completar TODAS as tarefas:

```
EXECUCAO CONCLUIDA

Postura (pre-flight): [declarada no Step 0]
Tarefas: N/N COMPLETAS
Erros: 0
Bloqueios: 0

Verificacao Final (suite do perfil):
  [comando]: PASS
  [comando]: PASS

Resultado: PRONTO PARA nemesis-tests (Skill 4.5) — invocar sem pausa
```

## Red Flags (Parar Imediatamente)

- Subagent produz codigo violando regras do perfil
- Mesma tarefa falha 2+ vezes
- Subagent nao consegue encontrar arquivos do plano
- Implementacao diverge significativamente do plano
- Subagent modifica arquivos fora do scope da tarefa
- Revisor nao consegue reproduzir o PASS relatado pelo implementador

## Lembrar

- Pre-flight de postura ANTES da primeira tarefa (F1)
- Agente fresco por tarefa — contexto isolado, contrato de handoff completo (F9)
- Revisor INDEPENDENTE roda a verificacao ele proprio — nao aceitar relato sem prova
- Execucao continua — NAO pause entre tarefas
- PARE somente para blocadores irresoluveis
- Nemesis enforcement valida codigo — confie nele, nunca o contorne
- Responder SEMPRE em PT-BR

## Integracao

**Skill anterior**: `nemesis-writing-plans` (validado pela `nemesis-critical-analysis` Ponto 2)
**Proxima skill apos conclusao**: `nemesis-tests` (Skill 4.5), invocada sem pausa
