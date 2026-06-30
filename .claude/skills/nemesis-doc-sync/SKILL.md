---
name: nemesis-doc-sync
description: >
  Trata documentacao como FEATURE. APOS a validacao (Skill 4.5) e ANTES do finishing (Skill 5),
  analisa o git diff da mudanca e decide se README.md e index.html precisam ser atualizados. Se nao,
  segue o fluxo. Se sim, reconcilia (codigo = verdade, regra do coeficiente, sem inserir por inserir),
  com HARD-GATE de revisao humana. Garante que a PR sempre contenha a documentacao sincronizada.
---

# Nemesis Doc Sync (documentacao como feature)

**Anuncio de inicio**: "Estou usando a skill nemesis-doc-sync para verificar se a mudanca exige atualizar README/index.html."

**Pre-requisito**: Skill 4.5 (`nemesis-tests`) concluida — codigo validado, testes/pentest verdes.

## Por que existe

Documentacao errada deixa o codigo em **check-mate**: o usuario confia na doc, nao no codigo. Por
isso a doc e tratada como **feature** e tem um passo proprio no pipeline, **apos a validacao e antes
do finishing** — assim a **PR sempre inclui a atualizacao de doc**, e nunca mais "atualiza codigo e
esquece a documentacao".

## Processo

### Fase 1: Coletar o que mudou (git diff real)
```bash
git diff --stat
git diff
```
Read-only (git de escrita e exclusivo do Fernando). O diff e a fonte do que mudou.

### Fase 2: GATE DE DECISAO — a mudanca afeta a documentacao publica?
Confronte o diff contra o que `README.md` e `index.html` DOCUMENTAM. Checklist de itens documentados
que podem ser afetados:
- **Contagens citadas:** categorias da `denylist-defender.json`, modulos M do pentest, crates,
  hooks BPF-LSM, visitors (arquivos vs despachados).
- **Coeficiente / camadas:** pretool denylists, eBPF, denylist embutida, heuristicas de scanner, visitors AST.
- **Plataformas suportadas** (Linux/macOS/Windows) e o que cada camada cobre.
- **Nomes de arquivos/comandos user-facing** (harness, `run-pentest.sh`, binarios, install).
- **Feature nova ou comportamento mudado** que o usuario percebe.
- **Enquadramento do pentest** (numeros, gate, modulos).
- **Passos de instalacao / requisitos.**

Para CADA item afetado, emita um veredito:
- **NAO PRECISA** — o diff nao toca nada documentado, OU a doc ja reflete. Justifique em 1 linha.
- **PRECISA** — liste exatamente o que ficou divergente (doc vs codigo), com `arquivo:linha`.

> **Regra dura: nao inserir doc por inserir.** Bugfix interno, refactor, mudanca de teste etc.
> geralmente NAO exigem README/index. So atualize o que a mudanca tornou divergente.

### Fase 3a: Veredito NAO PRECISA
Reporte "README/index.html ja refletem a mudanca; nada a atualizar" e siga para a Skill 5.

### Fase 3b: Veredito PRECISA — reconciliar (codigo = verdade)
Atualize `README.md` e `index.html` de forma ESPELHADA, com disciplina:
- **Codigo e a fonte de verdade.** Verifique cada numero/fato no codigo antes de escrever (nao invente).
- **Regra do coeficiente (AGENTS.md secao 3A):** a protecao e a soma das camadas; visitor e **metodo**,
  nao a unidade de cobertura; nao publique "N vetores = N visitors" nem numero agregado nao rastreavel.
- **Sem numero fragil:** prefira descrever por modulo/camada + gate (ex.: "M1..Mn, FAIL=0") a cravar
  um total que a proxima mudanca defasa.
- **Espelhe README <-> index.html:** os dois nao podem divergir entre si.
- **Cirurgico:** mude so o que ficou divergente; nao reescreva secoes inteiras sem necessidade.
- **Bilingue no index.html:** atualize `data-pt`, `data-en` E o texto visivel.

Apresente o diff das mudancas de doc.

### Fase 4: HARD-GATE — revisao humana
Documentacao e feature: BLOQUEIE ate o Fernando aprovar as mudancas de doc. Sem aprovacao, nao siga
para a Skill 5. Respostas validas: "sim", "pode", "aprovado", "ok", "prossiga".

## Saida

- Veredito (PRECISA / NAO PRECISA) e, se PRECISA, as mudancas de doc aplicadas e aprovadas.
- Alimenta a Skill 5 (`nemesis-finishing-branch`): a PR ja contem a doc sincronizada no git diff.

## Integracao

- **Skill anterior**: `nemesis-tests` (4.5).
- **Proxima skill**: `nemesis-finishing-branch` (5) — a PR agora inclui a documentacao atualizada.

## Lembrar

- Documentacao = feature. Doc errada = check-mate.
- GATE de decisao ANTES de editar (nao inserir por inserir).
- Codigo = verdade; regra do coeficiente; espelhar README/index.
- Git e exclusivo do Fernando. HARD-GATE de revisao humana.
- Sempre PT-BR.
