#!/bin/bash
set -euo pipefail

# ============================================================================
# Ativação UNIFICADA do eBPF-kernel do Nemesis — um único `sudo`.
# Cobre: BPF LSM no GRUB (1×) + setcap + cgroup + systemd (enable/start).
# O cgroup, o setcap e a atribuição de PID já são feitos pelo próprio unit
# (ExecStartPre/ExecStartPost/AmbientCapabilities) a cada boot.
# A ÚNICA etapa que exige reboot é o GRUB lsm=...,bpf (parâmetro de kernel).
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
NEMESIS_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
NEMESIS_USER=$(stat -c '%U' "$NEMESIS_ROOT")
SYSTEMD_DIR="/etc/systemd/system"
EBPF_BIN="$NEMESIS_ROOT/.nemesis/target/release/nemesis-ebpf-daemon"
REBOOT_REQUIRED=0

if [ "$(id -u)" -ne 0 ]; then
    echo "Este script precisa ser executado com sudo."
    echo "Uso: sudo bash $0"
    exit 1
fi

# Pré-flight: o binário release precisa existir (compilar antes).
if [ ! -f "$EBPF_BIN" ]; then
    echo "ERRO: binário não encontrado: $EBPF_BIN"
    echo "      Compile primeiro: cd $NEMESIS_ROOT/.nemesis && cargo build --release -p nemesis-ebpf-kernel"
    exit 1
fi

echo "[0/6] Verificando BPF LSM (necessário para lsm/bprm_check_security e lsm/socket_connect)..."
if grep -qw bpf /sys/kernel/security/lsm 2>/dev/null; then
    echo "  BPF LSM já ativo no boot atual. OK."
else
    echo "  BPF LSM NÃO está ativo. Tentando habilitar via GRUB (idempotente, com backup)..."
    GRUB_FILE="/etc/default/grub"
    if [ ! -f "$GRUB_FILE" ]; then
        echo "  AVISO: $GRUB_FILE não encontrado — habilite 'bpf' no lsm= do seu bootloader manualmente."
        REBOOT_REQUIRED=1
    elif grep -qE '^GRUB_CMDLINE_LINUX_DEFAULT=.*\blsm=' "$GRUB_FILE"; then
        # Já existe um lsm= customizado — NÃO editar automaticamente (risco de mangle).
        if grep -qE '^GRUB_CMDLINE_LINUX_DEFAULT=.*\blsm=[^"]*\bbpf\b' "$GRUB_FILE"; then
            echo "  GRUB já contém 'bpf' no lsm=. Apenas reboot pendente para ativar."
        else
            echo "  GRUB tem um lsm= customizado SEM 'bpf'. Edite manualmente adicionando ',bpf' ao lsm= e rode 'sudo update-grub'."
        fi
        REBOOT_REQUIRED=1
    else
        # Caso limpo: sem lsm= definido — adicionar o conjunto completo com bpf.
        cp -n "$GRUB_FILE" "$GRUB_FILE.nemesis.bak"
        echo "  backup: $GRUB_FILE.nemesis.bak"
        sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT="\(.*\)"/GRUB_CMDLINE_LINUX_DEFAULT="\1 lsm=lockdown,capability,landlock,yama,apparmor,bpf"/' "$GRUB_FILE"
        echo "  lsm=...,bpf adicionado ao GRUB. Rodando update-grub..."
        update-grub
        REBOOT_REQUIRED=1
    fi
fi

echo "[1/6] Gerando service files com paths do projeto atual..."
echo "  NEMESIS_ROOT=$NEMESIS_ROOT"
for service in nemesis-ebpf.service nemesis-cgroup-watcher.service; do
    sed -e "s|{{NEMESIS_ROOT}}|$NEMESIS_ROOT|g" \
        -e "s|{{NEMESIS_USER}}|$NEMESIS_USER|g" \
        "$SCRIPT_DIR/$service" > "/tmp/$service"
    cp "/tmp/$service" "$SYSTEMD_DIR/$service"
    echo "  -> $SYSTEMD_DIR/$service"
done

echo "[2/6] Configurando capabilities do binario eBPF..."
setcap cap_bpf,cap_perfmon,cap_sys_resource+eip "$EBPF_BIN"
echo "  setcap OK: $EBPF_BIN"

echo "[3/6] Recarregando systemd..."
systemctl daemon-reload

echo "[4/6] Habilitando services no boot..."
systemctl enable nemesis-ebpf.service
systemctl enable nemesis-cgroup-watcher.service

if [ "$REBOOT_REQUIRED" -eq 1 ]; then
    echo "[5/6] Reboot pendente (BPF LSM) — services HABILITADOS mas NÃO iniciados agora."
    echo "      Iniciariam sem o LSM ativo e falhariam no attach. Subirão após o reboot."
    echo "[6/6] Configuração concluída."
    echo ""
    echo "  ====================================================================="
    echo "  >>> REBOOT NECESSÁRIO para ativar o BPF LSM (lsm=...,bpf no kernel). <<<"
    echo "  Após reiniciar:  sudo reboot"
    echo "  Os services nemesis-ebpf e nemesis-cgroup-watcher subirão sozinhos."
    echo "  ====================================================================="
else
    echo "[5/6] Iniciando services (BPF LSM já ativo)..."
    systemctl restart nemesis-ebpf.service
    systemctl restart nemesis-cgroup-watcher.service
    echo "[6/6] Concluído — eBPF ativo (exec-block + egress)."
fi

echo ""
echo "Comandos uteis:"
echo "  systemctl status nemesis-ebpf            # estado do daemon eBPF"
echo "  systemctl status nemesis-cgroup-watcher  # estado do watcher"
echo "  journalctl -u nemesis-ebpf -f            # logs em tempo real"
echo "  sudo systemctl restart nemesis-ebpf      # reiniciar (aplica rebuild do binário)"
echo ""
echo "Egress allowlist: edite .nemesis/ebpf-kernel/denylist-ebpf/egress.toml"
echo "  enforce=true + allowlist=[\"CIDR:porta\"], depois: sudo systemctl restart nemesis-ebpf"
