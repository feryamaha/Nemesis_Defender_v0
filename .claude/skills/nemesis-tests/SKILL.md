---
name: nemesis-tests
description: >
  Valida a implementacao apos a Skill 4: cargo check + cargo test, rebuild release (autorizado
  intrinsecamente pelo pipeline), pentest estatico, capabilities eBPF, nemesis-doctor e pentest
  full. Fix autonomo de falhas com maximo de 2 tentativas por falha. Termina na PARADA UNICA
  do pipeline (relatorio consolidado + aguardar Fernando).
---

# Nemesis Tests (Validacao Pos-Execucao)

Executar bateria de testes apos implementacao (Skill 4). E a ultima fase autonoma do pipeline:
ao final dela vem a PARADA UNICA (aguardar o Fernando). Garante que o codigo compila, testes
unitarios passam, pentest estatico nao regrediu, binarios sao recompilados e o pentest full
com pretool reconectado valida os fixes.

**Anuncio de inicio**: "Estou usando a skill nemesis-tests para validar a implementacao."

**Pre-requisito**: Todas as tarefas do PLAN foram completadas (Skill 4 concluida).

## Processo

### Fase 1: Compilacao e testes unitarios (2 comandos em sequencia)

**Regra geral da skill**: cada comando e executado individualmente. O proximo comando so e executado se o anterior passou. Se um comando falhar, NAO executar os subsequentes — ir para Fase 4 (investigacao).

```bash
# Step 1: so executa Step 2 se este passar
cd .nemesis && cargo check --workspace

# Step 2: so executa Fase 2 se este passar
cd .nemesis && cargo test -p nemesis-defender
```

#### Step 1: cargo check --workspace

```bash
cd .nemesis && cargo check --workspace
```

- **PASS**: `Finished` sem `error[E...]` → prosseguir para Step 2
- **FAIL**: `error[E...]` presente → ir para Fase 4 (investigacao)

#### Step 2: cargo test -p nemesis-defender

```bash
cd .nemesis && cargo test -p nemesis-defender
```

- **PASS**: todos os testes `ok` → prosseguir para Fase 2
- **FAIL**: `FAILED` ou `panicked` → ir para Fase 4 (investigacao)

### Fase 2: Recompilacao de binarios release

O pentest estatico roda contra o binario release. Os fixes precisam ser compilados ANTES de
rodar o pentest, caso contrario o pentest testa um binario defasado.

```bash
cd .nemesis && cargo build --release --workspace
```

> **Autorizacao**: se o workflow foi chamado, `cargo build --release` dentro desta skill ja esta
> autorizado intrinsecamente. Nao requer HARD-GATE adicional.

- **PASS**: `Finished` sem erros → prosseguir para Fase 3
- **FAIL**: reportar erro a Fernando, aguardar orientacao

### Fase 3: Pentest estatico (run-pentest.sh)

Apos recompilar os binarios release, executar o pentest contra os binarios atualizados:

```bash
bash .nemesis/pentest-nemesis-control/nemesis-defender/run-pentest.sh
```

- **PASS**: `FAIL=0` e `STATUS: APROVADO` → prosseguir para Fase 5
- **FAIL**: `FAIL>0` → ir para Fase 4 (investigacao)
- Verificar especificamente:
  - M29 (se existir): todos os casos novos bloqueados, FP-guard nao bloqueado
  - M1-M28: sem regressao (nenhum modulo que passava antes agora falha)
  - M26 (FP): nenhum falso-positivo introduzido

### Fase 4: Investigacao de causa raiz + fix autonomo (se qualquer Step falhou)

Se qualquer comando das Fases 1-3 falhou, aplicar o metodo Fable (debugging por hipoteses):

1. **Investigar a causa raiz** — nao tratar sintoma
   - Ler a saida de erro COMPLETA; corrigir o PRIMEIRO erro (os demais costumam ser eco)
   - Identificar o arquivo e linha exata do problema
   - Checar a hipotese do artefato defasado (binario release antigo?) antes de culpar o fonte
   - Determinar se e regressao do codigo alterado ou problema pre-existente (baseline:
     a falha existe sem a mudanca? se pre-existente, NAO consertar em silencio: registrar
     no estacionamento e reportar na PARADA UNICA)
   - Confirmar a causa por predicao antes de editar
2. **Fix autonomo** (modo autonomo, default): implementar o fix cirurgico, escopo minimo,
   dentro dos arquivos da spec. **Maximo 2 tentativas por falha.**
3. **Retestar**: apos o fix, re-executar desde a Fase 1 Step 1
   - Se passar: prosseguir para a proxima Fase (o fix e as tentativas ficam registrados
     para o relatorio da PARADA UNICA)
   - Se falhar apos 2 tentativas: **PARADA DE EMERGENCIA** — reportar ao Fernando com
     comando que falhou, saida completa, causa raiz com evidencia, fixes tentados e
     hipoteses descartadas. Aguardar orientacao.
4. Fix que exigiria sair do escopo da spec = parada de emergencia imediata (sem tentar).

### Fase 5: Restaurar capabilities do eBPF (Linux apenas)

Apos `cargo build --release`, as capabilities do eBPF sao perdidas (setcap e por-inode).
Restaurar ANTES do nemesis-doctor e ANTES de reconectar o pretool.

```bash
sudo .nemesis/scripts/ensure-ebpf-caps.sh
```

- **PASS**: capabilities aplicadas → prosseguir para Fase 6
- **FAIL**: reportar erro a Fernando, aguardar orientacao
- **macOS**: pular esta fase (eBPF nao se aplica)

### Fase 6: nemesis-doctor (diagnostico de saude)

> **NOTA IMPORTANTE**: O nemesis-doctor deve ser executado ANTES de reconectar o pretool.
> Se o pretool estiver desconectado (maintenance mode), G4 pode reportar JSON invalido em
> `.devin/hooks.json` — isso e **esperado e nao e falha**. Apos reconectar o pretool, o agente
> nao consegue mais executar comandos de teste (cargo, bash, etc.) porque o pretool intercepta
> tudo. Por isso o nemesis-doctor roda aqui, nao depois.

```bash
.nemesis/target/release/nemesis-doctor
```

- **SAUDAVEL**: prosseguir para Fase 7
- **ATENCAO**: avaliar warnings:
  - G4 com `.devin/hooks.json` invalido = esperado se pretool desconectado (maintenance mode)
  - G5 sem `cap_bpf` = rodar Fase 5 novamente
  - Outros warnings: reportar a Fernando, perguntar se pode prosseguir
- **CRITICO**: reportar a Fernando, aguardar orientacao

### Fase 7: Reconectar pretool e executar pentest full

> **Autorizacao**: reconectar o pretool (descomentar hooks em `.devin/hooks.json`) pode ser feito
> pelo agente dentro desta skill, pois apenas restaura o estado original. **DESCONECTAR** o pretool
> e exclusivo do Fernando (invariante 12 do AGENTS.md) — o agente nunca desconecta.

> **AVISO CRITICO**: Apos reconectar o pretool, o agente NAO consegue mais executar nenhum
> comando de teste (cargo, bash, run-pentest.sh, etc.) porque o pretool intercepta todas as
> tool calls. Os unicos testes possiveis com pretool ativo sao o pentest full (que e engenharia
> social, nao tool calls diretas) e a observacao do bloqueio em tempo real. Por isso TODOS os
> testes automatizados (Fases 1-6) devem estar 100% aprovados ANTES de reconectar.

1. **Reconectar pretool**: o agente descomenta os hooks em `.devin/hooks.json` (ou arquivo de
   configuracao da IDE relevante). Reconectar = restaurar o estado protegido, permitido ao
   agente; DESCONECTAR permanece exclusivo do Fernando (invariante 12)
2. **Verificar pretool ativo**: executar um comando benigno e confirmar que o pretool intercepta
3. **Executar pentest full**: colar o conteudo de
   `nemesis-pentest-harness.md` como prompt para o agente, ou executar
   manualmente os vetores das Partes 1-13
4. **Validar resultados**:
   - Partes 1-12: sem regressao (todos bloqueados como antes)
   - Parte 13 (se existir): todos os vetores novos bloqueados
   - FP-guards: nenhum falso-positivo introduzido

### Fase 8: PARADA UNICA do pipeline (HARD-GATE humano)

- **Tudo passou**: emitir o relatorio consolidado da PARADA UNICA (formato no
  `nemesis-sdd-pipeline.md`): spec, plano, git diff real, tabela de validacao com saidas
  literais, decisoes tomadas, achados fora de escopo. **AGUARDAR o Fernando.**
  NAO invocar doc-sync nem finishing automaticamente: essas skills so executam com
  autorizacao explicita dele.
- **Algo falhou e nao foi possivel fixar** (2 tentativas esgotadas): parada de emergencia,
  reportar com evidencia completa.

## Saida Esperada

```
VALIDACAO POS-EXECUCAO:
  [PASS] cargo check --workspace
  [PASS] cargo test -p nemesis-defender
  [PASS] cargo build --release --workspace (autorizacao intrinseca do pipeline)
  [PASS] run-pentest.sh (FAIL=0, STATUS: APROVADO)
  [PASS] ensure-ebpf-caps.sh (Linux apenas)
  [PASS] nemesis-doctor (SAUDAVEL ou ATENCAO esperado em maintenance)
  [PASS] pentest full com pretool reconectado (Partes 1-13 bloqueadas)

⛔ PARADA UNICA: relatorio consolidado emitido. Aguardando Fernando
   (doc-sync e finishing so com autorizacao explicita).
```

## Integracao

**Skill anterior**: `nemesis-subagent-driven-development` (Skill 4)
**Proximo passo**: PARADA UNICA (relatorio + aguardar Fernando). Depois, SOMENTE com
autorizacao explicita dele: `nemesis-doc-sync` (4.6) e/ou `nemesis-finishing-branch` (5)

## Lembrar

- Os comandos sao obrigatorios e sequenciais (1 por vez, proximo so se anterior passou)
- `cargo build --release` vem ANTES do run-pentest.sh (o pentest roda contra o binario release)
- `ensure-ebpf-caps.sh` vem ANTES do nemesis-doctor (caps perdidas a cada rebuild)
- `nemesis-doctor` vem ANTES de reconectar o pretool (apos reconectar, nao da para testar)
- Apos reconectar o pretool, o agente so pode fazer pentest full (engenharia social), nao tool calls
- Investigar causa raiz, nao sintoma
- Fernando aprova qualquer fix necessario
- `cargo build --release` dentro do workflow ja esta autorizado (nao requer HARD-GATE)
- Reconectar pretool pode ser feito pelo agente (apenas restaurar hooks); DESCONECTAR e exclusivo do Fernando
- O pentest full valida os fixes contra vetores reais com pretool ativo
- Sempre PT-BR
