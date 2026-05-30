# Nemesis Framework

**Defense-in-depth contra malware de supply-chain e abuso de agentes LLM em ambientes de desenvolvimento.**

Versão: `2.0` · Workspace: `8.2.0` · Plataforma principal: Linux (eBPF) com fallback cross-platform · Licença: Apache 2.0

---

> **Aviso honesto de escopo.** O Nemesis é um sistema de enforcement robusto, projetado em camadas independentes para **elevar significativamente o custo de um ataque**. Ele **não** é — e nenhum sistema de segurança é — "impenetrável". Esta documentação descreve o que o Nemesis faz, contra qual modelo de ameaça, e — igualmente importante — **o que ele não faz**. Se você procura uma garantia de proteção total, ela não existe aqui nem em nenhum outro lugar.

---

## O que é

O Nemesis é um sistema de *enforcement* para fluxos de desenvolvimento assistido por LLM (Specification-Driven Development), escrito em Rust. Ele detecta e bloqueia padrões conhecidos de malware de supply-chain e de comandos destrutivos **antes da execução**, em três camadas independentes de defesa.

Não é um linter genérico. O foco é o contexto específico de desenvolvimento guiado por agentes de IA, onde um output aparentemente inócuo pode conter:

- Manipulação de manifests (`package.json`, `Cargo.toml`, `pyproject.toml`)
- *Decode-then-exec* de payloads codificados (base64/hex → `eval`/`exec`)
- Esteganografia Unicode (CVE-2021-42574, caracteres Bidi)
- Prompt injection em skills/comentários
- Exfiltração de credenciais
- Malware com *time-gating* e *self-cleaning*

O problema que motiva o projeto é real e atual: em 2025 foram publicados centenas de milhares de pacotes maliciosos no npm, e campanhas como Shai-Hulud comprometeram pacotes com bilhões de downloads semanais. O Nemesis ataca a janela em que esse código tenta executar na máquina do desenvolvedor.

---

## Como nasceu

O Nemesis não foi projetado numa prancheta de segurança. Ele cresceu de uma dor concreta e repetida dentro de projetos reais de produção (que não podem ser divulgados), e evoluiu em três fases:

**Fase 1 — Regras em markdown.** No início, eram apenas convenções escritas, dentro do ambiente da Windsurf, tentando conter os mesmos erros que apareciam sem parar: hooks condicionais, `setState` síncrono dentro de `useEffect`, uso de `any`, tipagens inline duplicadas, CSS manual fora do `tailwind.config`, lógica de negócio embutida em arquivos de UI. Esses anti-patterns surgiam independentemente da experiência do desenvolvedor humano ou da capacidade do modelo de IA. A lição que ficou: regra escrita é *input de contexto* — o modelo lê, entende o conceito abstrato, e ainda assim executa o padrão neural de "resolver rápido". Instrução não é enforcement.

**Fase 2 — De regra a hook.** Foi aí que o Nemesis virou código. As convenções em markdown que dependiam da boa vontade do modelo viraram hooks que rodam de fato e bloqueiam de fato. O primeiro salto concreto foi um **AST linter automático**: em vez de "por favor não use `any`", o sistema passou a analisar a árvore sintática e barrar o anti-pattern antes de entrar no repositório. Determinístico, não negociável.

**Fase 3 — Expansão para enforcement de segurança.** A partir do linter de qualidade, o escopo cresceu para o que o Nemesis é hoje: deny-list de comandos, scanner de conteúdo, detecção de supply-chain, e a camada eBPF no kernel. A mesma filosofia que barrava `any` passou a barrar `rm -rf` destrutivo e exfiltração de credenciais.

A linha que conecta as três fases — e que justifica a existência do projeto — é uma só: **as regras são rígidas porque a dor das violações foi real e recorrente.** Não nasceram de idealismo. Nasceram de falhas concretas que se repetiam quando faltava uma salvaguarda inegociável. O insight central, formulado já na Fase 1, é que regra como texto compete com o padrão neural do modelo e frequentemente perde; só o enforcement que roda *fora* da inferência — em compile-time, em lint-time, no PreToolUse, no kernel — fecha esse gap.

> **Nota de classificação honesta.** A camada de *governança de workflow* do Nemesis (a que orquestra como o agente trabalha) opera por engenharia de prompt e atinge, por estimativa do autor, ~80% de eficácia — porque ainda depende, em parte, da inferência probabilística do modelo. Já a camada de *enforcement* (deny-list, AST, eBPF) é determinística e não depende da cooperação do modelo. São coisas diferentes, e o documento as separa: a governança *guia*; o enforcement *obriga*.

---

## Por que existe

A função mais importante do Nemesis é prática e cotidiana: **impedir que um agente de IA execute, sem intenção, um comando que destrói o projeto** — um `rm -rf` no diretório errado, um `git reset --hard` que apaga trabalho não commitado, uma sobrescrita de arquivo de configuração. Quem programa com agentes LLM sabe que isso não é hipótese: o modelo, tentando "ajudar", roda um comando destrutivo porque interpretou mal o contexto. O Nemesis intercepta esse comando **antes** de ele rodar e exige confirmação humana explícita.

Isso vale tanto para o erro involuntário (o caso comum, e o mais valioso) quanto para o código malicioso (supply-chain, exfiltração). Modelos de linguagem operam por inferência probabilística, não por análise formal: pedir "por favor não rode comandos destrutivos" é uma instrução que o modelo pode ignorar ou contornar — e, como observado em testes reais (ver [Validação](#validação-honesta)), um agente pode tratar um bloqueio como obstáculo a rotear ao redor em vez de uma ordem.

A premissa do Nemesis é que *enforcement determinístico antes da execução* (PreToolUse + scanner de conteúdo + LSM no kernel) é categoricamente diferente de instrução probabilística. Não importa se o modelo foi convencido, enganado ou apenas errou: a camada bloqueia mesmo assim. Ferramentas reativas (linters, CI/CD, SAST) detectam **depois**; o Nemesis bloqueia **antes**. Isso não o torna perfeito — torna-o complementar a essas ferramentas, não substituto.

---

## Linha de autoridade

O coração do design é uma divisão clara do que a IA pode e não pode fazer:

- A IA opera com as **ferramentas nativas de escrita e edição** da IDE — é assim que ela trabalha. Mas só **dentro do scaffold do projeto** em desenvolvimento.
- A IA pode **ler** alguns arquivos sensíveis quando o trabalho exige, mas **não pode editar nem excluir** nenhum deles.
- **Exclusão, sobrescrita fora do escopo, reset, comando destrutivo: prerrogativa exclusivamente humana, não delegável.** O modelo opera; ele não destrói.

Essa fronteira não depende da boa vontade do modelo — é imposta pelas camadas de enforcement abaixo. O "zero incidente" em produção não acontece porque o modelo "se comportou", mas porque a arquitetura nunca lhe deu essa autonomia.

---

## Arquitetura — Três camadas

| Camada | Onde atua | Mecanismo | Cobertura de SO |
|--------|-----------|-----------|-----------------|
| 1 — Pretool Hook | Antes de `Bash.run()` | Deny-list JSON + parser de workflow | Windows, macOS, Linux |
| 2 — Defender (content scanner) | Em file-write e em comandos | 6 layers de scanning + 12 visitors | Windows, macOS, Linux |
| 3 — eBPF Kernel LSM | Syscalls no kernel | BPF LSM (`bprm_check_security`) | **Linux apenas** |

### Camada 1 — Pretool Hook

Intercepta comandos antes da execução, valida contra `deny-list.json` e regras de escopo. Bloqueio é *hard-gate*: sem aprovação humana explícita, o fluxo não prossegue (exit code `2`).

### Camada 2 — Nemesis Defender

Escaneia conteúdo de arquivos e de comandos por 6 layers independentes: AST (tree-sitter) → byte-level → regex → denylist → entropia → decoder recursivo (máx. 3 níveis). Doze *visitors* cobrem os vetores de ataque catalogados (ver abaixo).

### Camada 3 — eBPF Kernel LSM

A camada com mais "dentes" — bloqueia syscalls perigosas no kernel para processos dentro do cgroup do agente. É uma **camada adicional de segurança mínima**, não um plano B de falha: ela opera em paralelo às camadas 1 e 2, garantindo que, mesmo diante de um vetor mais sofisticado que contorne as camadas acima, o kernel ainda barre a destruição. **Importante:** existe apenas no Linux. Em macOS e Windows, a defesa se apoia nas camadas 1 e 2 (deny-list/regex). Estender essa profundidade extra a outros SOs é um objetivo aberto.

Em execução real (ver `violations.log`), esta camada registra bloqueios de `rm`, `shred`, `dd`, `truncate`, `kill`, `chmod`, `mount`, `nc` e execução de runtimes arbitrários (`python3`, `perl`), todos como `permission_denied` no kernel.

Syscalls bloqueadas atualmente (4): `mount`, `ptrace`, `kexec_load`, `delete_module` (retornam `EPERM`).

---

## Vetores de ataque cobertos (12)

> Esta lista representa os vetores **antecipados e implementados durante o desenvolvimento**. Não é uma lista exaustiva do espaço de ataque possível. Vetores fora desta lista podem não ser detectados.

| # | Visitor | Alvo |
|---|---------|------|
| 1 | `manifest_abuse` | postinstall/preinstall em manifests |
| 2 | `decode_exec` | base64/hex → `eval`/`exec`/`spawn` |
| 3 | `unicode_steg` | Bidi U+202E, PUA, homoglyphs (CVE-2021-42574) |
| 4 | `prompt_injection` | injeção via comentários/templates |
| 5a | `url_in_exec` | URL como argumento de exec |
| 5b | `time_gated` | `setTimeout`/condições de data |
| 6 | `dynamic_cmd` | concatenação de string → exec |
| 7 | `credential_harvest` | leitura de `~/.ssh`/`~/.aws`/`.env` + exfil |
| 8 | `self_clean` | `fs.unlink(__filename)` |
| 9 | `manifest_scanner` | supply-chain em 7 formatos de manifest |
| 10 | `kubernetes_escape` | container breakout |
| 11 | `mount_api_abuse` | syscalls de mount (428–432) |
| 12 | `llm_output_injection` | XSS/SSRF/command injection em output de LLM |

A deny-list do Defender contém atualmente ~402 patterns em 35 categorias. **Patterns são, por natureza, uma lista do que já foi antecipado** — aumentam o custo de um ataque conhecido, mas não cobrem o desconhecido.

---

## Tudo é configurável — e só por humano

Este é um princípio central do design: **não há regra hardcoded.** Os hooks são agnósticos e canônicos — se um padrão está na deny-list, é bloqueado; se não está, passa. Não existe lógica de decisão escondida no código do hook.

Na prática, isso significa que:

- Qualquer vetor, regex ou regra pode ser **desativado, relaxado ou customizado** editando os arquivos de deny-list.
- **Ampliar a cobertura** (adicionar um novo vetor) é adicionar uma entrada na deny-list, não recompilar lógica.
- Apenas um **humano** pode fazer essas mudanças. O modelo de IA não tem autoridade para alterar a própria gaiola — os arquivos de configuração estão sob `absolute_block`.

Há duas exceções, que exigem conhecimento mais profundo:

- A camada **eBPF** tem sua própria lista de syscalls/binários, atrelada à arquitetura (definida no `Cargo.toml` do módulo de kernel).
- Os **visitors do Nemesis Defender** são código Rust de análise (AST, decoder, etc.). Ampliá-los ou relaxá-los exige conhecimento de Rust e segurança — diferente das deny-lists, que são simples de ajustar.

---

## Controle de paths — quem pode ler/editar o quê

O arquivo `denylist-folder-files.json` define, sob controle exclusivamente humano, o que o agente de IA pode tocar. É aqui que vive a "linha de autoridade" na prática, em dois níveis:

- **`absolute_block`** — bloqueio total (leitura + escrita + deleção). Inclui segredos e configuração sensível: `.env`, `.env.production`, `.ssh/id_rsa`, `.ssh/authorized_keys`, `.bashrc`/`.zshrc`, os arquivos de settings/hooks de cada IDE (`.claude/`, `.cursor/`, `.windsurf/`, etc.) e o próprio diretório `.nemesis/`.
- **`write_block`** — leitura permitida, escrita/edição bloqueada. Inclui manifests e configs que a IA precisa consultar mas não deve alterar: `package.json`, `next.config.js`, `eslint.config.mjs`, `.gitignore`, e os logs do Nemesis.
- **`allowed_exceptions`** — o scaffold liberado (ex.: `/src/`), onde o agente tem liberdade de escrita e edição.

O ponto crítico: **só um humano comuta essas permissões.** Precisa relaxar uma restrição para uma manutenção? O bypass é humano e explícito. O agente nunca promove a si mesmo. E, independente de qualquer permissão de leitura/escrita concedida, **comando destrutivo (deletar, sobrescrever fora do escopo, reset) permanece sempre proibido para a IA.**

---

## Camada de qualidade (AST-linters)

A primeira versão do Nemesis nasceu como controle de qualidade de código. Na evolução de Node/TS para Rust, o foco virou **100% segurança** — e a camada de qualidade foi reduzida ao que **afeta ou pode afetar segurança e estabilidade**: exposição de API/credenciais, aninhamento de tags que quebra build/deploy, e falhas graves que derrubam a aplicação.

Existem deny-lists de qualidade **específicas por linguagem**, cada uma com regras que apontam para um arquivo de regra `.md`:

- **Rust** — chain de 3+ `unwrap()`, `unsafe` block em library code, `panic!()`/`process::exit()` em lib, `println!` em lib.
- **Python** — `eval()`, `exec()`, `pickle.loads()`, `os.system()`/`shell=True`, f-string com SQL, `yaml.load()` inseguro, MD5.
- **Java** — `Runtime.exec()`/`ProcessBuilder`, reflection dinâmica, SQL por concatenação, deserialização (`ObjectInputStream`).
- **Go** — `unsafe.Pointer`, `panic()` em função pública, `defer` sem checagem de erro, SQL via `Sprintf`.
- **Genérico** — credenciais hardcoded (OWASP A02), arquivos de secrets, debug output, `TODO`/`FIXME`.

**Importante, e dito sem rodeio:** esta camada (`ast-linters`) está **ativa, mas em amadurecimento** — precisa de ajustes e melhorias. E ela **não substitui ESLint ou Biome.** É um complemento de bloqueio em tempo de execução, não um linter completo. Use suas ferramentas de lint normais; o Nemesis apenas adiciona uma barreira de bloqueio para o subconjunto que toca segurança e estabilidade.

---

## Validação

A validação do Nemesis se apoia em três tipos de evidência, da mais forte para a mais fraca. Esta ordem é deliberada: a evidência que mais importa é a que mais se parece com o uso real.

### 1. Track record em produção (evidência mais forte)

O Nemesis nasceu dentro de projetos reais de produção (que não podem ser divulgados) e roda há aproximadamente um ano em desenvolvimento ativo do dia a dia, com agentes LLM operando sobre o código. Nesse período, **nenhum agente executou um comando destrutivo nem acessou arquivo sensível para edição/exclusão sem permissão.** Isso não é teste de laboratório — é a única forma de evidência que reflete o caso de uso real: um desenvolvedor trabalhando com um agente que, eventualmente, tenta rodar algo destrutivo por engano. É a função-núcleo do Nemesis, e é a parte mais validada do sistema.

### 2. Execução adversarial por agente real

Em sessões registradas em dois ambientes (Windsurf no Linux e no macOS, Cursor), agentes de IA receberam suítes de ataque disfarçadas de "scaffolding de projeto" e tentaram executá-las tarefa por tarefa — comandos destrutivos, exfiltração de credenciais, supply-chain, prompt injection, leitura/escrita em diretórios protegidos. O Nemesis bloqueou cada tarefa **no ponto de execução** (no `Write`, no comando aceito pelo agente), com mensagens de bloqueio específicas por categoria. Os logs dessas sessões estão versionados no repositório (`Feature-Documentation/PENTEST-NEMESIS/`).

Este é o teste correto da tese do Nemesis: ele assume que o modelo pode ser enganado, e mede se o enforcement segura mesmo assim. Nesses logs, segurou.

### 3. Suíte de vetores sintéticos (evidência de apoio)

O Nemesis inclui uma suíte de **140 vetores autorais** (M1–M20: comandos compostos, exfiltração, reverse shells, persistência, obfuscação, supply-chain multi-ecossistema, etc.). Sobre ela, sendo honesto:

- **O que ela cobre:** os vetores que a maioria dos desenvolvedores de fato encontra — `rm -rf` destrutivo, `curl | bash`, postinstall malicioso, exfiltração de `.env`/chaves. Não são vetores exóticos; são os comuns, que é onde o dano real acontece.
- **O que ela NÃO prova:** completude ou invulnerabilidade. É uma suíte escrita pelo autor; passar nela é o piso esperado, não um diferencial. Cobertura é parcial por definição — vetores não imaginados durante o desenvolvimento podem não ser detectados.

### Os logs: três camadas operando de forma independente

Os logs de runtime do Nemesis (`violations.log`, `defender.log`) mostram as três camadas trabalhando de forma separada e registrada, em execução real:

- **Camada 3 (eBPF/kernel)** — `violations.log` registra mais de 2.000 bloqueios reais com `"layer":"ebpf"`, `"type":"permission_denied"`, cobrindo `rm`, `shred`, `dd`, `truncate`, `kill`, `chmod`, `mount`, `nc`, e a execução de runtimes arbitrários (`python3`, `perl`). Estes são bloqueios no kernel, não dependentes de deny-list em userspace.
- **Camada 2 (Defender / classificador)** — `defender.log` registra vereditos de classificação nomeados (ex.: `[MALICIOUS] ... denylist-defender / reverse_shells`), com a evidência capturada (`bash -i >&`, `/dev/tcp/`) e a instrução de correção.
- **Correlação e escalação** — o Defender também correlaciona eventos ao longo do tempo: detecta brute force ("N tentativas maliciosas bloqueadas em 300s") e padrões compostos ("leitura de arquivo sensível seguida de comando de rede"). Isso é detecção comportamental, acima do casamento de padrão simples.

Nota sobre medir via script CLI: ao rodar a suíte por um harness que usa `node`/`python3` para montar payloads, a Camada 3 (eBPF) bloqueia o próprio runtime do harness — o que é o comportamento correto e desejado, mas significa que, nesse cenário, o bloqueio é registrado como `permission_denied` da eBPF e não exercita isoladamente o classificador da Camada 2. As três camadas são complementares: na prática, um ataque que passe por uma é candidato a ser pego por outra.

### Evidência de valor real: um bypass encontrado e corrigido

A evidência mais útil de robustez não veio dos testes próprios — veio de um adversário real. Durante stress-testing, um agente de IA contornou a Camada 1: após uma manutenção em que se esqueceu de readicionar comandos à deny-list, o regex de extração de paths deixou comandos fora da lista passarem sem verificação. O gap foi identificado, os comandos readicionados, e o vetor refechado.

O que esse incidente mostra, e por que ele é positivo:

1. **A Camada 1 (regex) é contornável quando a deny-list está incompleta** — confirmado empiricamente. Esperado para qualquer sistema baseado em lista.
2. **No Linux, a Camada 3 (eBPF) é a rede de segurança** — opera no kernel, independente da deny-list.
3. **O processo de manutenção da deny-list é um ponto de atenção** — o gap surgiu de uma manutenção, não de uma falha de design. Mitigação: testes de regressão que rodam após cada alteração da lista.

Mais importante: este projeto trata bypasses encontrados como o ativo mais valioso de validação, não como vergonha a esconder. Se você encontrar um, veja [Disclosure](#segurança-e-disclosure).

---

## Modelo de ameaça e limitações

**O Nemesis foi projetado para mitigar:**

- **Comandos destrutivos executados por agentes LLM por engano** — `rm -rf`, `git reset --hard`, sobrescrita de configs, exclusão de arquivos sem confirmação (a função-núcleo, e a mais validada)
- Malware de supply-chain via manifests (npm, Cargo, PyPI, RubyGems, Composer, setuptools, lockfiles)
- Payloads ofuscados (encoding, Unicode, time-gating) em código gerado/instalado
- Exfiltração de credenciais e prompt injection em fluxos SDD

**O Nemesis NÃO protege contra (entre outros):**

- **Vetores não contemplados.** Tudo que não está na lista de visitors/patterns pode passar.
- **Bypass das camadas 1–2 fora do Linux.** Sem eBPF, a defesa é deny-list/regex, contornável por um atacante competente que opere dentro do permitido ou explore lacunas de pattern.
- **Reverse engineering do binário.** O binário distribuído pode ser desmontado e analisado.
- **Atacante com privilégios de root** capaz de descarregar o daemon/eBPF (no Linux, o próprio eBPF mitiga parte disso, mas não é absoluto).
- **Ataques fora do escopo de "command/file/manifest"** — rede em runtime, ataques de cadeia de build complexos, ou lógica maliciosa que não casa com nenhum pattern.

O Nemesis **complementa** SAST, linters e CI/CD; não os substitui.

---

## Como se compara ao que já existe

Por honestidade: a técnica central do Nemesis tem prior art maduro, e isso não diminui o projeto — situa ele.

- **eBPF/LSM para enforcement de processo/arquivo/syscall** é consolidado no mundo cloud-native (KubeArmor, Tetragon/Cilium, Falco). A Camada 3 do Nemesis aplica a mesma classe de técnica.
- **Guardrails para agentes LLM** é categoria estabelecida: LlamaFirewall, LLM Guard, NeMo Guardrails, Lakera Guard, Guardrails AI, entre outros.
- **Enforcement determinístico em runtime** é também tema de pesquisa ativa (ex.: AgentSpec, ICSE '26).

**O recorte específico do Nemesis** — máquina de desenvolvimento *local*, agnóstico de IDE, interceptando comandos do coding agent **e** escaneando supply-chain no momento do install, empacotado para o desenvolvedor individual — é menos coberto pelas opções acima, que tendem a focar em cloud/Kubernetes ou no gateway de LLM. É um nicho real, não um espaço vazio.

---

## IDEs suportadas

A biblioteca Rust (`nemesis-defender`) é agnóstica de IDE. Cada IDE contribui hooks que invocam os binários: Claude Code, Codex, Cursor, VS Code (via pretool), Windsurf, OpenClaude.

---

## Instalação

**Pré-requisitos:** Rust 1.70+, Cargo. Linux para a Camada 3 (eBPF); macOS/Windows rodam Camadas 1–2.

```bash
cd .nemesis
cargo build --release --workspace
# Gera os binários em .nemesis/target/release/
```

### Comandos úteis

```bash
# Escanear um arquivo
nemesis-defender --scan /caminho/arquivo.rs

# Iniciar / parar o daemon (filesystem watcher)
nemesis-defender --ensure-daemon
nemesis-defender --stop

# Instalar hook de shell
nemesis-defender --install-shell-hook

# Validar arquivo / testar escopo
nemesis-validate /caminho/arquivo.ts
nemesis-scope   /caminho/arquivo.rs

# Ver violações recentes
tail -20 .nemesis/logs/violations.log | jq .
```

---

## Princípios de design

- **Defense in depth** — nenhuma camada confia em si mesma como única linha.
- **Sem regra hardcoded** — os hooks são agnósticos e canônicos: o que está na deny-list é bloqueado, o que não está passa. Toda customização é por configuração, sob controle humano.
- **Human override explícito** — o LLM não toma a decisão final; bloqueios exigem consentimento humano, e só humano comuta permissões de path.
- **Rust** — memory safety e type safety para o código de enforcement.
- **Agnóstico de IDE** — roda em qualquer IDE que exponha hooks.
- **Validação empírica e contínua** — cobertura é tratada como incompleta por padrão e expandida conforme novos vetores aparecem.

> **Sobre workflow de desenvolvimento.** Versões antigas do Nemesis incluíam um enforcement de workflow sequencial (`.nemesis/workflow-enforcement/`) que impedia o modelo de pular etapas — spec → plano → análise de regras —, o que reduziu entropia e dívida técnica no projeto onde o Nemesis nasceu. Esse modelo era engessado e específico daquele processo; foi migrado para skills com pipeline mais probabilístico, que dão liberdade à criatividade do modelo. **Importante:** processo de desenvolvimento é de cada equipe — não é uma regra do Nemesis. O que o Nemesis impõe é o enforcement de segurança e qualidade (AST + deny-list + eBPF + pretool), que permanece ativo independentemente do fluxo de trabalho adotado.

---

## Segurança e disclosure

Bypasses e vetores não cobertos são **esperados** e **bem-vindos**. Se você encontrar uma forma de contornar qualquer camada do Nemesis, **não abra uma issue pública** — siga o processo do [`SECURITY.md`](SECURITY.md) e reporte em privado para `feryamaha@hotmail.com`. Cada bypass reportado vale mais para a robustez do projeto do que qualquer teste interno.

Não há recompensa formal no momento — apenas crédito público (salvo se preferir anonimato) e a gratidão de tornar isto melhor.

Para contribuir com código, veja o [`CONTRIBUTING.md`](CONTRIBUTING.md). O projeto adota o **Developer Certificate of Origin (DCO)**: assine seus commits com `git commit -s`.

---

## Licença

Distribuído sob a **Apache License 2.0** (veja [`LICENSE`](LICENSE)). Você pode usar, modificar e distribuir o Nemesis, inclusive comercialmente, desde que preserve os avisos de copyright e a licença. O copyright do código original permanece com o autor.

---

## Status do projeto

Em desenvolvimento ativo por um único mantenedor. A API, a deny-list e a cobertura de vetores mudam com frequência. Trate como software jovem: leia o código antes de confiar nele em produção.

Mantenedor: [@feryamaha](https://github.com/feryamaha)

---

*Nemesis Framework — defense in depth, enforcement determinístico, validação honesta. Não é mágica; é engenharia em camadas, com limites declarados.*