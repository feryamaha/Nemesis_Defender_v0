#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
NEMESIS_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
NEMESIS_USER=$(stat -c '%U' "$NEMESIS_ROOT")
SYSTEMD_DIR="/etc/systemd/system"

if [ "$(id -u)" -ne 0 ]; then
    echo "Este script precisa ser executado com sudo."
    echo "Uso: sudo bash $0"
    exit 1
fi

echo "[1/5] Gerando service files com paths do projeto atual..."
echo "  NEMESIS_ROOT=$NEMESIS_ROOT"

for service in nemesis-ebpf.service nemesis-cgroup-watcher.service; do
    sed -e "s|{{NEMESIS_ROOT}}|$NEMESIS_ROOT|g" \
        -e "s|{{NEMESIS_USER}}|$NEMESIS_USER|g" \
        "$SCRIPT_DIR/$service" > "/tmp/$service"
    cp "/tmp/$service" "$SYSTEMD_DIR/$service"
    echo "  -> $SYSTEMD_DIR/$service"
done

echo "[2/5] Configurando capabilities do binario eBPF..."
if [ -f "$NEMESIS_ROOT/.nemesis/target/release/nemesis-ebpf-daemon" ]; then
    setcap cap_bpf,cap_perfmon,cap_sys_resource+eip \
        "$NEMESIS_ROOT/.nemesis/target/release/nemesis-ebpf-daemon"
    echo "  setcap OK: $NEMESIS_ROOT/.nemesis/target/release/nemesis-ebpf-daemon"
else
    echo "  (aviso: binario release nao encontrado, setcap adiado — compile com cargo build --release)"
fi

echo "[3/5] Recarregando systemd..."
systemctl daemon-reload

echo "[4/5] Habilitando services no boot..."
systemctl enable nemesis-ebpf.service
systemctl enable nemesis-cgroup-watcher.service

echo "[5/5] Iniciando services..."
systemctl start nemesis-ebpf.service
systemctl start nemesis-cgroup-watcher.service

echo ""
echo "Pronto. Comandos uteis:"
echo "  systemctl status nemesis-ebpf         # ver estado do daemon eBPF"
echo "  systemctl status nemesis-cgroup-watcher # ver estado do watcher"
echo "  journalctl -u nemesis-ebpf -f         # ver logs em tempo real"
echo "  sudo systemctl stop nemesis-ebpf      # parar"
echo "  sudo systemctl restart nemesis-ebpf   # reiniciar"