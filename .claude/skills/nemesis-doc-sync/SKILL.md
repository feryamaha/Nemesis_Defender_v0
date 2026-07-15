---
name: nemesis-doc-sync
description: >
  Trata documentacao como FEATURE. APOS a validacao (Skill 4.5) e ANTES do finishing (Skill 5),
  analisa o git diff da mudanca e decide se a superficie de doc do perfil (motor: README.md e
  release notes em Feature-Documentation/release-note/; dashboard: src/data/docs/*.json) precisa
  ser atualizada. Se nao, segue o fluxo. Se sim, reconcilia (codigo = verdade, regra do
  coeficiente, sem inserir por inserir). Roda automaticamente como ultimo passo autonomo; a
  PARADA UNICA obrigatoria (HARD-GATE humano) acontece no FIM dela, antes do finishing.
  Garante que a PR sempre contenha a documentacao sincronizada. Quando a mudanca e publicavel
  e a validacao fechou PASS, monta o combo de release (nova versao ou atualizar/re-tag) como
  TEXTO para o Fernando executar manualmente (o modelo nunca executa git).
---

# Nemesis Doc Sync (documentacao como feature)

> **Texto unico espelhado nos dois repos.** A superficie de doc vem do perfil:
> **motor** = `README.md` + release notes em `Feature-Documentation/release-note/`;
> **dashboard** = JSONs de docs em `src/data/docs/`.

## ULTIMO PASSO AUTONOMO (roda automaticamente, e a PARADA UNICA vem no fim dela)

Esta skill **faz parte da fase autonoma do pipeline** (modo auto): a `nemesis-tests` (4.5)
a invoca automaticamente, sem pausa. **A PARADA UNICA obrigatoria acontece no FIM desta
skill** (Fase 4), depois da doc-sync e antes do finishing. E nesse ponto que o Fernando
revisa o relatorio consolidado (incluindo as mudancas de doc, se houver) e decide entre
finalizar a entrega (`nemesis-finishing-branch`) ou gerar novas issues e reiniciar o ciclo
(PDCA). Apenas a `nemesis-finishing-branch` exige autorizacao explicita dele.

**Anuncio de inicio**: "Estou usando a skill nemesis-doc-sync para verificar se a mudanca exige atualizar a documentacao."

**Pre-requisito**: Skill 4.5 (`nemesis-tests`) concluida — codigo validado, suite do perfil
verde.

## Por que existe

Documentacao errada deixa o codigo em **check-mate**: o usuario confia na doc, nao no codigo.
Por isso a doc e tratada como **feature** e tem um passo proprio no pipeline, **apos a
validacao e antes do finishing**, rodando automaticamente — assim a **PR sempre inclui a
atualizacao de doc**, e nunca mais "atualiza codigo e esquece a documentacao".

## Processo

### Fase 1: Coletar o que mudou (git diff real)
```bash
git diff --stat
git diff
```
Read-only (git de escrita e exclusivo do Fernando). O diff e a fonte do que mudou.

### Fase 2: GATE DE DECISAO — a mudanca afeta a documentacao publica?

Confronte o diff contra o que a superficie de doc do perfil DOCUMENTA. Checklist de itens
documentados que podem ser afetados:
- **Contagens citadas:** categorias da denylist embutida, modulos M do pentest, crates,
  hooks BPF-LSM, visitors (arquivos vs despachados).
- **Coeficiente / camadas:** pretool denylists, eBPF, denylist embutida, heuristicas de
  scanner, visitors AST.
- **Plataformas suportadas** (Linux/macOS/Windows) e o que cada camada cobre.
- **Nomes de arquivos/comandos user-facing** (harness, `run-pentest.sh`, binarios, install).
- **Feature nova ou comportamento mudado** que o usuario percebe.
- **Enquadramento do pentest** (numeros, gate, modulos).
- **Passos de instalacao / requisitos.**
- **Release note (motor):** a mudanca adiciona feature, fecha bypass, altera config de
  enforcement, muda empacotamento/build ou comportamento user-facing? Se sim, verificar se
  uma nova release note e necessaria ou se uma existente em
  `Feature-Documentation/release-note/` precisa ser atualizada. Release notes sao por-versao;
  mudanca que sera publicada em release nova = nova release note. Bugfix interno sem impacto
  user-facing geralmente NAO exige release note.
- **Harness (F10):** a mudanca tocou arquivos do harness (`.devin/`, `.claude/skills/`,
  `AGENTS.md`, `CLAUDE.md`)? Se sim, rodar o procedimento de espelhamento de
  `nemesis-harness-integrity.md` e reportar o resultado; deriva = pendencia a resolver
  antes do finishing.

Para CADA item afetado, emita um veredito:
- **NAO PRECISA** — o diff nao toca nada documentado, OU a doc ja reflete. Justifique em 1 linha.
- **PRECISA** — liste exatamente o que ficou divergente (doc vs codigo), com `arquivo:linha`.

> **Regra dura: nao inserir doc por inserir.** Bugfix interno, refactor, mudanca de teste etc.
> geralmente NAO exigem atualizacao. So atualize o que a mudanca tornou divergente.

### Fase 3a: Veredito NAO PRECISA
Reporte "a doc ja reflete a mudanca; nada a atualizar" e siga para a Fase 4 (PARADA UNICA).

### Fase 3b: Veredito PRECISA — reconciliar (codigo = verdade)

Atualize a superficie de doc do perfil com disciplina:
- **Codigo e a fonte de verdade.** Verifique cada numero/fato no codigo antes de escrever
  (nao invente).
- **Regra do coeficiente (AGENTS.md secao 3A do motor):** a protecao e a soma das camadas;
  visitor e **metodo**, nao a unidade de cobertura; nao publique "N vetores = N visitors"
  nem numero agregado nao rastreavel.
- **Sem numero fragil:** prefira descrever por modulo/camada + gate (ex.: "M1..Mn, FAIL=0")
  a cravar um total que a proxima mudanca defasa.
- **Cirurgico:** mude so o que ficou divergente; nao reescreva secoes inteiras sem
  necessidade.
- **Cross-repo:** a doc publica de conceito/onboarding vive nos JSONs do dashboard
  (`src/data/docs/`); o README tecnico vive no motor. Cada doc-sync cuida da superficie do
  proprio repo; se a mudanca afeta a superficie do repo irmao, reporte ao Fernando (nao
  edite o outro repo a partir deste fluxo).

- **Release note (motor):** se a mudanca exige release note, crie ou atualize o arquivo em
  `Feature-Documentation/release-note/` seguindo o padrao das notas existentes (natureza da
  versao, o que mudou, o que nao mudou, validacao, limites). O numero da versao e decisao do
  Fernando.

Apresente o diff das mudancas de doc.

### Fase 3c: Handoff de release (combo git para o Fernando executar MANUALMENTE)

> **INVARIANTE ABSOLUTA (AGENTS.md invariante 4): o modelo NUNCA executa git de escrita.**
> Esta fase apenas MONTA o combo de comandos como TEXTO para o Fernando copiar e rodar no
> terminal nativo dele. Nenhum `git add/commit/push/tag` e executado pelo agente, jamais, nem
> "para testar". Comandos de LEITURA (`git tag -l`, `git log`, `git status`, `git branch`)
> sao permitidos e usados so para montar o combo correto.

**Quando roda:** SEMPRE que a validacao (Skill 4.5) fechou PASS. O combo e entregue para o
**Fernando decidir se e quando publicar** — nao e o modelo que decide se "vale" publicar. O
veredito PRECISA/NAO PRECISA da doc (Fase 2) NAO suprime o combo: mesmo com doc "NAO PRECISA",
o combo e montado e entregue (o Fernando pode querer subir a mudanca mesmo sem alterar doc).
Unico caso de pular: a validacao reprovou algo (ai a Fase 3c reporta "nao recomendo publicar,
pendencia em X" e NAO monta combo).

**Passos:**

1. **Confirmar que pode subir.** So oferecer o combo se a validacao (Skill 4.5) fechou tudo
   PASS (check + testes + build + pentest FAIL=0). Se algo reprovou ou ficou pendente,
   **NAO** entregar combo de release: reportar "nao recomendo publicar release, pendencia em
   X" e seguir para a Fase 4. O combo pressupoe "aprovado, testado e validado".

2. **Ler o estado de git (read-only)** para escolher o cenario correto:
   ```bash
   git tag -l                 # tags/versoes que ja existem
   git log --oneline -5       # ultimos commits (padrao de mensagem)
   git branch --show-current  # branch de push (o combo usa 'main' por padrao)
   ```

3. **Determinar o cenario e recomendar (o Fernando decide a versao):**
   - **NOVA VERSAO RELEASE** — a versao-alvo NAO existe ainda em `git tag -l`. Primeira
     publicacao daquela versao.
   - **ATUALIZAR RELEASE (re-tag)** — a versao-alvo JA existe como tag/release e o objetivo e
     republicar a correcao na MESMA versao (apaga a tag local+remota e recria). Usar quando a
     mudanca e um fix dentro de uma versao ja publicada.
   O agente RECOMENDA o cenario com base no `git tag -l`, mas a versao e a escolha (nova vs
   atualizar) sao **decisao do Fernando**.

4. **Entregar o combo preenchido** (mensagem de commit sugerida no padrao kebab-case do repo,
   ex.: `fix-...`, `feat-...`; versao e mensagem de tag confirmadas pelo Fernando). Modelos
   canonicos:

   **ATUALIZAR RELEASE** (mesma versao, apaga e recria a tag):
   ```bash
   git add .
   git commit -m "<mensagem-kebab-case>"
   git push origin main
   git tag -d <vX.Y.Z>
   git push origin --delete <vX.Y.Z>
   git tag -a <vX.Y.Z> -m "Nemesis Defender <vX.Y.Z>"
   git push origin <vX.Y.Z>
   ```

   **NOVA VERSAO RELEASE**:
   ```bash
   git add .
   git commit -m "<mensagem-kebab-case>"
   git push origin main
   git tag -a <vX.Y.Z> -m "Nemesis Defender <vX.Y.Z> - <descricao-curta-da-versao>"
   git push origin <vX.Y.Z>
   ```

5. **Declarar explicitamente** no fim do combo: "Git e exclusivamente seu (invariante 4).
   Este combo e TEXTO para voce executar manualmente no terminal nativo; nao rodei nenhum
   comando git. Confirme a versao e a mensagem antes de rodar."

Se ha placeholders que dependem do Fernando (numero da versao, mensagem final), deixa-los
marcados com `<...>` e apontar o que ele precisa preencher. Nunca inventar a versao.

### Fase 4: PARADA UNICA obrigatoria (HARD-GATE humano, fim da fase autonoma)

Esta e a **PARADA UNICA do pipeline**: depois da doc-sync, antes do finishing. Passos:

1. **Trust Ledger (lei F11):** invocar `nemesis-trust-ledger-update` para gravar todos os
   vereditos do ciclo (gates da Skill 0 P1/P2, rule-control, resultado da Skill 4.5,
   reconciliacoes) e o veredito desta doc-sync (PRECISA/NAO PRECISA), append-only em
   `.devin/ledger/trust-ledger.md`.
2. **Relatorio consolidado:** emitir o relatorio da PARADA UNICA no formato do workflow
   (`nemesis-sdd-pipeline-auto.md`): spec, plano, git diff real, tabela de validacao com
   saidas literais, decisoes tomadas, achados fora de escopo, veredito da doc-sync (e o diff
   das mudancas de doc, se houve), secao Trust Ledger, gate F10 (harness). **Se a Fase 3c
   gerou combo de release**, inclui-lo aqui (cenario recomendado + comando pronto como texto),
   com o lembrete de que git e execucao manual do Fernando.
3. **BLOQUEAR e aguardar o Fernando.** Documentacao e feature: as mudancas de doc (se
   houve) sao revisadas por ele aqui, junto do resto. **NAO invocar `nemesis-finishing-branch`
   automaticamente** — ela so executa com autorizacao explicita dele. Respostas validas para
   avancar ao finishing: "sim", "pode", "aprovado", "ok", "prossiga".

## Saida

- Veredito (PRECISA / NAO PRECISA) e, se PRECISA, as mudancas de doc aplicadas (incluindo
  release note, se aplicavel), apresentadas na PARADA UNICA para revisao do Fernando.
- Se a mudanca e publicavel e a validacao fechou PASS: o **combo de release** (cenario nova
  versao OU atualizar/re-tag) montado como TEXTO para o Fernando executar manualmente.
- Resultado do check de espelhamento do harness, quando aplicavel.
- Trust Ledger do ciclo gravado; relatorio consolidado emitido; pipeline PARADO aguardando
  o Fernando decidir entre finishing ou reiniciar o ciclo (PDCA).

## Integracao

- **Skill anterior**: `nemesis-tests` (4.5), que invoca esta skill automaticamente (sem pausa).
- **Proxima skill (SO com autorizacao explicita do Fernando na PARADA UNICA)**:
  `nemesis-finishing-branch` (5) — a PR ja inclui a documentacao sincronizada no git diff.

## Lembrar

- Documentacao = feature. Doc errada = check-mate.
- Roda automaticamente apos a Skill 4.5; a PARADA UNICA vem no FIM desta skill (Fase 4).
- GATE de decisao ANTES de editar (nao inserir por inserir).
- Codigo = verdade; regra do coeficiente; doc do perfil sincronizada.
- Cross-repo: cada doc-sync cuida da superficie do proprio repo.
- Mudanca em harness = rodar o procedimento de espelhamento (F10).
- Release note (motor): mudanca publicavel = verificar se exige nova release note.
- Validacao PASS = montar SEMPRE o combo git (nova versao OU atualizar/re-tag) como TEXTO,
  independentemente do veredito de doc; o Fernando decide se e quando publicar. O modelo
  NUNCA executa git, so entrega os comandos (invariante 4).
- Git e exclusivo do Fernando. Finishing so com autorizacao explicita do Fernando.
- Sempre PT-BR.
