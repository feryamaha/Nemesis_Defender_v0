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

# ── 2c. Ping de uninstall (best-effort) ──────────────────────────────────────
# Avisa a dashboard que este install foi removido. Best-effort: falha de rede
# nao quebra o uninstall. So tenta se o publisher e a identidade existirem.
if [ -x .nemesis/bin/nemesis-publisher ] && [ -s .nemesis/telemetry/identity.json ]; then
  .nemesis/bin/nemesis-publisher --unregister >/dev/null 2>&1 || true
  say "Ping de uninstall enviado (best-effort)."
fi

# ── 3. Remover hooks de IDE criados pelo Nemesis ─────────────────────────────
# Detecta a "assinatura" do Nemesis: referência ao binário do hook ou ao .nemesis/bin.
# (Importante: arquivos com hook quebrado fazem a IDE/TUI reclamar a cada sessão.)
NEM_REF='nemesis-pretool\|nemesis-posttool\|\.nemesis/bin\|chat.hookFilesLocations'

# 3a. Arquivos de hook que o install cria 100% para o Nemesis → remove direto.
for f in .codex/hooks.json .cursor/hooks.json .devin/hooks.json \
         .gemini/hooks.json .agents/hooks.json \
         .github/hooks/nemesis-pretool-hook.json; do
  if [ -f "$f" ] && grep -q "$NEM_REF" "$f" 2>/dev/null; then
    rm -f "$f" && say "Hook removido: $f"
  fi
done
# Remove a pasta .github/hooks se ficou vazia (era só do Nemesis).
rmdir .github/hooks 2>/dev/null || true

# 3b. settings COMPARTILHADOS (podem conter SUA config): NÃO apaga automaticamente.
# Apagar config sua seria pior que avisar — então listamos para você remover à mão.
shared_left=""
for f in .claude/settings.json .openclaude/settings.json .vscode/settings.json; do
  if [ -f "$f" ] && grep -q "$NEM_REF" "$f" 2>/dev/null; then
    shared_left="$shared_left $f"
  fi
done

# ── 4. Remover a pasta .nemesis/ ─────────────────────────────────────────────
rm -rf .nemesis && ok "Pasta .nemesis/ removida."

hr
ok "NEMESIS REMOVIDO deste projeto."
hr

# ── 5. CHECKLIST FINAL — confirme que nada ficou rodando ou órfão ────────────
echo
say "CHECKLIST FINAL (confirme que não sobrou resíduo):"
echo

if [ -n "$shared_left" ]; then
  warn "Settings COMPARTILHADOS ainda têm entrada do Nemesis — remova a entrada À MÃO (preserve o resto):"
  for f in $shared_left; do printf '          - %s\n' "$f"; done
  echo
fi

cat <<'EOF'
  [1] Procurar QUALQUER resquício de hook do Nemesis (o ideal é não retornar nada):
        grep -rIl 'nemesis-pretool\|nemesis-posttool\|\.nemesis/bin\|chat.hookFilesLocations' \
          .claude .openclaude .codex .cursor .devin .gemini .agents .github .vscode 2>/dev/null
      Se listar algum arquivo, edite/remova a entrada do Nemesis nele (manualmente).

  [2] Confirmar que o daemon NÃO está mais rodando:
        pgrep -fl nemesis-defender        # vazio = ok
      Se aparecer um PID, finalize:
        pkill -f nemesis-defender

  [3] (Linux, só se você ativou o eBPF opt-in) confirmar que o serviço de kernel parou:
        systemctl is-active nemesis-ebpf  # inactive/failed = ok
      Se ainda estiver ativo:
        sudo systemctl disable --now nemesis-ebpf

  [4] Reinicie a IDE para ela parar de carregar os hooks.
      Git é seu: nada foi commitado/removido do versionamento (revise com 'git status').
EOF
echo
say "Deu algum problema na desinstalação? Me escreva: feryamaha@hotmail.com — eu dou suporte."
say "E se puder, conte o MOTIVO do uninstall: feedback positivo ou negativo me ajuda muito."
hr
