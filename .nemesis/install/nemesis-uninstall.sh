#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# nemesis-uninstall.sh — remove o Nemesis do projeto atual.
# Rode na RAIZ do projeto (onde está .nemesis/), no seu terminal NATIVO.
# Reverte o nemesis-install.sh: para o daemon, remove os hooks de IDE que apontam
# para o Nemesis e remove a pasta .nemesis/. NÃO toca em git nem no código do seu projeto.
#   Uso:  bash .nemesis/install/nemesis-uninstall.sh        (pergunta antes)
#         NEMESIS_YES=1 bash .nemesis/install/nemesis-uninstall.sh   (sem perguntar)
# ─────────────────────────────────────────────────────────────────────────────
set -eu

say()  { printf '[nemesis-uninstall] %s\n' "$*"; }
warn() { printf '\033[33m[nemesis-uninstall] %s\033[0m\n' "$*"; }
ok()   { printf '\033[32m[nemesis-uninstall] %s\033[0m\n' "$*"; }
hr()   { printf '%s\n' "------------------------------------------------------------"; }

[ -d .nemesis ] || { say "Nenhum .nemesis/ aqui ($(pwd)). Rode na raiz do projeto com Nemesis instalado."; exit 1; }

# ── Confirmação (pule com NEMESIS_YES=1) ─────────────────────────────────────
if [ "${NEMESIS_YES:-0}" != "1" ] && [ -t 0 ]; then
  printf '[nemesis-uninstall] Isto REMOVE o Nemesis deste projeto: .nemesis/ (binarios, daemon,\n'
  printf '                    sua allowlist e logs) + os hooks de IDE que apontam pra ele.\n'
  printf '                    Continuar? [y/N]: '
  read -r ans || ans=""
  case "$ans" in y|Y|s|S) ;; *) say "Abortado."; exit 0 ;; esac
fi

# ── 1. Parar o daemon (se estiver rodando) ───────────────────────────────────
if [ -x .nemesis/bin/nemesis-defender ]; then
  .nemesis/bin/nemesis-defender --stop >/dev/null 2>&1 || true
fi
if [ -f .nemesis/runtime/defender.pid ]; then
  pid="$(cat .nemesis/runtime/defender.pid 2>/dev/null || true)"
  if [ -n "${pid:-}" ] && kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
    say "Daemon (pid $pid) parado."
  fi
fi

# ── 2. eBPF (Linux, OPT-IN) — desabilita o serviço de kernel ANTES de apagar .nemesis/ ───────
if [ -f .nemesis/ebpf-kernel/install-service.sh ]; then
  sudo bash .nemesis/ebpf-kernel/install-service.sh --uninstall >/dev/null 2>&1 \
    || sudo systemctl disable --now nemesis-ebpf >/dev/null 2>&1 \
    || warn "Se você instalou o serviço eBPF, remova-o manualmente (sudo systemctl disable --now nemesis-ebpf)."
fi

# ── 3. Remover hooks de IDE que apontam para o Nemesis ───────────────────────
# Arquivos criados 100% pelo Nemesis: remove se referenciarem 'nemesis'.
for f in .codex/hooks.json .cursor/hooks.json .devin/hooks.json \
         .gemini/hooks.json .agents/hooks.json \
         .github/hooks/nemesis-pretool-hook.json; do
  if [ -f "$f" ] && grep -q "nemesis" "$f" 2>/dev/null; then
    rm -f "$f" && say "Hook removido: $f"
  fi
done
# Arquivos COMPARTILHADOS (podem ter config sua): NÃO apaga — só avisa.
for f in .claude/settings.json .openclaude/settings.json .vscode/settings.json; do
  if [ -f "$f" ] && grep -q "nemesis" "$f" 2>/dev/null; then
    warn "Edite à mão p/ remover a entrada de hook do Nemesis (preservando o resto): $f"
  fi
done

# ── 4. Remover a pasta .nemesis/ ─────────────────────────────────────────────
rm -rf .nemesis && ok "Pasta .nemesis/ removida."

hr
ok "NEMESIS REMOVIDO deste projeto."
hr
cat <<'EOF'
  - Reinicie a IDE para os hooks pararem de carregar.
  - Git: nada foi commitado/removido do versionamento (git é exclusivamente seu).
    Revise com 'git status' e remova o que quiser do controle de versão.
EOF
