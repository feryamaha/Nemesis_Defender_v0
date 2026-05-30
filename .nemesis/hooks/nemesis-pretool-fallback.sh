#!/bin/bash
# Nemesis Pretool Fallback — bloqueia TUDO se o binario nao existe
# Este script e o safety net final — fail-closed por design

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$SCRIPT_DIR/../target/release/nemesis-pretool-check-unix"

if [ -x "$BINARY" ]; then
    # Binario existe: usar ele
    exec "$BINARY" "$@"
else
    # Binario NAO existe: BLOQUEAR TUDO
    echo "[NEMESIS FALLBACK] BINARIO NAO ENCONTRADO: $BINARY" >&2
    echo "[NEMESIS FALLBACK] BLOQUEANDO POR SEGURANCA — fail-closed" >&2
    echo "[NEMESIS FALLBACK] Recompile: cd .nemesis && cargo build --release" >&2
    exit 2
fi
