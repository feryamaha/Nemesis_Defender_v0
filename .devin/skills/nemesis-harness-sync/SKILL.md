---
name: nemesis-harness-sync
description: >
  Verifica e reconcilia o espelhamento do harness (lei F10): roda o procedimento
  deterministico de nemesis-harness-integrity.md, apresenta a deriva arquivo a arquivo e
  reconcilia. Divergencia material tem HARD-GATE humano (o Fernando escolhe o canonico);
  copia comprovadamente antiga propaga-se a mais recente com reporte.
---

# Nemesis Harness Sync (lei F10)

Verificar e reconciliar os espelhos do harness entre os dois repos e entre
`.devin/skills/` e `.claude/skills/`.

> **Texto unico espelhado nos dois repos.** O manifest e o procedimento de verificacao vivem
> em `.devin/rules/nemesis-harness-integrity.md` (fonte unica; nao duplicar aqui).

**Anuncio de inicio**: "Estou usando a skill nemesis-harness-sync para verificar e reconciliar os espelhos do harness."

## Quando invocar

- Apos qualquer edicao em arquivo espelhado (mesma sessao).
- No gate F10 do finishing (Step 1.5) quando ha deriva a reconciliar.
- Sob demanda do Fernando (auditoria periodica).

## Processo

### Step 1: Verificar (procedimento da regra canonica)

Executar os 3 comandos do procedimento de `nemesis-harness-integrity.md` (diff entre repos
com as exclusoes do manifest; diff interno `.devin/skills` vs `.claude/skills` em cada
repo). Saida vazia nos 3 = **ESPELHOS INTEGROS**: reportar e encerrar.

### Step 2: Classificar cada divergencia (com evidencia, nao suposicao)

Para cada arquivo divergente ou ausente:

1. **Ausencia simples** (existe em um espelho, falta no outro): classificar como propagacao
   pendente.
2. **Versao antiga comprovada**: uma copia e comprovadamente a versao anterior da outra
   (evidencia: conteudo e superconjunto com emendas datadas, frontmatter mais novo,
   git log do arquivo). Classificar como propagacao da mais recente.
3. **Divergencia material**: as copias tem conteudo genuinamente diferente (adaptacao local,
   emendas paralelas). Classificar como decisao humana.
4. **Item fora do manifest**: arquivo novo em `.devin/` que nao e espelhado nem exceção
   declarada. Classificar como pendencia de manifest (ou espelha, ou entra nas exceções).

### Step 3: Reconciliar

- **Classes 1 e 2:** propagar (copiar a versao canonica/mais recente para os espelhos),
  listando cada copia feita. Reportar o que foi propagado e por que (evidencia da classe).
- **Classe 3 (HARD-GATE):** apresentar ao Fernando o diff das versoes com a analise em 1
  linha por diferenca; BLOQUEAR ate ele escolher o canonico. So entao propagar.
- **Classe 4:** propor a atualizacao do manifest (em `nemesis-harness-integrity.md`, arquivo
  espelhado: a emenda segue o fluxo de emenda de lei, HARD-GATE humano) ou o espelhamento
  do arquivo. Nao decidir sozinho.

### Step 4: Re-verificar e registrar

1. Re-executar o procedimento do Step 1: precisa retornar **ESPELHOS INTEGROS**.
2. Registrar no Trust Ledger (`nemesis-trust-ledger-update`, evento `harness`): resultado,
   arquivos reconciliados, decisoes do Fernando se houve.

## Formato de saida

```
## Harness Sync (F10)

Verificacao inicial: [INTEGROS | deriva em N arquivos]

| Arquivo | Classe | Acao |
|---|---|---|
| [path] | [1-4] | [propagado motor→dash | HARD-GATE | manifest] |

Re-verificacao: [INTEGROS]
Ledger: [entrada gravada]
```

## Regras duras

- NUNCA "resolver" divergencia material escolhendo uma versao em silencio.
- NUNCA adaptar texto espelhado localmente (a parametrizacao e via perfil do repo).
- Propagacao e sempre por copia integral do arquivo canonico (nao merge manual criativo).
- Git e exclusivo do Fernando; esta skill so toca arquivos, nunca faz git de escrita.
- Sempre PT-BR.

## Integracao

**Regra canonica**: `.devin/rules/nemesis-harness-integrity.md` (manifest + procedimento).
**Invocada por**: gate F10 do `nemesis-finishing-branch`, `nemesis-doc-sync`,
`nemesis-postmortem-to-law` (emendas espelhadas), ou o Fernando.
**Skill relacionada**: `nemesis-trust-ledger-update` (registro do resultado).
