#!/bin/bash
# nemesis-build.sh — build do workspace + reativação AUTOMÁTICA das caps do eBPF.
#
# Use este wrapper no lugar de `cargo build --release --workspace`. Em Linux, após o
# build ele reaplica as capabilities do nemesis-ebpf-daemon (que caem a cada recompilação,
# pois setcap é por-inode). Em macOS/outros, a etapa de caps é no-op automaticamente.
#
# Argumentos extras são repassados ao cargo (ex.: -p nemesis-defender).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
NEMESIS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$NEMESIS_DIR"

echo "[nemesis-build] cargo build --release --workspace $*"
cargo build --release --workspace "$@"
rc=$?
if [ $rc -ne 0 ]; then
    echo "[nemesis-build] build FALHOU (rc=$rc) — caps nao reaplicadas." >&2
    exit $rc
fi

# Reativa caps do eBPF (Linux). No-op em macOS.
"$SCRIPT_DIR/ensure-ebpf-caps.sh"

echo "[nemesis-build] concluido. Lembrete: reinicie o daemon apos recompilar o defender —"
echo "                pkill -9 -f nemesis-defender; .nemesis/target/release/nemesis-defender --ensure-daemon"
