---
name: nemesis-subagent-driven-development
description: >
  Executa plano de implementacao tarefa por tarefa usando subagentes com contexto isolado.
  Two-stage review apos cada tarefa: spec compliance depois code quality. Execucao continua sem pausas entre tarefas.
---

# Nemesis Subagent-Driven Development (Rust)

Executar plano de implementacao enviando um agente fresco por tarefa, com two-stage review apos cada um:
validacao de spec compliance primeiro, depois validacao de qualidade de codigo.

**Principio core**: Agente fresco por tarefa + two-stage review (spec then quality) = qualidade alta, iteracao rapida.

**Execucao continua**: NAO pause para check-in entre tarefas. Execute TODAS as tarefas do plano sem parar.
As unicas razoes para parar sao: BLOQUEADO que voce nao consegue resolver, ambiguidade que genuinamente
impede progresso, ou todas as tarefas completadas.

**Anuncio de inicio**: "Estou usando a skill nemesis-subagent-driven-development para executar o plano."

**Pre-requisito**: Um plano aprovado existe em `Feature-Documentation/PLANS/`.

## Processo

### Step 1: Carregar e Revisar Plano

Ler o arquivo do plano. Revisar criticamente. Se ha preocupacoes, levanta-las ANTES de iniciar.
Se nenhuma preocupacao, criar rastreamento de tarefas e prosseguir.

```bash
cat Feature-Documentation/PLANS/PLAN_*.md | tail -1
```

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

#### Phase 3b: Disparar subagente (nemesis-implementer)

Enviar prompt com contexto preciso:
```
Tarefa atomica: [descricao completa da tarefa]
Crate: [crate afetada]
Arquivos: [lista exata de FILES INVOLVED para essa tarefa]
Codigo Rust esperado: [snippet/pseudocodigo esperado se aplicavel]
Comando de verificacao: cargo check -p <crate>
Plano original: Feature-Documentation/PLANS/PLAN_NNN.md
```

#### Phase 3c: Two-Stage Review

**Stage 1 — Spec Compliance**: 
- A implementacao de fato faz o que a spec requer?
- Todos os crates/modules afetados?
- Nenhum arquivo fora do scope?

**Stage 2 — Code Quality**:
- Codigo Rust seguro? (sem unsafe impropio, sem unwrap desnecessario)
- Segue convencoes? (snake_case, PascalCase, nomes descritivos)
- cargo check PASS?
- Nenhuma violacao de regras Nemesis?

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

2. Verificar a regra em CLAUDE.md
3. Entender o anti-padrão sendo bloqueado
4. Corrigir a implementacao
5. Re-executar `cargo check`

Se bloqueio persiste apos correcao:
- STOP execucao
- Reportar a Fernando: "Bloqueio Nemesis em TASK N: [descricao exata]"
- Aguardar instrucoes

### Step 5: Verificacao Final (Apos TODAS as Tarefas)

```bash
cd .nemesis && cargo check --workspace
cd .nemesis && cargo test --workspace
```

Ambos devem PASS.

### Step 6: Report de Execucao

Apos completar TODAS as tarefas:

```
EXECUCAO CONCLUIDA

Tarefas: N/N COMPLETAS
Erros: 0
Bloqueios: 0

Verificacao Final:
  cargo check --workspace: PASS
  cargo test --workspace: PASS

Resultado: PRONTO PARA nemesis-tests (Skill 4.5) — invocar sem pausa
```

## Red Flags (Parar Imediatamente)

- Subagent produz codigo violando regras Nemesis
- Mesma tarefa falha 2+ vezes
- Subagent nao consegue encontrar arquivos do plano
- Implementacao diverge significativamente do plano
- Subagent modifica arquivos fora do scope da tarefa

## Lembrar

- Agente fresco por tarefa — contexto isolado
- Two-stage review apos CADA tarefa
- Execucao continua — NAO pause entre tarefas
- PARE somente para blocadores irresoluveis
- Nemesis enforcement valida codigo — confie nele
- Responder SEMPRE em PT-BR

## Integracao

**Skill anterior**: `nemesis-writing-plans` (validado pela `nemesis-critical-analysis` Ponto 2)
**Proxima skill apos conclusao**: `nemesis-tests` (Skill 4.5), invocada sem pausa