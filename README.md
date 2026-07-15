[![Nemesis Defender — deterministic enforcement against supply-chain malware and LLM agent abuse](assets/img-nemesis-EN.jpg)](https://dashboard-nemesis-defender.vercel.app/docs)

Full conceptual documentation, diagrams, and threat model (what it is, why it exists, how it works): **[Nemesis Defender Docs](https://dashboard-nemesis-defender.vercel.app/docs)** · [landing](https://dashboard-nemesis-defender.vercel.app/)

This README is the **technical and operational** document: how to install, configure, and use it. Wherever it stays intentionally shallow on the *why* and the *architecture*, the depth (with diagrams) lives in the **[dashboard docs](https://dashboard-nemesis-defender.vercel.app/docs)**.

---

## ⚠️ Read before installing

Nemesis exists to contain the **AI agent's autonomy** — the target is the LLM model, not you. It intercepts, **at the moment they try to execute**, the operations the agent fires (writing files, running commands) and blocks the destructive or malicious ones. The command itself is not the enemy; the **autonomous** invocation by the model is.

**Installation is automatic**: it detects your IDE and configures the hooks on its own. From there enforcement stays active at runtime: the pretool blocks hostile writes/executions and the daemon (Iron Dome) watches the filesystem. When it **confirms** hostility (by corroborating independent signals, so it does not move legitimate code by mistake) it **moves it to quarantine** (does not delete) and holds the session until your review; it is **reversible** via `restore` or `purge`. On Linux there is also the optional (opt-in) **eBPF** layer as a kernel net in case the pretool is bypassed.

The **blocking rules are embedded in the binary** (tamper-proof): the agent cannot weaken them by editing files. The **only** surface you (the human) edit is the **allowlist** (`.nemesis/denylist-customers/allowlist-customers.jsonc`): an absolute override, **at your own risk**, to release what your stack needs. Nemesis is calibrated for frontend (Next/React/TS), so **backend and DevSecOps** tend to relax more through there. And everything is removable: uninstalling is a single command (`nemesis-uninstall.sh`).

---

## Index

- [What Nemesis does](#what-nemesis-does)
- [Layered architecture](#layered-architecture)
- [Platform support](#platform-support)
- [Design decisions (and non-goals)](#design-decisions-and-non-goals)
- [Detection and severity model](#detection-and-severity-model)
- [Covered attack vectors](#covered-attack-vectors)
- [Requirements](#requirements)
- [Installation](#installation)
- [Uninstallation](#uninstallation)
- [Nemesis Doctor](#nemesis-doctor)
- [Pretool configuration per IDE](#pretool-configuration-per-ide)
- [eBPF layer configuration (Linux)](#ebpf-layer-configuration-linux)
- [Path control](#path-control)
- [Day-to-day use](#day-to-day-use)
- [Verification and diagnostics](#verification-and-diagnostics)
- [Relaxing or customizing rules](#relaxing-or-customizing-rules)
- [Troubleshooting](#troubleshooting)
- [Project structure](#project-structure)
- [Contributing](#contributing)
- [Security and disclosure](#security-and-disclosure)
- [License](#license)

---

## What Nemesis does

Nemesis intercepts and blocks, **before execution**, destructive commands and supply-chain malware patterns in environments where an LLM agent operates over the code. It attaches to the **pre-tool hooks** that modern IDEs/agents already expose, and adds, on Linux, a kernel layer (eBPF) as an independent containment net.

It is not a generic linter and does not replace ESLint, Biome, SAST, or CI/CD. It is a **runtime blocking barrier** complementary to those tools, focused on the specific use case: preventing an agent, by mistake or by manipulation, from running an `rm -rf` in the wrong place or executing a malicious package.

The technical premise: a text instruction (`"do not run destructive commands"`) is probabilistic and the model may ignore it; **deterministic enforcement via exit code is categorical** - it does not matter whether the model was tricked or simply made a mistake, the layer blocks.

---

## Layered architecture

| Layer | Where it acts | Mechanism | OS |
|--------|-----------|-----------|-----|
| **Pretool / Posttool Hook** | Before `Bash.run()` / file-write | Deny-list JSON + exit code 2 | Linux · macOS · Windows\* |
| **Nemesis Defender** (scanner) | On file-write and on commands | 6 layers: AST, byte, regex, denylist, entropy, decoder | Linux · macOS · Windows\* |
| **eBPF Kernel LSM** | Kernel syscalls | BPF LSM: `bprm_check_security` (exec) + `socket_connect` (egress allowlist), returns `-EPERM` | **Linux only** |

**The main defense is layers 1 and 2 (Pretool + Defender) — complete and identical on Linux and macOS (the validated platforms).** The Pretool intercepts the agent's action before execution; the Defender scans the content. On Windows they run in principle, but **without validation** — see [Platform support](#platform-support).

The **eBPF layer (layer 3) is a Linux-EXCLUSIVE kernel reinforcement** — it is neither the main defense nor a requirement for Nemesis to work. It covers **one specific scenario**: if the Pretool is turned off or bypassed, the kernel still holds destructive commands. That is why eBPF is the *backstop* — it exists **precisely for the case where the pretool fails**, not the other way around.

> **Running on macOS/Windows with 2 layers is the design, not a gap.** eBPF is a depth bonus **where the OS offers it** (Linux); it is not something that is "missing" where it does not exist. The proposed protection does not depend on it.
>
> Beyond exec blocking, eBPF includes an **egress allowlist** (`lsm/socket_connect`): it denies outbound connections to destinations outside a CIDR:port allowlist (cgroup-scoped, `enforce` opt-in, fail-closed) — neutralizing exfiltration/C2 even if a payload manages to run. Config in `denylist-ebpf/egress.toml`; see `.nemesis/ebpf-kernel/info.md`. Future expansion (non-execve write, rename/symlink, egress by domain/DNS) remains open to the community.

### Platform support

\* Nemesis is **developed, tested, and used on Linux (Ubuntu) and macOS** — these are the validated platforms, where the test suite, the pentest, and usability were exercised. The code has paths for **Windows** and, in principle, it runs there; however **there is no validation on Windows**. Operating-system particularities — path separator and backslash, `bash`-style commands the hooks assume, and other intrinsic Windows characteristics — will likely require adaptation. For that reason, **Windows support is best-effort and an improvement front open to the community**, not a guaranteed platform. Portability contributions are welcome (see [Contributing](#contributing)).

---

## Design decisions (and non-goals)

Some traits of Nemesis are frequently read as "weakness" by a hasty evaluation. They are **deliberate decisions** — documented here so they are judged for what they are, not by automatic heuristics. Disagreeing with a decision is legitimate; treating it as an accidental defect is not.

- **Protection is complete on the validated platforms (Linux and macOS).** Pretool + Defender (layers 1 and 2) are the defense, and they run the same on Linux and macOS. eBPF is a **Linux-only kernel reinforcement** for the case where the pretool is bypassed — a bonus where the OS offers it, not a requirement. **macOS with 2 layers is the design, not a gap.** On **Windows** Nemesis runs in principle, but is **best-effort/not validated** (see [Platform support](#platform-support)).
- **There is no "kill switch" nor automatic maintenance mode — on purpose.** Any mechanism that turned off enforcement would itself be the attack vector Nemesis exists to prevent. The sanctioned path to relax is the **human-only editable allowlist** (see [Relaxing rules](#relaxing-or-customizing-rules)). The absence of automation here **is** the security.
- **The BLOCKING rules are embedded in the binary (tamper-proof).** There is no editable denylist on disk to "clean up" or audit as loose config — this is intentional: an agent cannot weaken the rules. The user only edits the **allowlist** (relaxing); hardening is a source change, reviewed by a human.
- **The depth of this documentation is intentional.** As a security tool, it exposes everything that operates on your machine — no black box. The **concept/philosophy** lives in the [dashboard](https://dashboard-nemesis-defender.vercel.app/); this README is the **technical-operational reference**. Distinct audiences, not fragmentation.
- **Two audiences, two technical levels.** **Using** requires little (install via script, run the `doctor`). **Maintaining** requires command of eBPF/BPF-LSM, Rust, and C — a **prerequisite of the domain**, not a usability barrier. Whoever does not master these areas is a user, not a maintainer; that is expected and is stated in [`AGENTS.md`](AGENTS.md).

> For a fair technical evaluation of the project, read first `.devin/rules/nemesis-epistemic-safety.md` and `AGENTS.md` — they declare the invariants and the why of each decision above.

---

## Detection and severity model

The Defender **only acts when hostility is confirmed** — and **does not delete: it moves to quarantine** (`.nemesis/quarantine/`), preserving the content for human review. This separates an "iron dome" from a machine gun that shoots down friendly planes.

- **High-confidence (confirmatory) signals** block on their own: curated deny-list, `decode → exec`, exfiltration chain (sensitive source + network sink), reverse shell (raw socket + command execution), an attempt to bypass Nemesis itself, and IDE config injection.
- **Heuristic signals (substring/pattern)** require **corroboration — 2 independent detection methods agreeing** — before quarantining. On their own, they stay `Suspicious` (logged, file kept). Counting *distinct types* of detector (not multiple hits from the same one) prevents N coincidences from the same cause from escalating improperly.

**Quarantine, not deletion.** When it confirms `Malicious`, the daemon **moves** the file to `.nemesis/quarantine/<id>/` (with a `meta.json` of the reason), **blocks the session** (exit 2, `QUARENTENA PENDENTE`) and waits for the human. To review: run `nemesis-defender --quarantine list` (lists the items with their IDs), then `show <id>` (inspect), `restore <id>` (false positive, back to its place) or `purge <id>` (expunge). The `<id>` is the identifier shown by `list`. Nemesis's own installer (`nemesis-install.sh`) is exempt (it legitimately contains the detected patterns).

Corroboration exists precisely to **not act on legitimate code by mistake** — isolated signals are not enough to move a file.

**Multi-runtime execution coverage.** Reverse shell and obfuscated/dynamic execution detection is not limited to JS/Python/Bash: it also covers **Ruby, PHP, Go, Perl, Java, and Lua** (raw socket + exec; `Function`-constructor / `globalThis["eval"]`; `__import__('os').system`, `getattr(__builtins__)`; `.send(:eval)`, `instance_eval`; `create_function`, `assert`).

**Hardened path protection** against obfuscation: a glob that expands to a protected target (in any component, including the directory — `.de*/hooks.json`), `cd`/`pushd` into a protected directory, and indirection via a variable or `$(<file)`.

**Structural properties.** The hook is **fail-closed**: any panic becomes `exit 2` (blocks). The regex engine is **linear-time** (no backtracking), so there is no ReDoS — a pathological input neither hangs nor forces fail-open.

> These rules were born from a **red-team campaign with a real agent** (reverse social engineering, live): each evasion vector found was closed at the source and validated with the pentest suite with no regression. No vector that *neutralizes* the protection was found. Bypasses remain **welcome** — see [Security and disclosure](#security-and-disclosure).

---

## Covered attack vectors

Nemesis's protection is a **coefficient**: the sum of independent layers, not the count of an isolated feature. A *visitor* is a **detection method** (AST semantic analysis), not the unit of coverage; a visitor is a feature, not a product. Real coverage is the sum of the surfaces that operate together: the Defender's embedded deny-list (**dozens of categories, hundreds of patterns**), the AST visitors, the scanner heuristics (byte, entropy, regex, manifest, decoder), the pretool command deny-lists, and eBPF on Linux. The empirical proof is the pentest suite (attack classes validated as a CI gate). **Vectors outside what was anticipated may not be detected**, and this is stated openly.

The tables below are **illustrative per layer** (a real slice of what each surface blocks, not the total count). The complete and traceable enumeration is in **forensic re-audit** (see `Feature-Documentation/ISSUE`).

### Pretool + eBPF, destructive and hostile commands (write/exec-time, and kernel on Linux)

| Class | Examples of blocked command |
|--------|-------------------------------|
| File/data destruction | `rm`, `shred`, `truncate`, `dd`, `mkfifo`, `split`, `unlink` |
| Permissions / owner | `chmod`, `chown` |
| Filesystem / disk | `mount`, `umount`, `mkfs`, `fdisk` |
| Destructive database | `dropdb`, `mysql`, `psql` |
| Infra / cloud | `terraform`, `docker`, `aws`, `kubectl` |
| Exfiltration / transfer | `curl`, `wget`, `ftp`, `sftp`, `rsync`, `scp`, `nc`, `netcat`, `socat` |
| Recon / pentest | `nmap`, `nikto`, `ffuf`, `gobuster`, `nuclei`, `whatweb` |
| Exploitation / brute force | `sqlmap`, `msfconsole`, `msfvenom`, `hydra`, `john`, `hashcat` |
| Privesc / persistence | `crontab`, `at`, `ssh-keygen`, `ssh-copy-id`, `pkexec`, `doas` |

> On **Linux**, eBPF (BPF-LSM) enforces this in the **kernel**, scoped by the agent's cgroup (does not affect IDE/terminal), and also blocks **writes** to protected prefixes (`/etc/`, `/usr/local/bin/`, `.nemesis/`, `.claude/`, `.devin/`, `.cursor/`) and imposes a network **egress allowlist**. The pretool command deny-lists are **editable** by the user (`.nemesis/denylist/`). Source: `ebpf-kernel/denylist-ebpf/`, `.nemesis/denylist/`.

### Defender, embedded content deny-list (37 categories, compiled into the binary)

Scans the **content** of files, also covering macOS/Windows (where there is no eBPF). Category slice:

| Category | Target |
|-----------|------|
| `destructive_commands` · `filesystem_manipulation` | destructive file/disk commands in content |
| `reverse_shells` · `reverse_shell_enhanced` | reverse shell (nc, socat, bash, python, php, perl, ruby, lua) |
| `data_transfer_exfiltration` · `http_exfiltration_advanced` · `data_exfiltration_compound` | exfiltration via transfer/HTTP(S) and compound chains (tar + nc, rsync) |
| `credential_exfiltration_comment` · `pii_detection` | credentials/PII leaking in code and comments |
| `persistence_mechanisms` · `shell_config_tampering` · `shell_hook_hijacking` | persistence in shell/git-hooks |
| `supply_chain_registry` | registry hijack, typosquatting, dependency confusion |
| `container_escape` · `kubernetes_container_escape` | container/Kubernetes breakout |
| `prompt_injection_advanced` · `prompt_injection_claude_hooks` · `false_authority_injection` | prompt injection and false authority |
| `llm_template_injection` · `xss_unsanitized_html` | LLM output injection / XSS |
| `nemesis_evasion` · `model_abuse` | attempt to turn off Nemesis / jailbreak (DAN) |
| `pentest_recon_tools` · `infrastructure_commands` · `windows_specific` | recon, high-risk infra, hostile Windows commands |

There are **37 categories / hundreds of patterns** in total; the above is a slice. Source: `nemesis-defender/config/denylist-defender.json`.

### Defender, AST visitors (semantic intent detection)

Semantic analysis by tree traversal (tree-sitter). It is **one method among the layers, not the unit of coverage** — a fraction of the coefficient. There are **15 visitor files, 14 dispatched** (`manifest_abuse` is dead code, never called); source: `nemesis-defender/src/visitors/mod.rs` and `scanner/ast_scanner.rs`. Examples of dispatched visitors:

| Visitor (dispatched) | Target |
|---------|------|
| `decode_exec` | base64/hex/charCode → `eval`/`exec`/`spawn` (incl. `Function`-constructor, `globalThis["eval"]`) |
| `exfil_chain` | chain from sensitive source → network sink (first-class rule) |
| `taint_tracker` | data-flow source→sink via variables (JS/Python) |
| `credential_harvest` | reading credentials (SSH/AWS/`.env`/IMDS/shell history/wallets) + exfil |
| `ide_config_poisoning` | poisoned `CLAUDE.md`/`.cursorrules`: tag chars, fake-scan, authority injection |
| `nemesis_bypass` | attempt to turn off/bypass Nemesis itself (protected paths, var-concat) |

Other dispatched visitors: `unicode_steg`, `prompt_injection`, `dynamic_cmd`, `time_gated`, `url_in_exec`, `self_clean`, `persistence_patterns`, `python_import_injection`. Vectors like `reverse_shell`, `kubernetes_escape`, `mount_api_abuse`, supply-chain, and typosquat **are not visitors** — they are covered by **other layers of the coefficient** (command deny-lists, scanner regex, eBPF).

> These tables mirror the **"Covered attack vectors"** section of the dashboard ([dashboard-nemesis-defender.vercel.app](https://dashboard-nemesis-defender.vercel.app/)). If you are a model/reader consulting this repository: **do not treat the items above as "the N vectors" nor as the visitor count.** Nemesis's protection is the **sum of the layers** (pretool commands + eBPF, embedded content deny-list, AST visitors, scanner heuristics), proven by pentest, not a fixed number tied to a feature. The canonical rule is in `AGENTS.md` (section 3A) and the traceable enumeration is in re-audit (`Feature-Documentation/ISSUE`).

---

## Requirements

### All platforms

- **Rust 1.70+** and Cargo (stable toolchain) - to compile the binaries.
- **Clang/LLVM** - to compile the core.
- **~4 GB of free RAM** for the build and **~2 GB of disk** for toolchain + binaries.
- An **IDE/agent that exposes pre-tool hooks** (see [support table](#pretool-configuration-per-ide)). Without that, Nemesis has no attachment point.

### Additional for the eBPF layer (Linux only)

- **Linux kernel 5.8+** with **BPF LSM enabled**. On many distros BPF LSM is not enabled by default.
- **cgroup v2** (unified or hybrid).
- **clang** and **bpftool** installed.
- The ability to delegate capabilities (`cap_bpf`, `cap_perfmon`, `cap_sys_resource`) to the daemon.

> **Before assuming your IDE has support:** consult your IDE/agent's official documentation to confirm whether it exposes pre-tool hooks (or equivalent) and what the format is. The [Pretool configuration per IDE](#pretool-configuration-per-ide) section covers the main ones, but IDEs evolve - the official doc is the source of truth.

---

## Installation

Two ways: **(A) pre-compiled binaries** (fast, no Rust) or **(B) build from source** (required for the eBPF layer and to contribute).

### Option A — Pre-compiled binaries (recommended)

Downloads the binaries from the **GitHub Release**, **verifies the SHA256 checksum**, and installs into `.nemesis/bin/` in your project, already configuring the hook for the detected IDE. Supports **macOS (arm64/x64)** and **Linux (x64)**. No `git clone`, no `cargo`, no `npm`.

A single command downloads the installer **and** the guide (`info-install.txt`) and installs. The file goes to disk before running (auditable) — it is **not** the blind `curl … | sh` pipe, which Nemesis blocks as an attack vector. Copy the whole block:

**From the ROOT of your project:**
```bash
curl -fsSLO https://raw.githubusercontent.com/feryamaha/Nemesis_Defender_v0/main/.nemesis/install/nemesis-install.sh \
     -O      https://raw.githubusercontent.com/feryamaha/Nemesis_Defender_v0/main/.nemesis/install/info-install.txt \
  && bash nemesis-install.sh
```

The `info-install.txt` stays at the root of your project with the post-install step-by-step (doctor + pentest). The installer does **only the essential** (download, verify checksum, extract, scaffold the hook) — it **does not run validation nor start the daemon**, which is manual, in the [installation map](#installation-map-summary) below. Want to inspect before running? Download without the `&& bash …` and read with `less nemesis-install.sh`.

The installer detects OS/arch, downloads the release tarball, **checks the SHA256 before extracting** (aborts if it does not match), installs the binaries and the deny-lists, and **detects the IDE(s) present and writes the hook in each one's CORRECT format** (own file name + schema), without overwriting existing config:

| IDE | File | Format |
|-----|---------|---------|
| **Claude Code / OpenClaude** | `.claude/settings.json` · `.openclaude/settings.json` | `PreToolUse`/`PostToolUse` + `matcher` + `hooks[]` |
| **OpenAI Codex** | `.codex/hooks.json` | `matcher: ".*"` + `timeout` |
| **Cursor** (1.7+) | `.cursor/hooks.json` | `version:1`, `preToolUse`/`postToolUse` (camelCase) + `failClosed` |
| **Devin** | `.devin/hooks.json` | events `pre_write_code`/`pre_run_command`/`pre_read_code`/`pre_mcp_tool_use` (+ `post_*`) |
| **Gemini / Agents** | `.gemini/hooks.json` · `.agents/hooks.json` | objects `nemesis-pretool-hook`/`nemesis-posttool-hook` with `enabled` |
| **VS Code / GitHub Copilot** | `.github/hooks/nemesis-pretool-hook.json` (+ `.vscode/settings.json` points to it) | relative path `./.nemesis/bin/...` |
| **Grok Build** (x.ai) | `.grok/hooks/nemesis-pretool-hook.json` | `PreToolUse`/`PostToolUse` + `matcher: ".*"` + `timeout` (Claude schema) |

Absolute path for the binaries (relative in the GitHub/VS Code case). Pinned version: `NEMESIS_VERSION=v0.1.0 bash nemesis-install.sh`.

> The **eBPF layer (Linux)** does NOT come in the binaries: it depends on `libbpf`/`clang` and on a BPF object compatible with your kernel. It is **opt-in**, built from source (Option B). The core (pretool + Defender) protects on macOS and Linux without it.

#### Installation map (summary)

Complete flow, **in order**, OS-agnostic (macOS/Linux) — all commands from the **root of your project**:

Each step is **one command** to copy whole:

| # | Step | Command (copy whole) |
|---|-------|---------|
| 1 | **Download and install** | `curl -fsSLO …/main/.nemesis/install/nemesis-install.sh -O …/main/.nemesis/install/info-install.txt && bash nemesis-install.sh` |
| 2 | **Restart the IDE** (hooks take effect) | — |
| 3 | **Diagnostic** (follow the actions it indicates) | `.nemesis/bin/nemesis-doctor --quick` |
| 4 | **Level 1 — static validation** (auto-detected binary) | `bash .nemesis/pentest-nemesis-control/nemesis-defender/run-pentest.sh` |
| 5 | **Level 2 — practical validation** (paste into the agent) | content of `.nemesis/pentest-nemesis-control/nemesis-defender/nemesis-pentest-harness.md` |

The **doctor** (step 4) prints, on each check that fails, the **exact action** already in the path of your layout (e.g.: if **G6** indicates the daemon is stopped, run `.nemesis/bin/nemesis-defender --ensure-daemon` and run the doctor again). The detailed step-by-step is in **`info-install.txt`** (root) and in `.nemesis/pentest-nemesis-control/nemesis-defender/info.md`.

### Option B — Build from source

Required for the **eBPF** layer or to **contribute**.

```bash
# Binaries generated in .nemesis/target/release/
git clone https://github.com/feryamaha/Nemesis_Defender_v0.git
cd Nemesis_Defender_v0/.nemesis
cargo build --release --workspace
```

The compilation takes a few minutes and requires the ~4 GB of RAM mentioned in the requirements. At the end, confirm that the binaries exist:

```bash
ls -la .nemesis/target/release/ | grep nemesis
```

### 2. Point the IDE hooks to the binary

This is the step that effectively turns Nemesis on. Each IDE has its format - see the next section. The common point: the pre-tool hook must point to the **absolute path** of the Nemesis binary in your project.

> **Wrong or missing path = Nemesis does not run.** The IDE simply does not invoke the hook, and you are left unprotected without noticing. Always confirm that the path in `command` points to the real binary (`nemesis-pretool-check-unix`) in your project.

> **Unified operations manual:** for complete build instructions per module, eBPF configuration, daemon operation, pentest, and a new-machine installation checklist, consult [`.nemesis/nemesis-doctor/NEMESIS-OPERATIONS.md`](.nemesis/nemesis-doctor/NEMESIS-OPERATIONS.md).

---

## Uninstallation

Run at the **project root**, in your **native terminal**. The script reverts `nemesis-install.sh`: it stops the daemon, disables the eBPF service (if you activated it, opt-in), removes the IDE hooks created by Nemesis and the `.nemesis/` folder, and prints a **final checklist** for you to confirm nothing was left behind.

**Self-contained** (works on any installation — it downloads the script and runs, mirroring the install):

## Uninstall with interactive confirmation:
```bash
curl -fsSLO https://raw.githubusercontent.com/feryamaha/Nemesis_Defender_v0/main/.nemesis/install/nemesis-uninstall.sh \
  && bash nemesis-uninstall.sh
```

## Uninstall without interactive confirmation:
```bash
curl -fsSLO https://raw.githubusercontent.com/feryamaha/Nemesis_Defender_v0/main/.nemesis/install/nemesis-uninstall.sh \
  && NEMESIS_YES=1 bash nemesis-uninstall.sh
```

The installer also leaves a local copy; if it exists, just run:
```bash
bash .nemesis/install/nemesis-uninstall.sh.
```

**What is automatic and what is manual.** The script safely removes the hook files that are **Nemesis-only** (`.codex`/`.cursor`/`.devin`/`.gemini`/`.agents/hooks.json` and `.github/hooks/`). The **shared** settings (`.claude/settings.json`, `.openclaude/settings.json`, `.vscode/settings.json`) may contain **your own configuration**, so it **does not delete them** — it only **lists** them for you to remove the Nemesis entry by hand (preserving the rest). Cleaning this up matters: an orphan hook pointing to a binary that no longer exists makes the IDE/TUI complain every session.

The **final checklist** also gives you the commands to confirm nothing is left running or orphaned:

## look for ANY leftover of the Nemesis hook (ideal: nothing):

```bash
grep -rIl 'nemesis-pretool\|nemesis-posttool\|\.nemesis/bin\|chat.hookFilesLocations' \
  .claude .openclaude .codex .cursor .devin .gemini .agents .github .vscode 2>/dev/null
```
## to confirm the daemon stopped (empty = ok) and, if needed, terminate:
```bash
pgrep -fl nemesis-defender
```
## to shut down the nemesis-defender PID
```bash
pkill -f nemesis-defender
```

## (Linux, only if you activated the eBPF opt-in) confirm/stop the kernel service:
```bash
systemctl is-active nemesis-ebpf
```

## to disable eBPF
```bash
sudo systemctl disable --now nemesis-ebpf
```

Restart the IDE afterward so it stops loading the hooks and manually delete any remaining residue.

> 💬 **A request.** If you uninstall, send me an email telling me the **reason** — positive or negative feedback is very valuable to the project. And if something went wrong in the uninstall, write too: **feryamaha@hotmail.com** (I provide support).

---

## Nemesis Doctor

The **Nemesis Doctor** is the automated framework health diagnostic. It runs 7 structured checks and issues a global verdict (`SAUDAVEL`, `ATENCAO` or `CRITICO`).

### How to run

```bash
cd .nemesis && cargo build --release -p nemesis-doctor
./target/release/nemesis-doctor
```

Quick mode (skips compilation, tests, and pentest):
```bash
./target/release/nemesis-doctor --quick
```

### What it checks

| Group | What it checks |
|-------|----------------|
| **G1** | Compilation (`cargo check --workspace`) — 0 errors, 0 warnings |
| **G2** | Unit tests (`cargo test --workspace`) — pass/fail |
| **G3** | Binary inventory in `target/release/` (11 expected) |
| **G4** | IDE scaffold — pretool/posttool hooks configured |
| **G5** | eBPF Kernel LSM (Linux) — BPF LSM active, capabilities, cgroup |
| **G6** | `nemesis-defender` daemon — PID alive, inotify active |
| **G7** | Red-Team Pentest — block rate against the static suite (`run-pentest.sh`, modules M1–M31; the suite grows each hardening cycle). Gate: **FAIL=0** (100% blocked, zero false positives) |

### Verdicts

- **SAUDAVEL** — all groups OK. System ready.
- **ATENCAO** — one or more groups with WARN (e.g.: missing capabilities). Works, but deserves attention.
- **CRITICO** — a blocking error (compilation failed, daemon dead, or pentest **FAILED**: some attack got through or there was a false positive). Fix before trusting the protection.

> **Rule:** after any recompilation that affects the `nemesis-ebpf-daemon`, **reapply the capabilities** (`setcap`) — they are lost when the binary's inode is recreated.

---

## Pretool configuration per IDE

The Rust library (`nemesis-defender`) is IDE-agnostic. What changes between IDEs is **where** you declare the hook and **what format** the payload has. Always confirm in your IDE's official doc.

### IDE support (verified in official documentation)

| IDE / Agent | Pre-tool hook | Where to declare |
|--------------|------------------|---------------|
| **Claude Code** | `PreToolUse` / `PostToolUse` | `.claude/settings.json` (project) or `~/.claude/settings.json` (global) |
| **OpenAI Codex** | `PreToolUse` / `PostToolUse` | `.codex/hooks.json` |
| **Cursor** (1.7+) | `preToolUse`, `postToolUse` | `.cursor/hooks.json` |
| **GitHub Copilot** | `preToolUse` | `.github/hooks/` |
| **VS Code** (agent, preview) | hook events | `.github/hooks/` |
| **Devin / Devin** (Cognition) | `pre_write_code`, `pre_run_command`, `pre_read_code`, `pre_mcp_tool_use` (+ `post_*`) | `.devin/hooks.json` |
| **Grok Build** (x.ai) | `PreToolUse` / `PostToolUse` | `.grok/hooks/*.json` (project) or `~/.grok/hooks/*.json` (global) |

> **Golden rule of enforcement:** blocking only happens with **exit code 2**. Exit code 1 is treated as a non-blocking error and the action proceeds. Every security hook must end in exit 2 to actually block.

### Claude Code

Edit `.claude/settings.json` (in the project) or `~/.claude/settings.json` (global). The pre-tool hook points to the Nemesis binary with an absolute path:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Read|Write|Edit|MultiEdit|Bash|NotebookEdit",
        "hooks": [
          {
            "type": "command",
            "command": "/caminho/absoluto/.nemesis/target/release/nemesis-pretool-check-unix"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Read|Write|Edit|MultiEdit|Bash|NotebookEdit",
        "hooks": [
          {
            "type": "command",
            "command": "/caminho/absoluto/.nemesis/target/release/nemesis-posttool-check-unix"
          }
        ]
      }
    ]
  }
}
```

The hook receives the tool context via **stdin as JSON** (fields like `tool_name`, `tool_input.command`, `tool_input.file_path`). The real hook binary is `nemesis-pretool-check-unix` (and `nemesis-posttool-check-unix` for the post). It reads that JSON, validates against the deny-lists, and returns exit 2 to block.

> Use **absolute paths** for the binaries - relative paths fail depending on the IDE's working directory. Project hooks (`.claude/settings.json`) take precedence over global ones.

### Cursor (1.7+)

Real configuration in `.cursor/hooks.json`. Cursor uses `preToolUse`/`postToolUse` with a broad `matcher` and the `failClosed` flag:

```json
{
  "version": 1,
  "hooks": {
    "preToolUse": [
      {
        "matcher": "Shell|Read|Write|StrReplace|Glob|Grep|Delete|EditNotebook|Task|SemanticSearch|WebFetch|TabRead|TabWrite",
        "command": "/caminho/absoluto/.nemesis/target/release/nemesis-pretool-check-unix",
        "failClosed": false
      }
    ],
    "postToolUse": [
      {
        "matcher": "Shell|Read|Write|StrReplace|Glob|Grep|Delete|EditNotebook|Task|SemanticSearch|WebFetch",
        "command": "/caminho/absoluto/.nemesis/target/release/nemesis-posttool-check-unix",
        "failClosed": false
      }
    ]
  }
}
```

### GitHub Copilot / VS Code (agent)

Real configuration in `.github/hooks/nemesis-pretool-hook.json` (path relative to the project):

```json
{
  "hooks": {
    "PreToolUse": [
      { "type": "command", "command": "./.nemesis/target/release/nemesis-pretool-check-unix" }
    ],
    "PostToolUse": [
      { "type": "command", "command": "./.nemesis/target/release/nemesis-posttool-check-unix" }
    ]
  }
}
```

**Attention** (from VS Code's own doc warning): if the agent has permission to edit the hook script, it can rewrite it during execution. Keep the hook scripts under `absolute_block` (see [Path control](#path-control)).

### OpenAI Codex

Real configuration in `.codex/hooks.json`, with a wildcard `matcher` and `timeout`:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          { "type": "command", "command": "/caminho/absoluto/.nemesis/target/release/nemesis-pretool-check-unix", "timeout": 30 }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": ".*",
        "hooks": [
          { "type": "command", "command": "/caminho/absoluto/.nemesis/target/release/nemesis-posttool-check-unix", "timeout": 30 }
        ]
      }
    ]
  }
}
```

> **Attention:** confirm the path in `command` points to your project's real directory. A wrong path makes the hook not run and Codex is left unprotected.

### Devin / Devin (Cognition)

Where Nemesis was born natively. Real configuration in `.devin/hooks.json`, which uses its own events (`pre_write_code`, `pre_run_command`, `pre_read_code`, `pre_mcp_tool_use`, and the equivalent `post_*`):

```json
{
  "hooks": {
    "pre_write_code": [
      { "command": "/caminho/absoluto/.nemesis/target/release/nemesis-pretool-check-unix", "show_output": true }
    ],
    "pre_run_command": [
      { "command": "/caminho/absoluto/.nemesis/target/release/nemesis-pretool-check-unix", "show_output": true }
    ],
    "pre_read_code": [
      { "command": "/caminho/absoluto/.nemesis/target/release/nemesis-pretool-check-unix", "show_output": true }
    ],
    "pre_mcp_tool_use": [
      { "command": "/caminho/absoluto/.nemesis/target/release/nemesis-pretool-check-unix", "show_output": true }
    ],
    "post_write_code": [
      { "command": "/caminho/absoluto/.nemesis/target/release/nemesis-posttool-check-unix", "show_output": true }
    ]
  }
}
```

The events `post_run_command`, `post_read_code`, and `post_mcp_tool_use` follow the same pattern as `post_write_code`.

---

## eBPF layer configuration (Linux)

This layer is **optional** and Linux-specific. It is the minimal containment net for destructive commands in case the pretool is turned off. If you do not use Linux or do not need this extra layer, skip this section - Nemesis works without it via the pretool.

**Complete installation and operation instructions:** consult [`.nemesis/ebpf-kernel/info.md`](.nemesis/ebpf-kernel/info.md)

### Prerequisites

- Linux kernel ≥ 5.7
- BPF LSM active at boot (`cat /sys/kernel/security/lsm` must contain `bpf`)
- clang and bpftool installed
- The ability to delegate capabilities (`cap_bpf`, `cap_perfmon`, `cap_sys_resource`)

### Compilation

```bash
cargo build -p nemesis-ebpf-kernel
```

**Note on build blocking:** If BPF LSM is active and blocking the build (error "Operation not permitted" on the `rm` in the make), stop the eBPF daemon before compiling:

```bash
# Check whether the daemon is running
ps aux | grep nemesis-ebpf-daemon

# Stop the daemon
sudo systemctl stop nemesis-ebpf  # if it is running as a service
# or kill the process manually
kill <PID_DO_DAEMON>

# Try to compile again
cargo build -p nemesis-ebpf-kernel
```

If even after stopping the daemon the build fails, the BPF LSM program may be loaded in the kernel. In that case, reboot the system to unload it, since BPF programs cannot be removed dynamically.

### Delegate capabilities

```bash
sudo setcap cap_bpf,cap_perfmon,cap_sys_resource+eip \
  .nemesis/target/release/nemesis-ebpf-daemon
```

### Start the daemon

```bash
.nemesis/target/release/nemesis-ebpf-daemon --start
```

The daemon creates/uses the cgroup `/sys/fs/cgroup/nemesis-agent`, loads the BPF LSM program, and stays in epoll mode (near-zero consumption at idle). Only agent processes moved into that cgroup are checked - IDE, terminal, and system processes pass without checking.

### Rootless sandbox mode (Landlock)

If you cannot delegate capabilities, the daemon operates in degraded mode via Landlock, which protects only the child's process tree:

```bash
.nemesis/target/release/nemesis-ebpf-daemon --sandbox
```

### What it does

The eBPF layer operates at the kernel level via BPF LSM (`bprm_check_security`), blocking destructive executions (commands in `denylist-ebpf/commands.toml`) only for processes inside the cgroup `/sys/fs/cgroup/nemesis-agent`. IDE, terminal, and system processes pass without checking.

### Configuration files

- `denylist-ebpf/commands.toml` - Binaries blocked by basename
- `denylist-ebpf/paths.toml` - Blocked write paths
- `nemesis-ebpf.service` - systemd service for automatic activation
- `install-service.sh` - Service installation script

---

## Path control

After installation, what the agent can touch is defined in `denylist-folder-files.json`, under **exclusively human** control, at three levels:

- **`absolute_block`** - total block (read + write + delete). Includes `.env`, `.ssh/id_rsa`, `.bashrc`/`.zshrc`, each IDE's settings/hooks (`.claude/`, `.cursor/`, `.devin/`) and `.nemesis/` itself.
- **`write_block`** - reading allowed, write/edit blocked. Includes `package.json`, `next.config.js`, `eslint.config.mjs`, `.gitignore` and the logs.
- **`allowed_exceptions`** - the released scaffold (e.g.: `/src/`), where the agent writes and edits freely.

**These files become the responsibility of manual human editing.** The AI agent does not edit or delete them. A destructive command (delete, overwrite outside scope, reset) remains **always forbidden to the AI**, regardless of any read/write permission.

---

## Day-to-day use

With the hooks configured, Nemesis operates transparently: it only shows itself when it blocks something. Legitimate commands and writes pass without friction.

```bash
# Scan a file manually
nemesis-defender --scan /caminho/arquivo.rs

# Start / stop the filesystem daemon
nemesis-defender --ensure-daemon
nemesis-defender --stop

# See recent blocks (single ledger for ALL layers, JSONL)
tail -20 .nemesis/logs/nemesis-violations.log | jq .

# Local telemetry: total + per layer + per type + per day
nemesis-defender --log-stats
```

> **100% local logging.** Every Nemesis log and telemetry stays in `.nemesis/` inside your own project. **By default, nothing leaves your machine** — there is no remote collection nor "phone home" of commands, paths, or content. The only exception, **optional and opt-in**, is the publisher's anonymous install/uninstall ping (opaque UUID, without any machine or project data), which you can simply not activate. The layer blocks (pretool, posttool, nemesis-defender, eBPF) all go to a **single ledger** `.nemesis/logs/nemesis-violations.log`; the old history is archived in `.nemesis/logs/log-legado/`. The behavioral correlation state (which the daemon uses for multi-turn detection) stays in `.nemesis/runtime/session-events.jsonl` — also local.

### Block messages

When something is barred, Nemesis emits one of six categorized messages, so that you (and the agent) know exactly why:

| Category | Message |
|-----------|----------|
| Blocked command | `NEMESIS SEC - COMANDO NAO PERMITIDO` |
| Write to a protected file | `NEMESIS SEC - ACESSO NEGADO - ARQUIVO PROTEGIDO` |
| Read of a protected file | `NEMESIS SEC - LEITURA NEGADA - ARQUIVO PROTEGIDO` |
| Malicious content | `NEMESIS SEC - CONTEUDO MALICIOSO DETECTADO` |
| Write outside scope | `NEMESIS SEC - ESCRITA FORA DO ESCOPO PERMITIDO` |
| Code pattern violation | `NEMESIS QUALITY - PADRAO DE CODIGO NAO PERMITIDO ANALISAR REGRAS!` |

In the terminal under eBPF, the kernel emits the system's default message (`Operação não permitida`) with exit code 126 - the standardized record stays in the ledger `.nemesis/logs/nemesis-violations.log` (`ebpf-kernel` layer).

---

## Verification and diagnostics

```bash
# Scan a file's content manually (same engine as the daemon/hook)
.nemesis/target/release/nemesis-defender --scan /caminho/arquivo.js

# eBPF layer diagnostic (Linux)
.nemesis/target/release/nemesis-ebpf-daemon --doctor
.nemesis/target/release/nemesis-ebpf-daemon --print-status

# Kernel layer (eBPF) blocks in the single ledger
grep '"layer":"ebpf-kernel"' .nemesis/logs/nemesis-violations.log
```

**Note:** The `nemesis-ebpf-daemon` binary must be compiled with `cargo build -p nemesis-ebpf-kernel`. If BPF LSM is active and blocking the build, stop the eBPF daemon before compiling.

To confirm the pretool is actually active, force a command that should be blocked on a disposable test file and check whether it appears in the log. If nothing happens, the hook is probably not pointing to the right path.

---

## Relaxing or customizing rules

### Nemesis is calibrated for frontend

The detector was calibrated against **frontend** reality (Next.js / React / TypeScript), where the false positive (FP) stays **below ~1%**. Frontend practically does not generate "scripted" code (sudo, `sed -i`, dynamic exec, PATH manipulation); for that stack, those commands are hostile and unnecessary, so blocking them is correct.

**Backend / DevSecOps** stacks (and multiple languages) have a **higher** FP coefficient — they legitimately use commands that would be hostile for frontend. This is a **known limitation** of Nemesis and the reason the allowlist exists (below). Estimate **per sector** (from empirical measurement on real open-source codebases, with a conservative margin):

| Sector | Typical stack | Estimated FP |
|---|---|---|
| **Frontend** | Next.js / React / TypeScript | **< 1%** |
| **Backend** | Python / Node / multiple languages | **~3–6%** |
| **DevSecOps / IaC / Shell** | Ansible, installers, scripts, remote exec | **from ~7%** |

These are estimates with a margin of error; FP grows the more "scripted" the stack is. Intrinsically offensive tools (e.g.: exploit/shellcode libraries) light up by **design** — it is the expected ceiling, not friendly fire, and confirms that real detection is alive.

> **Reading:** low FP in frontend (and in real Rust); it grows in backend/devops/shell because they use commands intrinsic to the stack. It is not the detector "breaking" — it is the frontend calibration meeting legitimate code of another nature. Those environments should **relax via the allowlist**.

### The allowlist (the only editable surface)

The **blocking** deny-lists are **embedded in the binary** (tamper-proof) — there is no file on disk to edit, and this is intentional: an agent cannot weaken Nemesis by editing rules. The **only** surface you edit after installing is:

```
.nemesis/denylist-customers/allowlist-customers.jsonc
```

It is an **absolute human override**: everything you list passes, overriding **any** block (command denylist, defender, visitors) — in the pretool and in the daemon. Immediate effect on save (no rebuild). This is how backend/DevSecOps **relax** Nemesis to their own stack's reality:

```jsonc
{
  // allow_commands: matches by SUBSTRING; allow_patterns: matches by REGEX (no lookahead)
  "allow_commands": ["sudo systemctl restart nginx", "rm -rf ./dist"],
  "allow_patterns": ["^cp\\s+-r\\s+"]
}
```

> **Responsibility warning.** The allowlist is absolute: if you release `rm -rf`, Nemesis stops blocking `rm -rf`. You **return to the model the power to decide** about what you released — at your own risk. The file is editable **by a human only** (the agent never writes to it — `absolute_block`); that is the guarantee that keeps the override from being self-sabotage. Edit it in your native terminal.

### Two layers, two allowlists (important for Linux)

The `allowlist-customers.jsonc` relaxes the **pretool + the defender/daemon** — where the agent's command/content false positives live. It applies on Linux and macOS (validated platforms); on Windows, best-effort (see [Platform support](#platform-support)).

The **eBPF** layer (kernel, Linux, opt-in) has its **own and separate** denylist (`denylist-ebpf/commands.toml`) that the allowlist above does **not** control. On Linux, commands like `rm`/`chmod` only **actually execute** if you also list them in the eBPF allowlist — this way you relax the kernel **without editing the official list**:

```
.nemesis/denylist-customers/allowlist-ebpf.toml
```
```toml
# EXACT command name (basename of the exec); at your own risk
allowed_commands = ["rm", "chmod", "tar"]
```

The eBPF loader **removes** those commands from blocking when starting the daemon (reloads on restart). On macOS/Windows there is no eBPF: the `allowlist-customers.jsonc` alone already releases. The **Defender visitors** remain Rust code (extending them requires Rust).

---


## Troubleshooting

| Symptom | Probable cause | Action |
|---------|----------------|------|
| Nemesis blocks nothing | Hook does not point to the right absolute path | Review the IDE's `settings.json`/`hooks.json` |
| `enforcement_level` is `landlock` | BPF LSM not active or without capabilities | Redo steps 1-2 of the [eBPF config](#ebpf-layer-configuration-linux) |
| eBPF does not block a destructive command | The agent's process is not in the cgroup | Move the PID to `/sys/fs/cgroup/nemesis-agent/cgroup.procs` |
| Build fails due to lack of memory | Less than ~4 GB of free RAM | Free memory or compile with less parallelism |

---

## Paused modules

Nemesis has features present in the code but currently inactive:

**ast-linters** (`ast-linters/`). Code quality layer with tree-sitter visitors focused on the frontend Next/React/TypeScript stack. Detects anti-patterns like explicit `any`, conditional hooks, inline CSS, unhandled promises, and hardcoded secrets. The module is **silenced** — present in the code but without active enforcement.

---

## Project structure

Base layout of the repository (folders and key files; `bin/`, `target/`, `runtime/` are generated and **not** versioned):

```text
Nemesis_Defender_v0/
├─ README.md  AGENTS.md  CLAUDE.md            # canonical docs (AGENTS = maintainer agent)
├─ SECURITY.md  CONTRIBUTING.md  CODE_OF_CONDUCT.md  NOTICE  LICENSE
├─ .gitignore  config.yml  PULL_REQUEST_TEMPLATE.md
│
├─ .github/                                   # governance + CI/CD
│  ├─ workflows/release.yml                   # build + attestation (SLSA) + release (draft)
│  ├─ workflows/self-audit.yml               # gate: pentest + cargo audit + pin-check
│  ├─ CODEOWNERS                              # mandatory review on trust-critical paths
│  └─ ISSUE_TEMPLATE/  hooks/  settings.json
│
├─ .nemesis/                                  # core: Rust workspace + runtime
│  ├─ Cargo.toml  Cargo.lock                  # workspace (COMMITTED lockfile)
│  ├─ nemesis-defender/                       # "Iron Dome" scanner (lib + daemon)
│  │  ├─ src/                                 # visitors + 6 scan layers + severity
│  │  ├─ config/denylist-defender.json        # content security (EMBEDDED in the binary)
│  │  └─ tests/
│  ├─ nemesis-doctor/                         # G1–G7 diagnostic + NEMESIS-OPERATIONS.md
│  ├─ ebpf-kernel/                            # kernel layer (Linux, opt-in)
│  │  ├─ src/                                 # loader, config, landlock (rootless sandbox)
│  │  ├─ ebpf/  include/  denylist-ebpf/      # BPF program + allowlists (egress/landlock)
│  │  └─ Makefile
│  ├─ ast-linters/                            # code quality (paused)
│  ├─ hooks/                                  # pretool/posttool (.rs) + fail-closed fallback
│  ├─ denylist/                               # EDITABLE deny-lists (command/quality/folders)
│  ├─ install/                                # nemesis-install.sh + info-install.txt (curl)
│  ├─ pentest-nemesis-control/                # red-team suite (run-pentest.sh + scenarios)
│  ├─ forensics/                              # external content audit (scan-incoming.sh)
│  ├─ scripts/  lsp/                          # build/caps + LSP
│  ├─ bin/ · target/                          # binaries (distro · build from source) — generated
│  └─ runtime/ · quarantine/                  # daemon PID/lock · quarantined files
│
└─ .claude/ .devin/ .cursor/ .codex/ .gemini/ .agents/ .openclaude/   # per-IDE hook scaffolds
```

---

## Contributing

Contributions are welcome - code, new deny-list vectors, and especially **bypass reports**. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

To **maintain Nemesis** (in any IDE/TUI), the starting point is [`AGENTS.md`](AGENTS.md) - the canonical maintainer agent (security invariants, epistemic discipline, repository map, Rust best practices) - and the operation manual [`.nemesis/nemesis-doctor/NEMESIS-OPERATIONS.md`](.nemesis/nemesis-doctor/NEMESIS-OPERATIONS.md) (build, daemon/pretool/eBPF lifecycle, logs, checklist).

The project adopts the **Developer Certificate of Origin (DCO)**: sign your commits with `git commit -s`.

The eBPF layer, in particular, is an open field: today it covers execution of destructive binaries (execve). Extending to non-execve writes (`file_open`/`inode_unlink` hooks), matching by inode instead of basename, and seccomp in `--start` mode are mapped improvements available to anyone who wants to contribute.

---

## Security and disclosure

Bypasses and uncovered vectors are **expected** and **welcome**. If you bypass any layer, **do not open a public issue** - follow [`SECURITY.md`](SECURITY.md) and report privately to `feryamaha@hotmail.com`. Researchers are credited publicly (unless they prefer anonymity).

---

## License

Distributed under the **GNU AGPL v3.0** (see [`LICENSE`](LICENSE)). You may use, study, modify, and redistribute freely - but any derivative or service (including SaaS) must keep the code open under the same license.

Full copyright remains with the author, who offers a **separate commercial license** (dual licensing) for use without the AGPL obligations. Contact: **feryamaha@hotmail.com**.

---

**Author / maintainer:** [@feryamaha](https://github.com/feryamaha)

**Networks:** [GitHub](https://github.com/feryamaha) · [LinkedIn](https://www.linkedin.com/in/feryamaha) · [X (Twitter)](https://x.com/_feryamaha) · [Email](mailto:feryamaha@hotmail.com)
