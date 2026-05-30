#!/bin/bash
# NEMESIS Auto-Installer
# Executa instalação completa automaticamente
# Uso: sudo ./.nemesis/nemesis-install.sh

set -e

# Detectar caminhos automaticamente
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NEMESIS_ROOT="$SCRIPT_DIR"
PROJECT_ROOT="$(dirname "$NEMESIS_ROOT")"
TARGET_DIR="$NEMESIS_ROOT/target/release"

echo "========================================"
echo "NEMESIS Auto-Installer"
echo "========================================"
echo ""
echo "Projeto detectado: $PROJECT_ROOT"
echo "Nemesis root: $NEMESIS_ROOT"
echo ""

# Verificar root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ Execute com sudo:"
    echo "   sudo ./.nemesis/nemesis-install.sh"
    exit 1
fi

# Verificar pré-requisitos
echo "[1/8] Verificando pré-requisitos..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo não instalado"
    echo "   Instale: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo "✅ Cargo encontrado"

# Compilar workspace principal
echo ""
echo "[2/8] Compilando workspace principal (17 binários)..."
echo "   Isso pode levar alguns minutos..."
cd "$NEMESIS_ROOT"
cargo build --release 2>&1 | tail -5
echo "✅ Workspace compilado"

# Compilar eBPF daemon
echo ""
echo "[3/8] Compilando eBPF daemon..."
cargo build --release -p nemesis-ebpf-kernel --bin nemesis-ebpf-daemon 2>&1 | tail -3
echo "✅ eBPF daemon compilado"

# Compilar cgroup-watcher
echo ""
echo "[4/8] Compilando cgroup-watcher..."
cargo build --release -p nemesis-ebpf-kernel --bin nemesis-cgroup-watcher 2>&1 | tail -3
echo "✅ Cgroup-watcher compilado"

# Configurar capabilities
echo ""
echo "[5/8] Configurando capabilities..."
if [ -f "$TARGET_DIR/nemesis-ebpf-daemon" ]; then
    setcap cap_bpf,cap_perfmon,cap_sys_resource+eip "$TARGET_DIR/nemesis-ebpf-daemon"
    echo "✅ Capabilities configuradas"
else
    echo "❌ Binário nemesis-ebpf-daemon não encontrado"
    exit 1
fi

# Copiar serviços systemd
echo ""
echo "[6/8] Instalando serviços systemd..."
cp "$NEMESIS_ROOT/ebpf-kernel/nemesis-ebpf.service" /etc/systemd/system/
cp "$NEMESIS_ROOT/ebpf-kernel/nemesis-cgroup-watcher.service" /etc/systemd/system/
systemctl daemon-reload
echo "✅ Serviços instalados"

# Criar cgroup
echo ""
echo "[7/8] Criando cgroup nemesis-agent..."
mkdir -p /sys/fs/cgroup/nemesis-agent
echo "✅ Cgroup criado"

# Habilitar e iniciar serviços
echo ""
echo "[8/8] Iniciando serviços..."
systemctl enable nemesis-ebpf --now 2>/dev/null || systemctl enable nemesis-ebpf
systemctl enable nemesis-cgroup-watcher --now 2>/dev/null || systemctl enable nemesis-cgroup-watcher
systemctl start nemesis-ebpf
systemctl start nemesis-cgroup-watcher
echo "✅ Serviços iniciados"

# Resumo
echo ""
echo "========================================"
echo "✅ INSTALAÇÃO CONCLUÍDA!"
echo "========================================"
echo ""
echo "Binários em: $TARGET_DIR"
echo ""
echo "Serviços ativos:"
systemctl is-active nemesis-ebpf && echo "  ✅ nemesis-ebpf" || echo "  ❌ nemesis-ebpf"
systemctl is-active nemesis-cgroup-watcher && echo "  ✅ nemesis-cgroup-watcher" || echo "  ❌ nemesis-cgroup-watcher"
echo ""
echo "Verifique saúde: ./.nemesis/check-nemesis.sh"
echo ""
