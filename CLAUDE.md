# Nemesis_Defender_v2.0 — instruções para o agente (Claude)

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

Processo: SDD pipeline em
`.devin/workflows/nemesis-sdd-pipeline.md`. Repositório 100% Rust (+ C do eBPF, infra de kernel
pré-existente).
