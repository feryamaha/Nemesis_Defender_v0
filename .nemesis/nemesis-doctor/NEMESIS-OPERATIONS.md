# NEMESIS — Unified Operations Manual

> **Canonical operations document of the Nemesis Framework v2.0.**
> Replaces `doc-cargo-compile-binary.md` and `pentest-nemesis-control/instruction-daemon+ebpf-info.md` (obsolete).
> All paths are relative to the project root: `/home/fernando/devproj/Nemesis_Defender_v0`.

---

## 0. Automatic diagnostics (recommended)

Before any manual check, run the structured diagnostics:

```bash
# Full diagnostics (compilation + tests + inventory + scaffold + eBPF + daemon + pentest)
.nemesis/target/release/nemesis-doctor

# Quick mode (skips G1 compile, G2 tests and G7 pentest)
.nemesis/target/release/nemesis-doctor --quick
```

The `nemesis-doctor` returns a report with 7 groups and a global verdict
(`SAUDAVEL` / `ATENCAO` / `CRITICO`) and exit code (0 = ok/warn, 1 = critical).

| Group | What it checks |
|-------|----------------|
| G1 | Compilation (`cargo check --workspace`) |
| G2 | Unit tests (`cargo test --workspace`) |
| G3 | Inventory of binaries in `target/release` |
| G4 | IDE scaffold (`hooks.json`/`settings.json` pretool/posttool) |
| G5 | eBPF Kernel LSM (Linux only) |
| G6 | Daemon `nemesis-defender` (PID + inotify) |
| G7 | Red-Team Pentest (`run-pentest.sh` + parse CSV) |

The sections below are the **manual checklist** for targeted inspection.

---

## 1. Workspace Structure

Cargo workspace `nemesis` (v8.2.0) with members:

```
.nemesis/
├── Cargo.toml          # workspace + root package (hooks/CLI)
├── ast-linters/        # AST semantic analysis (lib, no binary)
├── ebpf-kernel/        # eBPF enforcement (Linux)
├── nemesis-defender/   # Iron Dome scanner + daemon
└── nemesis-doctor/     # health diagnostics
```

### Expected binaries (11) in `.nemesis/target/release/`

| Source | Binaries |
|--------|----------|
| package `nemesis` | `nemesis-pretool-check`, `nemesis-pretool-check-unix`, `nemesis-pretool-check-windows`, `nemesis-pretool-hook`, `nemesis-posttool-check-unix`, `pre-edit-hook`, `debug-hook-env`, `nemesis-lsp` |
| `nemesis-defender` | `nemesis-defender` |
| `ebpf-kernel` | `nemesis-ebpf-daemon`, `nemesis-cgroup-watcher` |
| `ast-linters` | (lib — no binary) |
| `nemesis-doctor` | `nemesis-doctor` |

---

## 2. Compilation

### Full workspace

```bash
cd .nemesis && cargo build --release --workspace
```

> **Linux + eBPF:** prefer the wrapper, which recompiles AND reapplies the eBPF
> capabilities (lost on every build, since `setcap` is per-inode):
> ```bash
> .nemesis/scripts/nemesis-build.sh           # build --workspace + ensure-ebpf-caps
> ```
> To just reactivate the caps without recompiling: `.nemesis/scripts/ensure-ebpf-caps.sh`
> (idempotent; sudo only when it needs to apply; no-op on macOS).

### Per module

```bash
cd .nemesis && cargo build --release -p nemesis-defender
cd .nemesis && cargo build --release -p nemesis-doctor
cd .nemesis && cargo build --release -p nemesis-ebpf-kernel
cd .nemesis && cargo build --release -p ast-linters
```

### Quick check (without generating a binary)

```bash
cd .nemesis && cargo check --workspace
```

**What to analyze in the output:**
- `Finished ... profile` => compiled successfully.
- `warning:` => does not block, but should be reviewed.
- `error[Exxx]:` => **blocks the build** — fix before proceeding.

> After recompiling `nemesis-defender`, **restart the daemon** (section 4). Recompiling alone
> is not enough while the old daemon (binary in memory) is still alive.

---

## 3. eBPF Kernel LSM (Linux only)

Additional layer, independent of the pretool. Activated **only once** at installation.

### Verify

```bash
cat /sys/kernel/security/lsm                 # must contain 'bpf'
getcap .nemesis/target/release/nemesis-ebpf-daemon
ls /sys/fs/cgroup/nemesis-agent/
sudo bpftool prog list
```

### Diagnostics / start / stop

```bash
sudo .nemesis/target/release/nemesis-ebpf-daemon --doctor
sudo .nemesis/target/release/nemesis-ebpf-daemon --start
sudo killall nemesis-ebpf-daemon 2>/dev/null; echo "PARADO"
```

### Capabilities (once per machine)

```bash
sudo setcap cap_bpf,cap_perfmon,cap_sys_resource+eip \
        .nemesis/target/release/nemesis-ebpf-daemon
```

## Starts daemon + watcher, reapplies cap, re-enables on boot:

```bash
sudo bash .nemesis/ebpf-kernel/install-service.sh
```

##  Useful commands:
  systemctl status nemesis-ebpf         # view state of the eBPF daemon
  systemctl status nemesis-cgroup-watcher # view state of the watcher
  journalctl -u nemesis-ebpf -f         # view logs in real time
  sudo systemctl stop nemesis-ebpf      # stop
  sudo systemctl restart nemesis-ebpf   # restart

> macOS/Windows: eBPF does not apply — the defense stays on the pretool trails. The `nemesis-doctor` reports `NA`.

---

## 4. Daemon nemesis-defender

Real-time scanner (inotify). It should start **automatically** when the IDE
triggers the pretool hook (`nemesis-pretool-check-unix` runs `--ensure-daemon`).
If the IDE scaffold (section 5) is not configured, start it manually.

### Verify

```bash
pidof nemesis-defender && echo "ATIVO" || echo "INATIVO"
ls -la /proc/$(pidof nemesis-defender)/fd/ | grep inotify
```

### Start / stop / restart

```bash
# Start (if not running)
.nemesis/target/release/nemesis-defender --ensure-daemon

# Stop
pkill -9 -f "nemesis-defender"; pidof nemesis-defender || echo "PARADO"

# Restart (mandatory after recompiling the defender)
pkill -9 -f "nemesis-defender"; sleep 1; .nemesis/target/release/nemesis-defender --ensure-daemon
```

### Manual scan

```bash
.nemesis/target/release/nemesis-defender --scan /caminho/arquivo.ts
```

---

## 5. IDE Scaffold (hooks)

Without the **pretool** hook configured, the IDE does not trigger `nemesis-defender --ensure-daemon`
and the daemon **does not start on its own** (on Linux, only eBPF protects).

Files verified by `nemesis-doctor` (G4):

```
.devin/hooks.json
.claude/settings.json
.cursor/hooks.json
.codex/hooks.json
.github/hooks.json
```

**What to analyze:**
- File `{}` or empty => daemon does not start automatically.
- Must reference `pretool` (daemon ignition) and, ideally, `posttool` (post-write scan).
- The referenced binary must exist in `.nemesis/target/release/`.

---

## 6. Red-Team Pentest

Automated suite (184 tests, 26 modules) that injects malicious commands/files
into the pretool binary via stdin (non-destructive) and measures the block rate.

```bash
bash .nemesis/pentest-nemesis-control/nemesis-defender/run-pentest.sh \
    .nemesis/target/release/nemesis-pretool-check-unix
```

Requires `bash` + `node`. Generates `pentest-results.csv` and `pentest-results.md`.

**What to analyze:**
- Rate `>= 95%` => `PRODUCAO-READY`; `90-94%` => `HARDENING`; `< 90%` => `NAO LANCAR`.
- Module **M26** uses inverted logic: blocking there = **false-positive** (regression).

---

## 7. Logs and telemetry — 100% local recording

> **Privacy:** all Nemesis recording is **local** — written only inside
> `.nemesis/` in the project itself. **Nothing is exfiltrated, sent, or telemetered outside**
> the machine of whoever installs it. There is no server, remote collection, or "phone home". The data
> exists only for the dev themselves to audit and validate the protection.

The layers (pretool, posttool, nemesis-defender, eBPF) record each block on a standardized
line in a **single ledger**:

```
.nemesis/logs/nemesis-violations.log     # JSONL — ONE block event per line
```

Schema:
```json
{"ts":"2026-06-11T09:25:34-03:00","date":"2026-06-11","time":"09:25:34","layer":"pretool","message":"NEMESIS SEC - LEITURA NEGADA - ARQUIVO PROTEGIDO · .env"}
```
- `layer` ∈ `pretool` | `posttool` | `nemesis-defender` | `ebpf-kernel`
- `message` = standard message (vocabulary of the 6 messages), already with the target (`· <target>`)

### Local telemetry

```bash
.nemesis/target/release/nemesis-defender --log-stats
```

Prints total blocks, **per layer** (priority order — eBPF, the last layer,
should have the LOWEST volume), **per type** (most frequent first) and **per day**. It serves to
validate the protection, see the most frequent vectors and audit false-positives.

### Files in `.nemesis/`

| File | Function |
|---|---|
| `.nemesis/logs/nemesis-violations.log` | **Single** ledger of blocks (all layers) |
| `.nemesis/logs/log-legado/` | Archived history of old logs |
| `.nemesis/runtime/session-events.jsonl` | Runtime state (non-log): the pretool writes each tool-call; the daemon reads it for **behavioral correlation** (multi-turn / escalation). Also 100% local. |

> After recompiling or relocating paths, restart the daemon
> (`pkill -9 -f nemesis-defender` + `--ensure-daemon`) so it re-reads `session-events.jsonl`
> at the current path (`runtime/`).

---

## 8. Installation checklist (new machine)

- [ ] Rust + system dependencies installed (`build-essential`, `libbpf-dev`, `clang`, `bpftool`).
- [ ] `cd .nemesis && cargo build --release --workspace` without errors.
- [ ] 11 binaries present in `.nemesis/target/release/` (run `nemesis-doctor`).
- [ ] eBPF: capabilities + service active (Linux) — once.
- [ ] IDE scaffold with pretool/posttool pointing to the correct binaries.
- [ ] `nemesis-doctor` returns verdict `SAUDAVEL`.
