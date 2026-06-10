#!/bin/bash
# ensure-ebpf-caps.sh — reaplica as Linux capabilities do nemesis-ebpf-daemon.
#
# POR QUE: `setcap` grava as capabilities no INODE do binário. Toda recompilação gera
# um inode novo, então as caps (cap_bpf, cap_perfmon, cap_sys_resource) são PERDIDAS a
# cada `cargo build`. Este script detecta a ausência e reaplica — idempotente.
#
# eBPF é Linux-only: em macOS/outros este script é no-op (a defesa fica no pretool +
# Iron Dome, que são agnósticos de SO).
#
# Requer sudo apenas quando precisa aplicar (não eleva se as caps já estão presentes).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DAEMON="${1:-$SCRIPT_DIR/../target/release/nemesis-ebpf-daemon}"
REQUIRED="cap_bpf,cap_perfmon,cap_sys_resource+eip"

if [ "$(uname -s)" != "Linux" ]; then
    echo "[ensure-caps] $(uname -s) nao-Linux — eBPF nao se aplica. Nada a fazer."
    exit 0
fi
if [ ! -x "$DAEMON" ]; then
    echo "[ensure-caps] Daemon ausente: $DAEMON (compile com cargo build --release). Pulando." >&2
    exit 0
fi
if ! command -v setcap >/dev/null 2>&1 || ! command -v getcap >/dev/null 2>&1; then
    echo "[ensure-caps] setcap/getcap ausentes — instale 'libcap2-bin'. Pulando." >&2
    exit 0
fi

current="$(getcap "$DAEMON" 2>/dev/null || true)"
if echo "$current" | grep -q "cap_bpf" && echo "$current" | grep -q "cap_perfmon"; then
    echo "[ensure-caps] OK — caps ja presentes: ${current:-<nenhuma>}"
    exit 0
fi

echo "[ensure-caps] Caps ausentes (inode novo apos rebuild). Aplicando '$REQUIRED' em:"
echo "             $DAEMON"
if [ "$(id -u)" -eq 0 ]; then
    setcap "$REQUIRED" "$DAEMON"
else
    sudo setcap "$REQUIRED" "$DAEMON"
fi

new="$(getcap "$DAEMON" 2>/dev/null || true)"
if echo "$new" | grep -q "cap_bpf"; then
    echo "[ensure-caps] Aplicado com sucesso: $new"
    exit 0
else
    echo "[ensure-caps] FALHA. Rode manualmente: sudo setcap $REQUIRED \"$DAEMON\"" >&2
    exit 1
fi
