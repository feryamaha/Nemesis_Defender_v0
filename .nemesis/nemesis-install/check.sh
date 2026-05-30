#!/bin/bash
# NEMESIS Health Check
# Verifica status e saúde da instalação

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NEMESIS_ROOT="$SCRIPT_DIR"
TARGET_DIR="$NEMESIS_ROOT/target/release"

echo "========================================"
echo "NEMESIS Health Check"
echo "========================================"
echo ""

ERRORS=0

# 1. Verificar BPF LSM no kernel
echo "[1/7] Verificando BPF LSM no kernel..."
if grep -q "bpf" /sys/kernel/security/lsm 2>/dev/null; then
    echo "  ✅ BPF LSM ativo"
else
    echo "  ⚠️  BPF LSM não detectado (pode requerer reboot com params corretos)"
    ((ERRORS++))
fi

# 2. Verificar binários
echo ""
echo "[2/7] Verificando binários..."
BINARIES=(
    "nemesis-ebpf-daemon"
    "nemesis-cgroup-watcher"
    "nemesis-enforce"
    "nemesis-pretool-check"
)
for bin in "${BINARIES[@]}"; do
    if [ -f "$TARGET_DIR/$bin" ]; then
        echo "  ✅ $bin"
    else
        echo "  ❌ $bin (não encontrado)"
        ((ERRORS++))
    fi
done

# 3. Verificar capabilities
echo ""
echo "[3/7] Verificando capabilities..."
if [ -f "$TARGET_DIR/nemesis-ebpf-daemon" ]; then
    CAPS=$(getcap "$TARGET_DIR/nemesis-ebpf-daemon" 2>/dev/null)
    if echo "$CAPS" | grep -q "cap_bpf"; then
        echo "  ✅ Capabilities OK"
    else
        echo "  ⚠️  Capabilities não configuradas"
        ((ERRORS++))
    fi
else
    echo "  ❌ Binário não encontrado"
    ((ERRORS++))
fi

# 4. Verificar cgroup
echo ""
echo "[4/7] Verificando cgroup..."
if [ -d "/sys/fs/cgroup/nemesis-agent" ]; then
    echo "  ✅ /sys/fs/cgroup/nemesis-agent"
else
    echo "  ❌ Cgroup não existe"
    ((ERRORS++))
fi

# 5. Verificar serviços systemd
echo ""
echo "[5/7] Verificando serviços systemd..."
for svc in nemesis-ebpf nemesis-cgroup-watcher; do
    if systemctl is-active "$svc" &>/dev/null; then
        echo "  ✅ $svc (running)"
    elif systemctl is-enabled "$svc" &>/dev/null; then
        echo "  ⚠️  $svc (enabled mas não running)"
        ((ERRORS++))
    else
        echo "  ❌ $svc (não instalado)"
        ((ERRORS++))
    fi
done

# 6. Verificar arquivos de serviço
echo ""
echo "[6/7] Verificando arquivos de serviço..."
for svc in nemesis-ebpf.service nemesis-cgroup-watcher.service; do
    if [ -f "/etc/systemd/system/$svc" ]; then
        echo "  ✅ /etc/systemd/system/$svc"
    else
        echo "  ❌ $svc não instalado"
        ((ERRORS++))
    fi
done

# 7. Verificar logs recentes
echo ""
echo "[7/7] Verificando logs recentes (últimos 2 minutos)..."
if journalctl -u nemesis-ebpf --since "2 minutes ago" 2>/dev/null | grep -q "error\|failed"; then
    echo "  ⚠️  Erros encontrados nos logs"
    ((ERRORS++))
else
    echo "  ✅ Sem erros recentes"
fi

# Resumo
echo ""
echo "========================================"
if [ $ERRORS -eq 0 ]; then
    echo "✅ TUDO OK - Nemesis está saudável!"
else
    echo "⚠️  $ERRORS problema(s) encontrado(s)"
fi
echo "========================================"
echo ""

# Comandos úteis
echo "Comandos úteis:"
echo "  Ver logs:  journalctl -u nemesis-ebpf -f"
echo "  Status:    systemctl status nemesis-ebpf"
echo "  Reiniciar: sudo systemctl restart nemesis-ebpf"
echo ""
