# NEMESIS — Compilação de Binários (Manual de Instalação)

> **Propósito:** Guia completo para compilar todos os binários do Nemesis Workflow Enforcement Framework para portar o sistema para outro projeto ou computador.

---

## 📋 Pré-requisitos

### Sistema Operacional
- **Linux** com kernel 5.8+ (BPF LSM requerido)
- Recomendado: Ubuntu 22.04+ ou equivalente

### Ferramentas Necessárias
```bash
# Instalar Rust (rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Instalar dependências do sistema (Ubuntu/Debian)
sudo apt update
sudo apt install -y \
    build-essential \
    libbpf-dev \
    libclang-dev \
    llvm \
    clang \
    bpftool \
    pkg-config \
    libssl-dev

# Instalar target necessário para compilação BPF
rustup target add x86_64-unknown-linux-gnu
```

---

## 🏗️ Estrutura do Workspace

O Nemesis é organizado como um workspace Cargo com múltiplos crates:

```
.nemesis/
├── Cargo.toml              # Workspace principal + binários hooks/CLI
├── ast-linters/            # AST semantic analysis (tree-sitter)
│   └── Cargo.toml
└── ebpf-kernel/            # eBPF kernel enforcement
    └── Cargo.toml
```

---

## 📦 Binários a Compilar

### 1. Binários Principais do Workspace (nemesis)

Local: `/Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/Cargo.toml`

| Binário | Propósito |
|---------|-----------|
| `nemesis-pretool-check` | Hook de verificação pretool |
| `nemesis-pretool-check-unix` | Hook Unix específico |
| `nemesis-pretool-check-windows` | Hook Windows específico |
| `pre-edit-hook` | Hook pré-edição |
| `debug-hook-env` | Debug de ambiente |
| `nemesis-enforce` | CLI de enforcement |
| `nemesis-install-hooks` | Instalador de hooks |
| `nemesis-analysis` | Análise de código |
| `nemesis-pretool-hook` | Hook pretool principal |
| `run-nemesis-analysis` | Runner de análise |
| `nemesis-scope` | Gerenciamento de escopo |
| `test-all-workflows` | Testador de workflows |
| `nemesis-validate` | Validador |
| `workflow-step-tracker` | Tracker de passos |
| `nemesis-install` | Instalador do Nemesis |
| `ast-workflow-runner` | Runner de workflows AST |
| `nemesis-lsp` | Servidor LSP |

### 2. Binários do eBPF Kernel

Local: `/Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/ebpf-kernel/Cargo.toml`

| Binário | Propósito |
|---------|-----------|
| `nemesis-ebpf-daemon` | Daemon eBPF principal |
| `nemesis-cgroup-watcher` | Watcher de cgroups para múltiplos agentes |

### 3. AST Linters (Biblioteca)

Local: `/Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/ast-linters/Cargo.toml`

- Compila como biblioteca (lib), não possui binários executáveis
- Usado pelos outros componentes do Nemesis

---

## 🔨 Comandos de Compilação

### Compilação Completa (Release)

```bash
# Navegar para o diretório .nemesis
cd /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis

# Compilar todos os binários do workspace principal
cargo build --release

# Compilar binários específicos do eBPF kernel
cargo build --release -p nemesis-ebpf-kernel --bin nemesis-ebpf-daemon
cargo build --release -p nemesis-ebpf-kernel --bin nemesis-cgroup-watcher
```

### Compilação por Componente

#### Workspace Principal (todos os hooks e CLI)
```bash
cd /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis
cargo build --release
```

#### eBPF Daemon
```bash
cd /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis
cargo build --release -p nemesis-ebpf-kernel --bin nemesis-ebpf-daemon
```

#### Cgroup Watcher
```bash
cd /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis
cargo build --release -p nemesis-ebpf-kernel --bin nemesis-cgroup-watcher
```

#### AST Linters (biblioteca)
```bash
cd /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis
cargo build --release -p ast-linters
```

---

## 📂 Locais dos Binários Compilados

Após compilação, os binários estarão em:

```
.nemesis/target/release/
├── nemesis-pretool-check
├── nemesis-pretool-check-unix
├── nemesis-pretool-check-windows
├── pre-edit-hook
├── debug-hook-env
├── nemesis-enforce
├── nemesis-install-hooks
├── nemesis-analysis
├── nemesis-pretool-hook
├── run-nemesis-analysis
├── nemesis-scope
├── test-all-workflows
├── nemesis-validate
├── workflow-step-tracker
├── nemesis-install
├── ast-workflow-runner
├── nemesis-lsp
├── nemesis-ebpf-daemon      (de ebpf-kernel)
└── nemesis-cgroup-watcher   (de ebpf-kernel)
```

---

## ⚙️ Instalação e Ativação

### 1. Configurar permissões (eBPF)

```bash
# Definir capabilities para o daemon eBPF
sudo setcap cap_bpf,cap_perfmon,cap_sys_resource+eip \
    /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/target/release/nemesis-ebpf-daemon

# Verificar
getcap /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/target/release/nemesis-ebpf-daemon
# Esperado: cap_bpf,cap_perfmon,cap_sys_resource+eip
```

### 2. Instalar arquivos de serviço systemd

```bash
# Copiar serviço eBPF
sudo cp /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/ebpf-kernel/nemesis-ebpf.service \
    /etc/systemd/system/

# Copiar serviço cgroup-watcher
sudo cp /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/ebpf-kernel/nemesis-cgroup-watcher.service \
    /etc/systemd/system/

# Recarregar systemd
sudo systemctl daemon-reload
```

### 3. Criar cgroup nemesis-agent

```bash
sudo mkdir -p /sys/fs/cgroup/nemesis-agent
```

### 4. Iniciar serviços

```bash
# Habilitar e iniciar eBPF daemon
sudo systemctl enable nemesis-ebpf
sudo systemctl start nemesis-ebpf

# Habilitar e iniciar cgroup-watcher
sudo systemctl enable nemesis-cgroup-watcher
sudo systemctl start nemesis-cgroup-watcher
```

### 5. Verificar status

```bash
# Verificar eBPF daemon
systemctl status nemesis-ebpf

# Verificar cgroup-watcher
systemctl status nemesis-cgroup-watcher

# Verificar BPF LSM no kernel
cat /sys/kernel/security/lsm
# Esperado: ...,bpf,...

# Verificar logs
journalctl -u nemesis-ebpf -n 20
journalctl -u nemesis-cgroup-watcher -n 20
```

---

## 🔄 Atualização (Recompilação)

Para recompilar após modificações:

```bash
cd /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis

# Limpar build anterior (opcional)
cargo clean

# Recompilar tudo
cargo build --release
cargo build --release -p nemesis-ebpf-kernel --bin nemesis-ebpf-daemon
cargo build --release -p nemesis-ebpf-kernel --bin nemesis-cgroup-watcher

# Reiniciar serviços
sudo systemctl restart nemesis-ebpf
sudo systemctl restart nemesis-cgroup-watcher
```

---

## 🛠️ Comandos Úteis de Diagnóstico

```bash
# Status do enforcement
cd /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis
./target/release/nemesis-ebpf-daemon --doctor

# Self-test
cd /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis
./target/release/nemesis-ebpf-daemon --self-test

# Verificar deny-list carregada
cat /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/ebpf-kernel/denylist-ebpf/commands.toml

# Logs em tempo real
journalctl -u nemesis-ebpf -f
journalctl -u nemesis-cgroup-watcher -f
```

---

## 📝 Checklist de Instalação

- [ ] Rust e dependências do sistema instalados
- [ ] Código fonte do Nemesis copiado para `.nemesis/`
- [ ] Todos os binários compilados com `cargo build --release`
- [ ] Capabilities definidas para `nemesis-ebpf-daemon`
- [ ] Arquivos de serviço systemd copiados
- [ ] Cgroup `nemesis-agent` criado
- [ ] Serviços systemd habilitados e iniciados
- [ ] Status verificado e BPF LSM ativo no kernel

---

## 🆘 Troubleshooting

### Erro: "failed to read denylist-ebpf/commands.toml"
Certifique-se de executar a partir do diretório `.nemesis/` ou que os arquivos de configuração existam.

### Erro: "No such file or directory" para vmlinux.h
Regenerar o header BPF:
```bash
sudo bpftool btf dump file /sys/kernel/btf/vmlinux format c > \
    /Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/ebpf-kernel/ebpf/vmlinux.h
```

### Erro: BPF LSM não ativo
Verificar kernel:
```bash
cat /boot/config-$(uname -r) | grep BPF
cat /boot/config-$(uname -r) | grep LSM
```
Deve mostrar `CONFIG_BPF=y` e `CONFIG_BPF_LSM=y`.

---

**Documento gerado em:** 2026-05-06  
**Versão do Nemesis:** 8.2.0  
**Local do projeto:** `/Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis`
