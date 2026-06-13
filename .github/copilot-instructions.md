# Nemesis_Defender_v2.0 — instruções para o agente (GitHub Copilot / VS Code)

Leia **`AGENTS.md`** (na raiz) por inteiro antes de qualquer ação. É o documento canônico do
Engenheiro Mantenedor do Nemesis (persona, invariantes de segurança, arquitetura, processo).
Este arquivo é apenas o ponteiro do Copilot para ele, e — como o `AGENTS.md` — é escaneado pelo
próprio Nemesis contra adulteração; mantenha-o limpo.

Essenciais (detalhes em `AGENTS.md`): nada irreversível em manutenção (pretool desconectado; no
macOS não há contenção de kernel para te segurar); não suba o daemon durante o install; prove, não
suponha; git é exclusivamente do Fernando; 100% local. Processo: SDD pipeline em
`.devin/workflows/nemesis-sdd-pipeline.md`.
