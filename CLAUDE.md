# Nemesis_Defender_v0 — instruções para o agente (Claude)

Leia **`AGENTS.md`** (na raiz) por inteiro antes de qualquer ação. É o documento canônico do
Engenheiro Mantenedor do Nemesis (persona, invariantes de segurança, arquitetura, processo) e vale
para qualquer IDE/TUI. Este arquivo é apenas o ponteiro da Claude para ele, e — como o `AGENTS.md`
— é escaneado pelo próprio Nemesis contra adulteração; mantenha-o limpo.

Essenciais (detalhes em `AGENTS.md`): nada irreversível em manutenção (pretool desconectado; no
macOS não há contenção de kernel para te segurar); não suba o daemon durante o install; git é
exclusivamente do Fernando; 100% local.

**Disciplina epistêmica (anti-sycophancy) — regra principal:** empatia não é concordância factual;
o enquadramento do usuário não é verdade observada. Não confirme sem evidência, não trate
possibilidade como confirmação, não afirme causa-raiz sem prová-la (fonte vs binário; dev vs
distro). Auto-auditoria antes de concluir. Regra canônica: `.devin/rules/nemesis-epistemic-safety.md`.

**Método de trabalho do modelo:** `.devin/rules/nemesis-fable-method.md` (orientação antes da
ação, debugging por hipóteses, verificação antes de concluir, triagem de reversibilidade,
guarda contra contexto obsoleto e alucinação).

**Canon por módulo do motor:** `.devin/rules/nemesis-global-defender.md` — regra global (engine-only)
que destila o que cada módulo do motor é, o que faz, como convergem no produto Nemesis, e o que
pode/não pode na codebase. Leia antes de tocar em qualquer módulo. Fonte de verdade: o código em
`.nemesis/` + a doc da dashboard irmã (`../Dashboard-Nemesis-Defender/src/data/docs/`).

Processo: SDD pipeline em dois modos: `.devin/workflows/nemesis-sdd-pipeline-auto.md`
(default, 100% automatico) e `.devin/workflows/nemesis-sdd-pipeline-manual.md` (100% manual,
parada obrigatoria em cada skill). **Modo auto (default):** do input a validacao completa
E a doc-sync, sem pausas intermediarias; ao final da `nemesis-doc-sync`, PARADA UNICA
obrigatoria (relatorio + aguardar o Fernando). A `nemesis-doc-sync` roda automaticamente
como ultimo passo autonomo (a revisao das mudancas de doc acontece na PARADA UNICA); so a
`nemesis-finishing-branch` exige autorizacao explicita dele, nunca automatica. Repositorio 100% Rust
(+ C do eBPF e shell scripts herdados, infra pré-existente: herdar, não introduzir toolchain
novo).
