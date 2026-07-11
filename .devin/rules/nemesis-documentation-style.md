---
trigger: always_on
status: active
scope: canonical
last_updated: 2026-07-03
---

# Nemesis: Estilo de Documentacao

> Regra canonica e transversal para todo conteudo textual de documentacao do Nemesis
> (README.md, dashboard JSONs em src/data/docs/, qualquer artefato HTML ou MD voltado ao leitor).

## Regra 1: Sem travessao

Nao usar em dash (—) nem en dash (–) em nenhum texto, em qualquer idioma.
Usar virgula, dois-pontos ou parenteses no lugar.
Para intervalos numericos, usar "1 a 3" em vez de "1–3".

## Regra 2: Sem primeira pessoa do singular

Nao usar "eu", "meu", "minha", "meus", "minhas" em textos de documentacao.
Usar classe gramatical narrativa tecnica dissertativa: voz impessoal, terceira
pessoa ou estrutura passiva.
Exemplos:
- "Eu escolhi esse numero" -> "A escolha desse numero" ou "Esse numero foi escolhido"
- "Citacoes que eu verifiquei" -> "Citacoes verificadas contra o repositorio"
- "minha contencao" -> "a contencao" ou "a contencao do sistema"

## Regra 3: Texto minimo, maximo de imagens

Minificar a quantidade de texto dissertativo em paginas HTML de documentacao.
Condensar paragrafos longos em bullets curtos ou frases diretas quando possivel.
Substituir blocos de texto explicativo por diagramas, SVGs ou ilustracoes sempre
que o conteudo permitir representacao visual.
Objetivo: melhorar leitura, entendimento e interpretacao do leitor.
Manter rastreabilidade tecnica (citacoes de arquivo-fonte) de forma concisa.

## Integracao

Aplique junto com: as invariantes de seguranca do AGENTS.md, a disciplina
epistemica (.devin/rules/nemesis-epistemic-safety.md) e o SDD pipeline.
