---
trigger: always_on
status: active
scope: repo-local
repo: Nemesis_Defender_v0
last_updated: 2026-07-22
---

# Nemesis: regra global do motor (canon por módulo)

> **O que é esta regra.** É o mapa canônico "o que cada módulo do motor é, o que faz, como
> converge no produto, e o que pode / não pode ao mexer nele". É a **regra global do Nemesis
> Defender** pedida pelo Fernando: o lugar único onde qualquer modelo aprende o motor ANTES de
> tocar em código, para errar menos por falta de contexto.
>
> **Regra per-repo POR DESIGN (perfil motor; NÃO espelhada para a dashboard).** Descreve os
> módulos Rust/eBPF do motor, que não existem no repo irmão. Declarada como exceção no manifest
> de `nemesis-harness-integrity.md`. Referenciada por `AGENTS.md` §8 e por `CLAUDE.md`.
>
> **Este arquivo é escaneado contra poisoning como os demais docs canônicos.** Descreve regras e
> classes de ataque **por conceito e por nome de categoria**, nunca reproduzindo a sintaxe de um
> comando destrutivo. Nunca o isente do scan.

---

## 0. Como usar esta regra — anti-alucinação (a razão de ela existir)

O Fernando observou que modelos, mesmo dentro desta codebase, pulam etapa, não seguem regra,
alucinam e não checam a fonte. Esta seção é o antídoto e tem precedência sobre todo o resto.

**Fonte de verdade, nesta ordem (onde divergirem, o de cima manda):**
1. **O código em `.nemesis/`** — o código manda, sempre. Número, comportamento, path: confirme no
   arquivo antes de afirmar.
2. **A doc interativa da dashboard** (`../Dashboard-Nemesis-Defender/src/data/docs/`, construída por
   análise forense e "verificada contra o repositório"): conceito, por que, arquitetura, diagramas.
   Trilha em `onboarding/` (10 conceitos + `visao-geral` + `cobertura` = conceito→arquivo-fonte).
3. **`AGENTS.md` / `nemesis-repo-profile.md` / `nemesis-fable-method.md`** — invariantes, comandos,
   método. Se um deles divergir do código, o **código vence** (ver §6; foi assim que se descobriu que
   o `AGENTS.md` dizia 4 camadas quando são 3).

**Disciplina obrigatória (herda `nemesis-epistemic-safety.md` e §2A do `AGENTS.md`):**
- Não afirme número, contagem ou comportamento de módulo **sem abrir o arquivo-fonte**. Cite o path.
- Distinga **método** (um visitor é um método de detecção) de **cobertura** (o coeficiente, a soma
  das camadas). NUNCA reduza a proteção a "N visitors" (ver §3 e `AGENTS.md` §3A).
- Distinga **fonte vs binário** (`.nemesis/target/release/` dev vs `.nemesis/bin/` distro) e **dev vs
  distro**. Resolva path pelo ancestral `.nemesis`, nunca por profundidade fixa.
- Leis transversais NÃO são reescritas aqui (evita deriva, lei F10): invariantes de segurança em
  `AGENTS.md` §2; coeficiente em §3A; Rust "o que praticamos / o que não" em §7; comandos de
  validação e regras de linguagem em `nemesis-repo-profile.md`; método/anti-alucinação em
  `nemesis-fable-method.md`. **Esta regra é a CAMADA DE MÓDULO**, e aponta para elas.

---

## 1. O que é o Nemesis (síntese destilada da dashboard)

Enforcement **determinístico e ativo** contra malware de supply-chain e abuso de agentes LLM,
que bloqueia o ataque **no momento da execução**, 100% na máquina do desenvolvedor. Rust, AGPL-3.0.

- **A tese:** instrução probabilística não contém; enforcement determinístico contém. Um agente
  ignora um pedido educado no prompt; não ignora um processo que termina com `exit 2` antes da ação.
- **A cadeia de autoridade que o motor preserva:** HUMANO → NEMESIS → AGENTE DE IA → SISTEMA.
  Inteligência não implica autoridade. O modelo **opera, não destrói**: exclusão, sobrescrita fora
  de escopo, reset e comando destrutivo são prerrogativa exclusivamente humana, não delegável.
- **A origem da ação é irrelevante:** o gate barra o destrutivo venha de prompt injection, de
  desalinhamento do modelo, ou de erro humano. Não tenta ler intenção; bloqueia a ação.

---

## 2. As 3 camadas (são TRÊS, não quatro)

Defesa em profundidade = camadas **independentes**: se uma falha ou está em manutenção, a próxima
ainda contém. O `AGENTS.md` §3 listava "4 camadas" tratando **fail-closed** como camada; fail-closed
é uma **postura** (ver §5), não uma camada. São três (fonte: dashboard `arquitetura`, `visao-geral`,
`9-defesa-profundidade-decisao`; e o código citado abaixo).

| # | Camada | Onde/quando | O que faz | Fonte |
|---|---|---|---|---|
| 1 | **Pretool / Posttool hook** | userspace, síncrono, write/exec-time | Intercepta a ação ANTES de executar; traduz a ferramenta da IDE em intenção; valida denylist + escopo; nega com `exit 2`. **É o ponto de entrada: sem pretool, o Nemesis não funciona.** Aciona a trilha do Defender. O PostTool audita depois (nunca bloqueia). | `hooks/nemesis-pretool-check-unix.rs` (`:190-194` bloqueio; `:312-347` tradução) |
| 2 | **Defender + daemon (Iron Dome)** | userspace, assíncrono, filesystem | O "cérebro": `scan_content` escaneia conteúdo e `compute_severity` (função pura) dá o veredito Clean/Suspicious/Malicious. O daemon vigia o filesystem e, ao **confirmar** hostilidade, **move** o arquivo para `.nemesis/quarantine/` (preserva, não deleta), trava a sessão para revisão humana; reversível por `restore`/`purge`. **Security-only** (nunca trata qualidade, nunca deleta). | `nemesis-defender/src/lib.rs` (`scan_content`, `compute_severity`), `quarantine.rs` |
| 3 | **eBPF / BPF-LSM** | kernel, **Linux-only**, opt-in | Backstop **independente do pretool**: nega exec de comando bloqueado no gancho `bprm_check_security` (`-EPERM`) e conexão fora da allowlist de egress em `socket_connect`; escopo por cgroup do agente; Landlock sem root (`no_new_privs`). Vale mesmo com o pretool desconectado. | `ebpf-kernel/ebpf/nemesis-block.bpf.c`, `src/{loader,egress,landlock,config}.rs` |

**Dois fatos que amarram as camadas:**
- **Camadas 1 e 2 partilham o MESMO motor** `scan_content` → veredito idêntico para bytes idênticos.
  A fonte única das regras de conteúdo é a `denylist-defender.json`, **embutida no binário** via
  `include_str!` (não editável, não fica no disco).
- **macOS/Windows não têm camada de kernel.** É design, não lacuna: no macOS, com o pretool
  desconectado, **nada contém** (não assuma proteção que não existe — `AGENTS.md` invariante 2).

---

## 3. Decisão: severidade, corroboração e o coeficiente

- **`compute_severity` (função pura, determinística)** — `nemesis-defender/src/lib.rs:377-437`.
  Tier A (confirmatório): **1 sinal basta** para Malicious (ex.: `decode_exec`, `reverse_shell`,
  `exfil_chain`, `taint_tracker`, `unicode_bidi`, `ide_config_poisoning`, `nemesis_bypass`).
  Tier B (corroborante, sujeito a FP): conta **tipos distintos**; `0 → Clean`, `1 tipo → Suspicious`
  (loga e mantém), `2+ tipos → Malicious`. **Conta tipos, não hits** (várias ocorrências da mesma
  causa não escalam).
- **Reversibilidade e custo assimétrico:** deixar passar é ruim, apagar código legítimo é pior. Por
  isso: só evidência forte age sozinha; a fraca precisa de companhia; e a ação é reversível (bloquear
  escrita / mover para quarentena), nunca deletar.
- **O coeficiente (REGRA CANÔNICA, `AGENTS.md` §3A):** a proteção é a **soma das camadas** e das
  taxonomias de cada uma (denylist embutida, heurísticas de scanner, visitors AST, denylists de
  comando, allowlist de egress), **não** a contagem de uma feature. **Visitor é método, não vetor.**
  NUNCA declare "N vetores = N visitors" nem invente um número agregado sem taxonomia auditável. A
  **prova empírica** da cobertura é a suíte de pentest estático (gate de CI `self-audit`).

---

## 4. Mapa dos módulos — o canon (path · o que é/faz · o que NÃO fazer)

> Workspace Cargo em `.nemesis/` (`Cargo.toml`): **5 membros** — `ast-linters`, `ebpf-kernel`,
> `nemesis-defender`, `nemesis-doctor`, `nemesis-publisher` — mais o **pacote raiz `nemesis`
> v8.2.0**, que produz os binários de hook. (A dashboard diz "4 membros": ver §6, o código manda.)

| Módulo (path em `.nemesis/`) | O que é / faz | Camada | O que NÃO fazer ao mexer |
|---|---|---|---|
| `hooks/` | Bins do pacote raiz `nemesis`: `nemesis-pretool-check-unix.rs`, `pretool-hook.rs`, `nemesis-posttool-check-unix.rs`, `pre-edit-hook.rs`, variantes windows/debug. Ponto de entrada (Camada 1), tradução ferramenta→intenção, `exit 2`, fail-closed. | 1 | Só em **manutenção coordenada pelo Fernando** (invariante 12; flag `maintenance_mode_required`). Nunca introduzir caminho que "passe" em erro (quebra o fail-closed). |
| `nemesis-defender/src/` (crate) | O motor: `lib.rs` (`scan_content`, `compute_severity`, exclusões), `watcher/daemon.rs` (quarentena, isenções por nome), `quarantine.rs`, `violations_log.rs`, `pid.rs`, CLI (`--scan`, `--daemon`). Camadas 1+2 core. | 1+2 | Não duplicar regra de conteúdo fora da fonte única; não `unwrap()` em caminho de input não-confiável; daemon é **security-only** (regra de qualidade nunca chega nele). |
| `nemesis-defender/src/scanner/` | Pipeline de conteúdo (8 arquivos): `byte_scanner.rs` (BiDi/PUA/homoglyph/zero-width), `entropy.rs` (Shannon), `regex_layer.rs` (denylist embutida), `manifest_scanner.rs`, `decoder.rs` (recursivo, máx 3), `ast_scanner.rs` (tree-sitter), `denylist_loader.rs`, `allowlist_loader.rs`. | 2 | A ordem real está em `lib.rs`, não no comentário de `mod.rs` (ver §6). Manter limites de DoS (profundidade/tamanho). |
| `nemesis-defender/src/visitors/` | Detecção de **intenção semântica** por travessia de AST: **15 arquivos** — `credential_harvest`, `decode_exec`, `dynamic_cmd`, `exfil_chain`, `ide_config_poisoning`, `manifest_abuse`, `nemesis_bypass`, `persistence_patterns`, `prompt_injection`, `python_import_injection`, `self_clean`, `taint_tracker`, `time_gated`, `unicode_steg`, `url_in_exec` (+ `mod.rs`). | 2 | Visitor é **método**, não unidade de cobertura (ver §3). Não contar visitors como "vetores". |
| `nemesis-defender/config/denylist-defender.json` | **Regras de conteúdo EMBUTIDAS no binário** via `include_str!` (`denylist_loader.rs`). Tamper-proof, não fica no disco. ~**37 categorias** (`reverse_shells`, `data_exfiltration_compound`, `persistence_mechanisms`, `supply_chain_registry`, `prompt_injection_*`, `nemesis_evasion`, `pii_detection`, ...). Fonte única das regras de segurança. | 2 | Endurecer regra = mudança de fonte revisada por humano; nunca duplicar fora daqui; confirmar a contagem no arquivo antes de citá-la. |
| `ebpf-kernel/` (crate, Linux) | `ebpf/nemesis-block.bpf.c` (`bprm_check_security`, `socket_connect`, mapas hash/ringbuf/array/LPM), `src/loader.rs`, `egress.rs`, `landlock.rs`, `config.rs`, `denylist-ebpf/commands.toml`. Backstop de kernel. | 3 | `unsafe` legítimo vive no C do eBPF, não no Rust de userspace. `cargo test` do `nemesis-ebpf-kernel` exige `--release`. Não sobrepor decisão anterior da cadeia LSM (`ret != 0`). Não commitar `.bpf.o`. |
| `nemesis-doctor/` (crate) | Diagnóstico/observabilidade. `src/checks/`: `compile` (G1), `unit_tests` (G2), `ebpf`, `daemon`, `scaffold`, `inventory`, `pentest` (G7). `--quick` pula os pesados (G1/G2/G7). Manual canônico de operação: `NEMESIS-OPERATIONS.md`. | — | Rodar `--quick` ANTES de reconectar o pretool (`AGENTS.md` §9). Não inventar rótulo de grupo fora dos nomeados. |
| `nemesis-publisher/` (crate) | Publicação da **telemetria mínima opt-in** (ping anônimo install/uninstall, UUID opaco; sem dado de máquina/projeto). **5º membro do workspace.** | — | Nenhum dado do usuário sai da máquina fora deste opt-in decidido pelo Fernando (invariante 7). Ampliar telemetria exige decisão explícita dele. |
| `ast-linters/` (crate) | Camada de **qualidade de código** — hoje **silenciada** (em desenvolvimento, afinando falso-positivos antes de validar como proteção). | (extra) | Não tratar como camada de segurança ativa; qualidade nunca vira ação do daemon. |
| `denylist/` | Denylists de **comando EDITÁVEIS pelo usuário**: `deny-list.json`, `deny-list-base.json`, `deny-list-generic.json`, `deny-list-quality.json`, `denylist-folder-files.json`. Isentas do scan do daemon. | 1 | Não confundir com as regras EMBUTIDAS (`denylist-defender.json`); estas ficam no disco por design. |
| `denylist-customers/` | **A ÚNICA superfície que o humano edita para relaxar**, por sua conta e risco: `allowlist-customers.jsonc` / `.json`, `allowlist-ebpf.toml`. | 1/3 | Relaxa (allowlist), nunca endurece; override absoluto — tratar com cuidado. |
| `quarantine/` | Estado de runtime da quarentena (`PENDING.json` + arquivos movidos + `meta.json`). Enquanto houver pendência, o pretool trava a sessão. | 2 | Não editar à mão; resolver por `restore`/`purge` via CLI. Não versionar conteúdo. |
| `forensics/` | Auditoria de conteúdo externo: `incoming/` + `scan-incoming.sh` → APROVADO/REPROVADO. **Isenta da quarentena do daemon** (`daemon_quarantine_exempt`): escaneia/loga mas não move nem trava. | — | `incoming/` e relatório não são versionados; é triagem, não garantia — sempre ler + revisar. |
| `install/` | Instalador: `nemesis-install.sh`, `nemesis-uninstall.sh`, `com.nemesis.publisher.plist`, `nemesis-publisher.service`, `info-install.txt`. Isento por nome. | — | **NUNCA subir o daemon durante o install** (invariante 3): ele quarentenaria o próprio instalador. Validar por `--scan`, não pelo pretool. |
| `scripts/` | Shell herdado: `ensure-ebpf-caps.sh` (setcap por-inode após build release), `nemesis-build.sh`. | — | Script shell **não pode manipular path do harness em variável** (o Defender quarentena por design — visitor `nemesis_bypass`). Herdar, não introduzir toolchain novo. |
| `lsp/` | `nemesis-lsp.rs` (bin do pacote raiz): diagnósticos no editor via LSP. | — | — |
| `runtime/` | Estado de runtime: `defender.pid`, `session-events.jsonl`. | 2 | Não editar à mão; é estado, não config. |
| `telemetry/` | `identity.json` (UUID opaco), `publisher.env`, `sync-state.json`, `doctor-full.json`. | — | Mínima e opt-in; sem dado de máquina/projeto (invariante 7). |
| `target/` | Saída de build. Dev: `target/release/`; distro: `bin/`. | — | Resolver pelo ancestral `.nemesis`. **Nenhuma cópia de binário para outro path** (`repo-profile` regra 6). |
| `logs/nemesis-violations.log` | **Ledger único** de violações. | todas | Não isentar; não fabricar entradas. |
| `pentest-nemesis-control/` | Suíte de **pentest estático** por classe de ataque (módulos M); gate de CI `self-audit`; `nemesis-defender/run-pentest.sh` (contra binário release; `FAIL=0` + `STATUS: APROVADO`). | prova | Não fixar contagem de testes (ver §6); o script auto-detecta. |

---

## 5. Qualidade de código para o motor funcionar (do / don't)

> Fonte primária: `AGENTS.md` §7 ("o que PRATICAMOS / o que NÃO praticamos") e as 6 regras de
> linguagem de `nemesis-repo-profile.md`. Não repetir aqui; abaixo só o essencial ligado a módulo.

**PODE / DEVE:** só Rust NOVO em `.nemesis/` (herdar C do eBPF e shell pré-existentes, não introduzir
toolchain novo); resolução de path por ancestor-walk; `include_str!` para regra crítica embutida;
`Result`/`Option` + `match` exaustivo; defaults **fail-closed**; validar por crate
(`cargo check -p <crate>`); `--release` obrigatório no `nemesis-ebpf-kernel`; manter warnings em zero.

**NÃO PODE:** `unwrap()`/`expect()` em caminho de input não-confiável; profundidade fixa de `.parent()`
para achar `.nemesis`; duplicar regra de conteúdo fora de `denylist-defender.json`; `unsafe` novo no
Rust de userspace; dependência de rede/serviço externo; reproduzir literal de comando destrutivo em
doc escaneado; **isentar** arquivo de config do scan; `cargo build --release` sem autorização (só a
Skill 4.5 tem autorização intrínseca); **git de escrita** (exclusivo do Fernando); tocar `hooks/` fora
de manutenção coordenada; **mudar escopo** de spec/implementação por conta própria (invariante 13).

---

## 6. Divergências conhecidas — o código manda, não invente para "consertar"

Estas divergências são reais e estão catalogadas na `cobertura` da dashboard. Ao encontrá-las, cite a
fonte e o número do arquivo; não harmonize inventando.

- **Membros do workspace:** `Cargo.toml` lista **5** (inclui `nemesis-publisher`); a dashboard
  (`visao-geral`) diz "4 membros". **Código manda: 5** + pacote raiz `nemesis` v8.2.0.
- **Camadas:** são **3** (pretool, daemon, eBPF). O `AGENTS.md` §3 dizia 4 tratando fail-closed como
  camada — corrigido; fail-closed é postura.
- **"18 visitors":** o diretório tem **15 arquivos** de visitor (+ `mod.rs`). Nomes como
  `reverse_shell` e `manifest_registry_redirect` são **sinais emitidos** (em regex/decoder/manifest),
  não arquivos de visitor. E contar visitors não mede cobertura (§3).
- **Ordem do pipeline:** o comentário de `scanner/mod.rs` diz seis camadas; a execução real em
  `lib.rs` tem mais chamadas (byte com quatro sub-scans, mais `ide_config` e `exfil_chain` antes da
  AST). **`lib.rs` manda.**
- **Contagem de pentest:** diverge entre fontes (`run-pentest.sh` "184/26"; `self-audit.yml` e
  `AGENTS.md` "194/194"). O script auto-detecta; **não fixar número** (defasa).
- **SLSA / doctor:** attestation declarada no `release.yml` (não fixar linhas); doctor nomeia G1/G2/G7,
  demais grupos = módulos de `checks/`.

---

## 7. Integração com o pipeline de desenvolvimento

Esta regra é carregada no **pré-flight** do SDD pipeline (`nemesis-sdd-pipeline-auto.md` /
`-manual.md`). O **roteador de módulo** do pré-flight mapeia os paths que a spec vai tocar para a
seção de módulo correspondente aqui (§4), carrega o contexto e as guardas daquele módulo no contrato
de handoff, e — para os módulos-jóia (`hooks/`, `pentest-nemesis-control/`) — eleva o trabalho à
camada de maior raciocínio. O histórico por módulo (manutenção, melhoria, problema) vive nos ledgers
de `.devin/ledger/modules/`. Cross-reference: `AGENTS.md` §8 (mapa "onde mexer") aponta para esta
regra como o canon de módulo.
