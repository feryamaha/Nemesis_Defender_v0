# nemesis-ebpf-kernel — Operating Instructions

## Prerequisites

| Requirement | Verification |
|---|---|
| Linux kernel ≥ 5.7 | `uname -r` |
| BPF LSM active at boot | `cat /sys/kernel/security/lsm` must contain `bpf` |
| clang installed | `which clang` |
| bpftool installed | `which bpftool` |
| Compiled binary | `cargo build --release -p nemesis-ebpf-kernel` |

---

## 1. Enable BPF LSM at boot (one time, requires reboot)

> **Shortcut (recommended):** `sudo bash .nemesis/ebpf-kernel/install-service.sh` now does this
> automatically (edits GRUB with backup, idempotently) + setcap + cgroup + systemd —
> **a single sudo**. The manual edit below is only necessary if you have a custom `lsm=`
> (the script does not touch it in that case, for safety) or prefer to do it by hand.

```bash
sudo sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT="\(.*\)"/GRUB_CMDLINE_LINUX_DEFAULT="\1 lsm=lockdown,capability,landlock,yama,apparmor,bpf"/' /etc/default/grub
sudo update-grub
sudo reboot
```

Verify after reboot:
```bash
cat /sys/kernel/security/lsm
# expected: lockdown,capability,landlock,yama,apparmor,bpf
```

---

## 2. Compile the project

```bash
# From the project root
cargo build --release -p nemesis-ebpf-kernel
```

**IMPORTANT - Build block by the BPF LSM:**

If the BPF LSM is active and blocking the build ("Operation not permitted" error on make's `rm` during the libbpf-sys compilation), follow these steps:

```bash
# 1. Check if the daemon is running
ps aux | grep nemesis-ebpf-daemon

# 2. Stop the daemon
sudo systemctl stop nemesis-ebpf  # if running as a service
# or kill the process manually
kill <PID_DO_DAEMON>

# 3. Try compiling again
cargo build --release -p nemesis-ebpf-kernel
```

If the build still fails even after stopping the daemon, the BPF LSM program may be loaded in the kernel. BPF programs cannot be removed dynamically. In that case:

```bash
# Reboot the system to unload the BPF LSM program
sudo reboot
```

After the reboot, compile before starting the daemon again.

The BPF object (`.bpf.o`) is compiled automatically by the daemon on first run via `make`.

---

## 3. Enable eBPF enforcement (complete procedure)

All steps below must be run from the project root.

### 3.1 Create the agent cgroup (once per boot, requires sudo)

```bash
sudo mkdir -p /sys/fs/cgroup/nemesis-agent
```

Verify:
```bash
stat -c "%i" /sys/fs/cgroup/nemesis-agent
# must return a number (e.g., 14014) — this is the cgroup_id
```

### 3.2 Delegate capabilities to the binary (once per build)

Required after every `cargo build` that recreates the binary:

```bash
sudo setcap cap_bpf,cap_perfmon,cap_sys_resource+eip \
  .nemesis/target/release/nemesis-ebpf-daemon
```

Verify:
```bash
getcap .nemesis/target/release/nemesis-ebpf-daemon
# expected: cap_sys_resource,cap_perfmon,cap_bpf=eip
```

### 3.3 Start the BPF LSM daemon

```bash
.nemesis/target/release/nemesis-ebpf-daemon --start
```

Expected output:
```
[nemesis] loading BPF LSM program into kernel...
make: Nada a ser feito para 'all'.
[nemesis] Agent cgroup_id 14014 registered in BPF
[nemesis] BPF LSM attached — enforcement active. Ctrl-C to stop.
```

The line `Agent cgroup_id ... registered in BPF` confirms that the per-cgroup filter is active.
If this line does not appear, the cgroup was not created correctly (go back to step 3.1).

The daemon stays in **epoll mode** — consumes ~0% CPU and ~10MB RAM at idle. It wakes only when an execve is intercepted.

### 3.4 Move the agent processes into the cgroup

For the BPF to block commands, the LLM agent process must be inside the cgroup:

```bash
echo <PID_DO_AGENTE> | sudo tee /sys/fs/cgroup/nemesis-agent/cgroup.procs
```

Verify:
```bash
cat /proc/<PID>/cgroup
# expected: 0::/nemesis-agent
```

All child processes inherit the cgroup automatically.

### Automatic activation via systemd (recommended)

Install the service once:

```bash
sudo bash .nemesis/ebpf-kernel/install-service.sh
```

After installation, the daemon starts automatically at boot. Commands:

```bash
systemctl status nemesis-ebpf       # view state
journalctl -u nemesis-ebpf -f       # view logs in real time
sudo systemctl stop nemesis-ebpf    # stop
sudo systemctl restart nemesis-ebpf # restart
```

systemd takes care of: creating the cgroup, delegating capabilities and starting the daemon.
The daemon moves itself into the cgroup automatically on start — subprocesses inherit it.

### Manual activation (alternative)

```bash
# 1. Create cgroup (once per boot)
sudo mkdir -p /sys/fs/cgroup/nemesis-agent

# 2. Capabilities (once per build)
sudo setcap cap_bpf,cap_perfmon,cap_sys_resource+eip .nemesis/target/release/nemesis-ebpf-daemon

# 3. Start daemon (dedicated terminal)
.nemesis/target/release/nemesis-ebpf-daemon --start
```

The daemon moves itself into the cgroup automatically — there is no need to move PIDs manually.

---

## 4. Check state without starting

```bash
# Full diagnostics
.nemesis/target/release/nemesis-ebpf-daemon --doctor

# Quick status
.nemesis/target/release/nemesis-ebpf-daemon --print-status
```

Important fields in `--doctor`:
- `bpf_lsm_active`: true = BPF LSM active in the kernel
- `enforcement_level`: `"bpf_lsm"` = enforcement active | `"landlock"` = no root/CAP_BPF
- `can_load_bpf`: true = sufficient capabilities

---

## 5. Rootless sandbox mode (Landlock + seccomp)

Protects only the child process's process tree, with no need for root:

```bash
.nemesis/target/release/nemesis-ebpf-daemon --sandbox
```

---

## 6. Per-cgroup filter architecture

The BPF LSM program filters executions by cgroup. Only processes inside the cgroup `/sys/fs/cgroup/nemesis-agent` are checked against the deny-list. IDE, terminal and system processes pass without verification.

Internal flow:
1. The daemon reads the inode of the `/sys/fs/cgroup/nemesis-agent` directory (cgroup_id)
2. The cgroup_id is registered in the BPF map `agent_cgroup_map`
3. On each execve, the BPF program compares `bpf_get_current_cgroup_id()` with the registered cgroup
4. Processes outside the cgroup: `return 0` (allowed)
5. Processes inside the cgroup: checked against `blocked_commands`
6. If the command is on the deny-list: `return -EPERM` (blocked) + event in the ringbuf

---

## 7. Configurable deny-lists

| File | Content |
|---|---|
| `denylist-ebpf/commands.toml` | Binaries blocked by basename |
| `denylist-ebpf/paths.toml` | Blocked write paths |
| `denylist-ebpf/landlock-allowed-exec.toml` | Exec allowed in sandbox mode |
| `denylist-ebpf/egress.toml` | **Egress allowlist** (CIDR:port) + `enforce` flag |

Edit and restart the daemon to apply. Does not require recompilation.

### 7.1 Egress allowlist (lsm/socket_connect)

Beyond blocking `execve`, the daemon intercepts **outbound connections** (`lsm/socket_connect`)
from processes in the agent cgroup and **denies by default** destinations outside the allowlist —
neutralizing exfiltration/C2 even if a payload manages to run. Config in `denylist-ebpf/egress.toml`:

```toml
enforce = true                         # false (default) = only observes/logs; true = enforces deny-by-default
allowlist = ["140.82.112.0/20:443"]    # "CIDR:port"  (port 0 = any)
```

- Match via **LPM trie** (longest-prefix on CIDR), IPv4 and IPv6. Port validated by equality
  (0 = any). Empty allowlist + `enforce=true` ⇒ denies everything (fail-closed).
- Non-IP families (AF_UNIX, etc.) are not blocked. A block records
  `NEMESIS SEC - CONEXAO NAO PERMITIDA` in the ledger.
- Apply changes: `sudo systemctl restart nemesis-ebpf` (or SIGHUP). v1 is per IP/CIDR;
  domain/DNS is left for v2.

> **Design note (per-basename exec allowlist — discarded):** an exec allowlist by
> basename in eBPF was evaluated and **discarded**. The exec hook only sees the basename (`git`), not the
> command-line, so it cannot distinguish `git diff` (ok) from `git checkout`/`git reset`
> (hostile) — that granularity belongs to the **pretool** (regex over the whole line), which already
> does it. Furthermore, allowlisting interpreters (`bash`, `python3`) would turn the enforcement into a false
> sense of security. Per-command-line exec policy stays in the pretool; eBPF keeps the
> per-basename denylist + the egress.

### Full list of commands blocked in the kernel (39 commands)

| Category | Commands |
|---|---|
| Data destruction | rm, shred, truncate, dd, mkfifo, split, csplit |
| Permissions | chmod, chown |
| Filesystem | mount, umount, mkfs, fdisk |
| Databases | dropdb, mysql, psql |
| Infrastructure/cloud | terraform, docker, aws, kubectl |
| Scripting languages | python, python3, ruby, perl, lua |
| Editors | nano, vim, vi, emacs, micro |
| Exfiltration | curl, wget, ftp, sftp, rsync, scp, nc, netcat, socat |
| Process control | kill, killall, nohup |
| Text-based injection | sed, awk, gawk, mawk, ed, ex |
| Others | ln, pax, tar, zip, unzip |

---

## 8. Run validation tests

With the daemon running in another terminal:

```bash
# Level 1 — basic commands
bash test-violations/pentest-ebpf-kernel/level-1-bpf-lsm.sh

# Level 2 — evasion via subprocesses and absolute paths
bash test-violations/pentest-ebpf-kernel/level-2-resource-exhaustion.sh

# Level 3 — bypass via symlinks and wrappers
bash test-violations/pentest-ebpf-kernel/level-3-kernel-bypass-attempts.sh
```

---

## 9. Violation logs

Single unified ledger (the old `violations.log` was removed from the architecture):

```bash
grep '"layer":"ebpf-kernel"' .nemesis/logs/nemesis-violations.log
```

Per-line schema: `{ts, date, time, layer, message}`. A blocked egress appears as
`NEMESIS SEC - CONEXAO NAO PERMITIDA · ...`.

---

## Enforcement summary by level

| Condition | Active level | Scope |
|---|---|---|
| No root, no CAP_BPF, BPF LSM active | `landlock` | process tree only |
| With CAP_BPF + CAP_PERFMON + CAP_SYS_RESOURCE, BPF LSM active | `bpf_lsm` | agent cgroup (via `agent_cgroup_map`) |
| BPF LSM inactive at boot | `pretool` | IDE/Cascade hooks only |

---

## Separation of responsibilities: pretool vs eBPF

| Layer | Responsibility |
|---|---|
| **pretool** (deny-list.json) | Code-pattern rules (TypeScript, React, hooks, naming). Intercepts the LLM agent's tool calls inside the IDE |
| **eBPF kernel** (commands.toml) | Blocking of destructive executions via `execve`. Acts at the kernel level, only for processes in the agent cgroup |

---

## 10. Check whether eBPF is active

Use the following commands to diagnose the eBPF-kernel state:

```bash
# Check whether BPF LSM is active in the kernel
cat /sys/kernel/security/lsm
# Expected: lockdown,capability,landlock,yama,apparmor,bpf

# Check whether the daemon is running
systemctl status nemesis-ebpf
# Expected: Active: active (running)

# Restart the daemon (after modifying the deny-list)
sudo systemctl restart nemesis-ebpf
# Purpose: Reloads the deny-list (commands.toml) to apply changes

# Check enforcement status (if the binary is available)
.nemesis/target/release/nemesis-ebpf-daemon --doctor
# Important fields: bpf_lsm_active, enforcement_level, can_load_bpf

# Check logs in real time
journalctl -u nemesis-ebpf -f
# Look for: [nemesis] BLOCKED or [VIOLATION] PermissionDenied
```

Done. Useful commands:
  systemctl status nemesis-ebpf         # view the eBPF daemon state
  systemctl status nemesis-cgroup-watcher # view the watcher state
  journalctl -u nemesis-ebpf -f         # view logs in real time
  sudo systemctl stop nemesis-ebpf      # stop
  sudo systemctl restart nemesis-ebpf   # restart
