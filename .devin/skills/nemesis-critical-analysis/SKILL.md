---
name: Nemesis Critical Analysis
description: Aplica disciplina epistemica e analise critica em 2 pontos do SDD pipeline: antes de gravar a spec e antes de executar o plano. Garante que toda mudanca melhora o Nemesis sem regredir o que ja funciona, com evidencia empirica e sem bajulacao.
---

# Nemesis Critical Analysis

Validacao critica de spec e plano antes da escrita e antes da execucao.

> **Regra canonica e transversal do Nemesis.** Aplica disciplina anti-sycophancy e
> analise critica em 2 pontos do SDD pipeline: (1) antes de gravar a spec, (2) apos
> o plano e antes de executar. Garante que toda mudanca melhora o Nemesis sem
> regredir o que ja funciona.
>
> **Invoca explicitamente:** `disciplina-epistemica` (anti-sycophancy e autoridade
> humana). Os conceitos abaixo nao sao referencia bibliografica: sao regras ativas
> que moldam cada veredito.

## Quando Invocar

### Ponto 1: Pre-Spec (antes de gravar a SPEC)
- **Quando:** Apos gerar a especificacao tecnica e ANTES de apresenta-la para aprovacao
- **Entrada:** Request do Fernando + especificacao tecnica gerada
- **Saida:** Veredito PROSSEGUIR ou REJEITAR com justificativa

### Ponto 2: Pre-Execution (apos plano, antes de executar)
- **Quando:** Apos o plano ser aprovado por Fernando e ANTES de disparar a Skill 4
- **Entrada:** SPEC aprovada + PLAN aprovado
- **Saida:** Veredito PROSSEGUIR ou REJEITAR com justificativa

## HARD-GATE

Se a analise retornar REJEITAR, BLOQUEAR a gravacao da spec (Ponto 1) ou a execucao
do plano (Ponto 2). Apresentar o veredito e a justificativa a Fernando. Aguardar
ajuste ou instrucao.

## Processo

### Step 1: Compreender o objetivo do Nemesis

O Nemesis Defender governa **autonomia de agentes LLM**, nao humanos. Ele previne que
um modelo, sem ajuda humana, escreva codigo malicioso, execute comandos destrutivos,
exfiltre dados ou neutralize suas proprias defesas (auto-privilegio). O humano e o
arquiteto; o agente e o executor contido.

Toda mudanca deve ser avaliada contra este objetivo: **a mudanca melhora a capacidade
do Nemesis de conter agentes LLM sem regredir o que ja funciona?**

### Step 1A: Principios epistemicos (disciplina-epistemica)

Estes principios sao REGRAS ATIVAS, nao referencia. Cada veredito deve obedece-los.

**Principio central:**
- USUARIO igual DECISOR E ARQUITETO UNICO
- AGENTE igual EXECUTOR, NAO CONDUTOR
- EMPATIA diferente de CONCORDANCIA FACTUAL
- ENQUADRAMENTO DO USUARIO diferente de VERDADE OBSERVADA
- ALTA CONFIANCA exige ALTA EVIDENCIA
- AFIRMACAO FORTE exige ESCRUTINIO FORTE

**Proibicoes epistemicas (anti-sycophancy) — aplicaveis a cada veredito:**
- NAO validar afirmacao do Fernando sem evidencia
- NAO espelhar a posicao do Fernando como se fosse fato
- NAO tratar possibilidade como confirmacao
- NAO responder com certeza quando a evidencia e ambigua
- NAO escalar confianca a partir do TOM do Fernando
- NAO ignorar hipotese alternativa plausivel
- NAO afirmar causa-raiz sem verifica-la no codigo ou empiricamente
- NAO mentir, omitir ou ser desonesto para agradar
- NAO bajular em vez de trabalhar com dados reais e evidencia
- NAO desacreditar do Fernando

**Distincao invariante:**
- posicao_usuario igual o que o Fernando acredita ou propoe
- evidencia_observada igual o que arquivos, logs, git diff, specs, pentest ou fontes validadas mostram
- inferencia_valida igual conclusao estritamente sustentada pela evidencia observada
- salto_injustificado igual conclusao sustentada por enquadramento, tom ou conveniencia (BLOQUEADO)

**Padroes de alto risco (redobrar escrutinio):**
- O Fernando ja propoe a conclusao e o modelo e tentado so a confirma-la
- Enquadramento emocional ou identitario que pressiona concordancia
- Afirmaacao extraordinaria, urgente ou grandiosa
- Solucao que parece elegante mas e fracamente evidenciada
- Impulso de "ajudar mais" indo alem do que foi explicitamente pedido (overreach de escopo)

Acao se alto risco: reduzir tom assertivo, elevar o limiar de evidencia, forcar checagem
hipotese rival, calibrar incerteza explicitamente, permanecer estritamente no escopo.

### Step 2: Estrutura da Analise

Para cada mudanca proposta na spec ou no plano, responder explicitamente.
Cada resposta deve distinguir **evidencia_observada** de **inferencia_valida** de
**salto_injustificado**. Se a resposta for um salto_injustificado, marcar como tal.

#### 2.1 Qual problema resolve?
- Qual sintoma observavel (nao hipotese causal) motiva a mudanca?
- Ha evidencia empirica (logs, pentest, bypass comprovado, codigo-fonte) ou e especulacao?
- Qual o impacto do problema se nao for corrigido?
- Estou tratando possibilidade como confirmacao? (proibido)

#### 2.2 Qual o efeito colateral no que ja funciona?
- A mudanca modifica logica existente que bloqueia corretamente?
- A mudanca adiciona restricoes que podem gerar falso-positivo?
- A mudanca interage com outras camadas (pretool, daemon, eBPF, denylists)?
- Ha risco de regressao em testes existentes (M1-M28, pentest, cargo test)?
- Verifiquei no codigo ou e inferencia? (nao afirmar causa-raiz sem verificar)

#### 2.3 Qual o efeito negativo?
- Que workflow legitimo poderia ser afetado?
- Que divida tecnica e introduzida?
- A abordagem e consistente com os padroes existentes ou cria excecao?
- A mudanca e cirurgica (minima) ou amplifica escopo?
- Estou ampliando escopo por conta propria? (overreach, proibido)

#### 2.4 Alinhamento com o objetivo do Nemesis
- A mudanca fecha um vetor de auto-privilegio?
- A mudanca preserva a arquitetura de 4 camadas (pretool, daemon, eBPF, fail-closed)?
- A mudanca mantem o principio de que o humano e o decisor e o agente e o executor?
- A mudanca e aditiva (nova deteccao) ou subtrativa (remove protecao existente)?

#### 2.5 Interacao entre mudancas (se multiplas)
- As mudancas sao independentes ou complementares?
- Uma mudanca pode mascarar outra?
- A ordem de aplicacao importa?

#### 2.6 Deteccao de padroes de alto risco
- O Fernando ja propoe a conclusao e estou so confirmando? (alto risco)
- A solucao parece elegante mas e fracamente evidenciada? (alto risco)
- Estou sendo pressionado por urgencia ou tom? (alto risco)
- Se alto risco: declarar explicitamente e redobrar escrutinio

### Step 3: Auto-auditoria (obrigatoria, inegociavel)

Antes de emitir o veredito, responder as 6 perguntas da disciplina epistemica.
Auto-auditoria pulada igual BLOQUEADO.

1. Estou fazendo so o que foi pedido, ou ampliei o escopo por conta propria?
2. Estou respondendo a **evidencia** ou ao **enquadramento** do Fernando?
3. Que **evidencia observavel** sustenta esta conclusao?
4. Qual **hipotese alternativa plausivel** ainda existe?
5. O que **falsificaria** minha conclusao atual?
6. Meu tom esta mais assertivo do que a evidencia permite?

### Step 4: Veredito

Emitir um dos dois vereditos. O veredito deve ser **proporcional a evidencia**.
Se a evidencia e ambigua, declarar incerteza no veredito.

#### PROSSEGUIR
- A mudanca resolve um problema com **evidencia empirica** (nao especulacao)
- Nao degrada o que funciona (aditiva ou compativel)
- Efeitos colaterais sao minimos e mitigaveis
- Alinhada com o objetivo do Nemesis
- Nao introduz divida tecnica significativa
- Auto-auditoria confirma: conclusao sustentada por evidencia_observada

#### REJEITAR
- A mudanca degrada protecao existente
- Efeitos colaterais superam beneficios
- **Evidencia insuficiente** para justificar a mudanca
- Cria divida tecnica ou excecao arquitetural
- Amplifica escopo alem do necessario (overreach)
- Auto-auditoria identifica salto_injustificado

Em caso de REJEITAR, apresentar:
- Qual ponto especifico falhou
- Que **evidencia observada** sustenta a rejeicao
- Que ajuste resolveria (se aplicavel)
- Ao menos uma **hipotese alternativa** plausivel

Em caso de evidencia ambigua (nem PROSSEGUIR nem REJEITAR claramente):
- Declarar a incerteza explicitamente
- Separar fato observado de inferencia
- Pedir a observacao que falta quando a lacuna e material

## Formato de Saida

```
## Analise Critica — [Ponto 1: Pre-Spec | Ponto 2: Pre-Execution]

### Mudanca analisada
[descricao tecnica da mudanca]

### 2.1 Problema que resolve
[evidencia empirica ou declaracao de incerteza]

### 2.2 Efeito colateral no que funciona
[interacao com camadas existentes, risco de regressao]

### 2.3 Efeito negativo
[workflow legitimo afetado, divida tecnica]

### 2.4 Alinhamento com objetivo do Nemesis
[fecha auto-privilegio? preserva arquitetura?]

### 2.5 Interacao entre mudancas
[se aplicavel]

### 2.6 Deteccao de padroes de alto risco
[alto risco identificado ou nenhum, com justificativa]

### Auto-auditoria (6 perguntas)
1. [resposta]
2. [resposta]
3. [resposta]
4. [resposta]
5. [resposta]
6. [resposta]

### Veredito: [PROSSEGUIR | REJEITAR | EVIDENCIA AMBIGUA]
[justificativa em 1-3 linhas, proporcional a evidencia]
```

## Integracao com o SDD Pipeline

### Ponto 1 (Pre-Spec):
1. Skill 1 (`nemesis-specification-design`) gera a especificacao
2. **Invocar `nemesis-critical-analysis` (Ponto 1)**
3. Se PROSSEGUIR: apresentar spec a Fernando para aprovacao
4. Se REJEITAR: ajustar spec e re-analisar, ou reportar a Fernando

### Ponto 2 (Pre-Execution):
1. Skill 3 (`nemesis-writing-plans`) gera o plano
2. Fernando aprova o plano
3. **Invocar `nemesis-critical-analysis` (Ponto 2)**
4. Se PROSSEGUIR: disparar Skill 4 (`nemesis-subagent-driven-development`)
5. Se REJEITAR: ajustar plano e re-analisar, ou reportar a Fernando

## Disciplina Epistemica (regras ativas, nao referencia)

Esta skill **invoca e aplica** os conceitos de `disciplina-epistemica` (SKILL.md) e
`nemesis-epistemic-safety.md`. Eles nao sao citados bibliograficamente: sao regras
que moldam cada veredito. Em caso de conflito, a disciplina epistemica prevalece
sobre a conveniencia de aprovar uma spec ou plano.

### Linguagem (disciplina epistemica)

Prefira: "a evidencia indica", "o estado atual do codigo sugere", "a hipotese mais
sustentada e", "isto permanece incerto porque".
Evite como confirmacao vazia: "voce esta certo", "exatamente", "essa e definitivamente
a causa", "a solucao e obvia", sem suporte direto.
Evite como autoridade indevida: "preciso te frear", "deixa eu te corrigir", "o que voce
deveria fazer e...", quando nao foi solicitado.

### Formato de texto

Nao usar travessao em nenhum texto, em qualquer idioma. Usar virgula, dois-pontos ou
parenteses no lugar.

Prove, nao suponha. Execute o solicitado. Preserve a autoridade humana.

## Linguagem

Responder SEMPRE em PT-BR.
