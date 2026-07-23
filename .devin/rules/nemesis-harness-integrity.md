---
trigger: always_on
status: active
scope: canonical
last_updated: 2026-07-22
---

# Nemesis: Integridade do Harness (manifest de espelhamento)

> Regra canônica e compartilhada (idêntica nos dois repos irmãos). Operacionaliza a lei F10
> do método Fable: **afirmação sobre o harness em documento canônico precisa de verificador
> mecânico**. "Regras idênticas em ambos os repos" deixou de ser afirmação em prosa e virou
> manifest + procedimento de verificação. Origem: em 2026-07-09 uma auditoria real encontrou
> 2 das 3 regras compartilhadas e 8 das 9 skills divergentes entre os repos, exatamente
> porque a lei existia sem verificador.

## Verificador determinístico (procedimento, não script)

> **Origem desta decisão de design (falha→lei, 2026-07-09):** a primeira materialização do
> verificador foi um script shell em `.devin/scripts/`; o próprio Defender o quarentenou
> (visitor `nemesis_bypass`: variável de shell contendo path protegido do harness). Correto
> por design: script shell manipulando paths do Nemesis é indistinguível de tentativa de
> ofuscação. Por isso o verificador vive como PROCEDIMENTO em markdown, executado pelo agente
> como tool calls individuais (cada uma visível ao pretool), nunca como script shell.

Executar os 3 comandos abaixo (read-only, classe A). Saída vazia nos 3 = **ESPELHOS ÍNTEGROS**.
Qualquer linha de saída = **DERIVA** (ou item novo fora do manifest, que deve ser classificado).

```bash
# 1) Entre repos (rodar do diretório pai, que contém os dois repos)
diff -rq Nemesis_Defender_v0/.devin Dashboard-Nemesis-Defender/.devin \
  -x context-file-handling.md -x nemesis-repo-profile.md \
  -x prompt-refinement.md -x nemesis-global-defender.md -x ledger -x specs -x plans -x issue -x ops -x hooks.json -x scripts

# 2) Interno ao motor (rodar na raiz de Nemesis_Defender_v0)
diff -rq .devin/skills .claude/skills -x SKILL-nemesis-defender.md -x disciplina-epistemica

# 3) Interno ao dashboard (rodar na raiz de Dashboard-Nemesis-Defender)
diff -rq .devin/skills .claude/skills -x SKILL-nemesis-defender.md -x disciplina-epistemica
```

## Manifest de espelhamento

**Lei do manifest:** `.devin/` é compartilhado 1:1 entre os dois repos, e `.devin/skills/` é
espelhado em `.claude/skills/` dentro de cada repo, EXCETO as exceções declaradas abaixo.
A lista de exceções (os `-x` do procedimento) É o manifest; item novo em `.devin/` ou é
espelhado, ou entra nas exceções no mesmo diff que o cria.

### Exceções per-repo POR DESIGN (nunca espelhar, nunca acusar deriva)

- **Motor:** `.devin/rules/nemesis-repo-profile.md`, `.devin/rules/nemesis-global-defender.md`
  (canon por módulo do motor; per-repo por design, não existe no dashboard), `.devin/hooks.json`
  (scaffold do pretool, estado local da IDE). A `nemesis-fable-method.md` NÃO é exceção: é compartilhada
  (as skills espelhadas citam as leis F1..F12 nos dois repos; origem da correção: 2026-07-09,
  o livro de leis existia só no motor com referências quebradas no dashboard).
- **Dashboard:** `.devin/rules/context-file-handling.md`, `.devin/rules/nemesis-repo-profile.md`,
  `.devin/workflows/prompt-refinement.md`, `.devin/specs/`, `.devin/plans/`, `.devin/issue/`,
  `.devin/ops/`.
- **Dados locais de processo (nunca espelhar):** `.devin/ledger/` (Trust Ledger é histórico
  do repo). `Feature-Documentation/` também é per-repo.
- `AGENTS.md` e `CLAUDE.md` são per-repo (conteúdo próprio), mas AMBOS referenciam este
  manifest em vez de afirmar identidade em prosa.

### Exceções do espelho interno (`.devin/skills/` ↔ `.claude/skills/`)

`disciplina-epistemica` e `SKILL-nemesis-defender.md` NÃO são espelhadas para
`.claude/skills/`: a Claude as carrega via plugin global de skills. Intencional, não é deriva.

### Texto único, parametrizado por perfil

O texto espelhado é ÚNICO: onde a diferença de stack importa, o texto referencia o **perfil do
repo** (`.devin/rules/nemesis-repo-profile.md`) ou traz os blocos de comando dos dois perfis,
como os workflows já fazem. É proibido "adaptar" uma cópia editando-a localmente: isso recria
a deriva que esta regra existe para impedir.

## Lint estrutural de skills (procedimento, read-only)

Mesma forma do verificador de espelhamento: comandos executados como tool calls, saída
vazia = **PASS**. Roda nos mesmos gates do espelhamento. Checa a conformidade das skills
com a spec de Agent Skills: `name` do frontmatter = nome do diretório (minúsculas-hífen),
`description` presente, SKILL.md abaixo de 500 linhas (acima disso, extrair conteúdo pesado
para arquivos de referência da própria skill — disclosure progressivo).

```bash
# 4) name do frontmatter == nome do diretório (exceção do manifest excluída)
grep -H '^name:' .devin/skills/*/SKILL.md .claude/skills/*/SKILL.md \
  | grep -v '/disciplina-epistemica/' \
  | awk -F'[/:]' '{n=$NF; sub(/^ +/,"",n); if ($3 != n) print "NOME!=DIR: "$0}'

# 5) SKILL.md dentro do teto de 500 linhas
wc -l .devin/skills/*/SKILL.md .claude/skills/*/SKILL.md | awk '$1>=500 && $NF!="total"'

# 6) frontmatter com description presente
grep -L '^description:' .devin/skills/*/SKILL.md .claude/skills/*/SKILL.md
```

> **Origem (2026-07-17):** destilação de práticas externas (anthropics/skill-creator
> `quick_validate` e marketingskills `validate-skills`), adaptada à forma
> procedimento-em-markdown desta regra, por solicitação do Fernando. Na primeira execução
> o lint já encontrou e corrigiu uma não-conformidade real (`nemesis-critical-analysis`
> com `name` em maiúsculas/espaços nas cópias espelhadas).

## Quando o verificador RODA (gates)

1. **Finishing (Skill 5):** se o `git diff` da entrega toca qualquer arquivo de harness
   (`.devin/`, `.claude/skills/`, `AGENTS.md`, `CLAUDE.md`), o procedimento precisa estar
   verde ANTES de gerar a PR.
2. **Doc-sync (Skill 4.6):** mesma condição; divergência é reportada como pendência.
3. **Após qualquer edição em arquivo espelhado:** editar uma vez, propagar às demais cópias e
   rodar o procedimento na sequência (na mesma sessão).
4. **Sob demanda:** skill `nemesis-harness-sync` (verifica, apresenta a deriva arquivo a
   arquivo e reconcilia com HARD-GATE humano).

## Regras duras

- Deriva NÃO se resolve escolhendo uma versão em silêncio: em divergência de conteúdo
  material, a escolha do canônico é do Fernando (via `nemesis-harness-sync`); quando uma
  cópia é comprovadamente a versão antiga da outra, propaga-se a mais recente e reporta-se.
- É proibido criar script shell que manipule paths do harness (ver origem acima); o
  verificador é procedimento em markdown, executado por tool calls.
- Este arquivo é escaneado contra poisoning como os demais docs canônicos: descrever regras
  por conceito, nunca reproduzir sintaxe de comando perigoso, nunca isentar do scan.

## Integração

Aplicar junto com: `nemesis-fable-method.md` (F10), `nemesis-trust-ledger.md` (F11),
as invariantes do `AGENTS.md` e o SDD pipeline. Lei sem verificador é intenção; este manifest
é o verificador desta lei.
