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

Processo: SDD pipeline em dois modos: `.devin/workflows/nemesis-sdd-pipeline-auto.md`
(default, 100% automatico) e `.devin/workflows/nemesis-sdd-pipeline-manual.md` (100% manual,
parada obrigatoria em cada skill). **Modo auto (default):** do input a validacao completa
sem pausas intermediarias; ao final, PARADA UNICA obrigatoria (relatorio + aguardar o
Fernando). `nemesis-doc-sync` e `nemesis-finishing-branch` so executam com autorizacao
explicita dele, nunca automaticamente. Repositorio 100% Rust
(+ C do eBPF e shell scripts herdados, infra pré-existente: herdar, não introduzir toolchain
novo).
