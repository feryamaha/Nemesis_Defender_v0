---
name: nemesis-tests
description: >
  Valida a implementacao apos a Skill 4: check + testes do perfil, rebuild (autorizado
  intrinsecamente pelo pipeline), e no motor: pentest estatico, capabilities eBPF,
  nemesis-doctor e pentest full. Fix autonomo de falhas com maximo de 2 tentativas por
  falha, com reconciliacao de vereditos no Trust Ledger. Termina na PARADA UNICA do
  pipeline (relatorio consolidado + aguardar Fernando).
---

# Nemesis Tests (Validacao Pos-Execucao)

Executar bateria de testes apos implementacao (Skill 4). E a ultima fase autonoma do
pipeline: ao final dela vem a PARADA UNICA (aguardar o Fernando).

> **Texto unico espelhado nos dois repos.** Os comandos por fase vem do perfil do repo
> (`.devin/rules/nemesis-repo-profile.md`). As Fases 3, 5, 6 e 7 sao EXCLUSIVAS do perfil
> motor (pentest, eBPF, doctor, pretool); no dashboard elas sao puladas com a anotacao
> "só-motor" no relatorio.

**Anuncio de inicio**: "Estou usando a skill nemesis-tests para validar a implementacao."

**Pre-requisito**: Todas as tarefas do PLAN foram completadas (Skill 4 concluida).

## Processo

**Regra geral da skill**: cada comando e executado individualmente. O proximo comando so e
executado se o anterior passou. Se um comando falhar, NAO executar os subsequentes — ir para
Fase 4 (investigacao).

### Fase 1: Compilacao/tipos e testes unitarios

```bash
# Motor:
cd .nemesis && cargo check --workspace      # Step 1
cd .nemesis && cargo test -p nemesis-defender   # Step 2

# Dashboard:
bun run lint          # Step 1
bunx tsc --noEmit     # Step 2
```

- **PASS**: prosseguir para a Fase 2
- **FAIL**: ir para Fase 4 (investigacao)

### Fase 2: Build

O build valida a arvore inteira e, no motor, produz os binarios release contra os quais o
pentest roda (fixes precisam ser compilados ANTES do pentest, senao o pentest testa binario
defasado).

```bash
# Motor:
cd .nemesis && cargo build --release --workspace

# Dashboard:
bun run build
```

> **Autorizacao (motor)**: se o workflow foi chamado, `cargo build --release` dentro desta
> skill ja esta autorizado intrinsecamente. Nao requer HARD-GATE adicional.

- **PASS**: prosseguir (motor: Fase 3; dashboard: Fase 8)
- **FAIL**: ir para Fase 4 (investigacao)

### Fase 3: Pentest estatico (só-motor)

Apos recompilar os binarios release, executar o pentest contra os binarios atualizados:

```bash
bash .nemesis/pentest-nemesis-control/nemesis-defender/run-pentest.sh
```

- **PASS**: `FAIL=0` e `STATUS: APROVADO` → prosseguir para Fase 5
- **FAIL**: `FAIL>0` → ir para Fase 4 (investigacao)
- Verificar especificamente:
  - Modulo novo (se existir): todos os casos novos bloqueados, FP-guard nao bloqueado
  - Modulos existentes: sem regressao (nenhum modulo que passava antes agora falha)
  - Modulo de FP: nenhum falso-positivo introduzido

### Fase 4: Investigacao de causa raiz + fix autonomo (se qualquer Step falhou)

Se qualquer comando falhou, aplicar o metodo Fable (F2, debugging por hipoteses):

1. **Investigar a causa raiz** — nao tratar sintoma
   - Ler a saida de erro COMPLETA; corrigir o PRIMEIRO erro (os demais costumam ser eco)
   - Identificar o arquivo e linha exata do problema
   - Checar a hipotese do artefato defasado (build antigo?) antes de culpar o fonte
   - Determinar se e regressao do codigo alterado ou problema pre-existente (baseline:
     a falha existe sem a mudanca? se pre-existente, NAO consertar em silencio: registrar
     no estacionamento e reportar na PARADA UNICA)
   - Confirmar a causa por predicao antes de editar
2. **Fix autonomo** (modo autonomo, default): implementar o fix cirurgico, escopo minimo,
   dentro dos arquivos da spec. **Maximo 2 tentativas por falha.**
3. **Retestar**: apos o fix, re-executar desde a Fase 1
   - Se passar: prosseguir para a proxima Fase (o fix e as tentativas ficam registrados
     para o relatorio da PARADA UNICA)
   - Se falhar apos 2 tentativas: **PARADA DE EMERGENCIA** — reportar ao Fernando com
     comando que falhou, saida completa, causa raiz com evidencia, fixes tentados e
     hipoteses descartadas. Aguardar orientacao.
4. Fix que exigiria sair do escopo da spec = parada de emergencia imediata (sem tentar).
5. **Reconciliacao de vereditos (lei F11)**: se a investigacao concluir que a falha era
   detectavel na spec ou no plano (ou seja, um gate anterior aprovou o que aqui reprovou),
   anotar a reconciliacao para o Trust Ledger: `reconciliacao: gate=[Skill 0 P1|P2|
   rule-control] deixou passar [descricao em 1 linha]`. Isso e sinal de calibracao do gate,
   nao culpa; entra no ledger na PARADA UNICA.

### Fase 5: Restaurar capabilities do eBPF (só-motor, Linux apenas)

Apos `cargo build --release`, as capabilities do eBPF sao perdidas (setcap e por-inode).
Restaurar ANTES do nemesis-doctor e ANTES de reconectar o pretool.

```bash
sudo .nemesis/scripts/ensure-ebpf-caps.sh
```

- **PASS**: capabilities aplicadas → prosseguir para Fase 6
- **FAIL**: reportar erro a Fernando, aguardar orientacao
- **macOS**: pular esta fase (eBPF nao se aplica)

### Fase 6: nemesis-doctor (só-motor, diagnostico de saude)

> **NOTA IMPORTANTE**: O nemesis-doctor deve ser executado ANTES de reconectar o pretool.
> Se o pretool estiver desconectado (maintenance mode), G4 pode reportar JSON invalido em
> `.devin/hooks.json` — isso e **esperado e nao e falha**. Apos reconectar o pretool, o
> agente nao consegue mais executar comandos de teste porque o pretool intercepta tudo.
> Por isso o nemesis-doctor roda aqui, nao depois.

```bash
.nemesis/target/release/nemesis-doctor
```

- **SAUDAVEL**: prosseguir para Fase 7
- **ATENCAO**: avaliar warnings:
  - G4 com `.devin/hooks.json` invalido = esperado se pretool desconectado (maintenance mode)
  - G5 sem `cap_bpf` = rodar Fase 5 novamente
  - Outros warnings: reportar a Fernando, perguntar se pode prosseguir
- **CRITICO**: reportar a Fernando, aguardar orientacao

### Fase 7: Reconectar pretool e executar pentest full (só-motor)

> **Autorizacao**: reconectar o pretool (descomentar hooks em `.devin/hooks.json`) pode ser
> feito pelo agente dentro desta skill, pois apenas restaura o estado original.
> **DESCONECTAR** o pretool e exclusivo do Fernando (invariante 12 do AGENTS.md) — o agente
> nunca desconecta.

> **AVISO CRITICO**: Apos reconectar o pretool, o agente NAO consegue mais executar nenhum
> comando de teste porque o pretool intercepta todas as tool calls. Os unicos testes
> possiveis com pretool ativo sao o pentest full (que e engenharia social, nao tool calls
> diretas) e a observacao do bloqueio em tempo real. Por isso TODOS os testes automatizados
> (Fases 1-6) devem estar 100% aprovados ANTES de reconectar.

1. **Reconectar pretool**: o agente descomenta os hooks em `.devin/hooks.json` (ou arquivo
   de configuracao da IDE relevante). Reconectar = restaurar o estado protegido, permitido
   ao agente; DESCONECTAR permanece exclusivo do Fernando (invariante 12)
2. **Verificar pretool ativo**: executar um comando benigno e confirmar que o pretool intercepta
3. **Executar pentest full**: colar o conteudo de `nemesis-pentest-harness.md` como prompt
   para o agente, ou executar manualmente os vetores das Partes do harness
4. **Validar resultados**:
   - Partes existentes: sem regressao (todos bloqueados como antes)
   - Parte nova (se existir): todos os vetores novos bloqueados
   - FP-guards: nenhum falso-positivo introduzido

### Fase 8: PARADA UNICA do pipeline (HARD-GATE humano)

- **Tudo passou**: (1) invocar `nemesis-trust-ledger-update` para registrar os vereditos do
  ciclo e as reconciliacoes no Trust Ledger (lei F11); (2) emitir o relatorio consolidado da
  PARADA UNICA (formato no workflow do SDD pipeline): spec, plano, git diff real, tabela de
  validacao com saidas literais, decisoes tomadas, achados fora de escopo, secao Trust
  Ledger. **AGUARDAR o Fernando.** NAO invocar doc-sync nem finishing automaticamente:
  essas skills so executam com autorizacao explicita dele.
- **Algo falhou e nao foi possivel fixar** (2 tentativas esgotadas): parada de emergencia,
  reportar com evidencia completa (a parada de emergencia tambem e registrada no ledger).

## Saida Esperada

```
VALIDACAO POS-EXECUCAO (perfil: [motor|dashboard]):
  [PASS] Fase 1 (check/lint + testes)
  [PASS] Fase 2 (build; motor: release com autorizacao intrinseca)
  [PASS] Fase 3 (pentest estatico)            [só-motor]
  [PASS] Fase 5 (capabilities eBPF)           [só-motor, Linux]
  [PASS] Fase 6 (nemesis-doctor)              [só-motor]
  [PASS] Fase 7 (pentest full, pretool ativo) [só-motor]
  [OK]   Trust Ledger atualizado (vereditos + reconciliacoes do ciclo)

⛔ PARADA UNICA: relatorio consolidado emitido. Aguardando Fernando
   (doc-sync e finishing so com autorizacao explicita).
```

## Integracao

**Skill anterior**: `nemesis-subagent-driven-development` (Skill 4)
**Proximo passo**: PARADA UNICA (relatorio + aguardar Fernando). Depois, SOMENTE com
autorizacao explicita dele: `nemesis-doc-sync` (4.6) e/ou `nemesis-finishing-branch` (5)

## Lembrar

- Os comandos sao obrigatorios e sequenciais (1 por vez, proximo so se anterior passou)
- Motor: build release vem ANTES do pentest; caps eBPF ANTES do doctor; doctor ANTES de
  reconectar o pretool; apos reconectar, so pentest full (engenharia social)
- Investigar causa raiz, nao sintoma; reconciliar veredito de gate furado no ledger (F11)
- Fernando aprova qualquer fix que saia do escopo da spec
- Sempre PT-BR
