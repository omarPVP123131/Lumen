#!/usr/bin/env bash
# ============================================================================
# LÚMEN — Instalador de Git Hooks para Linux / macOS / Termux (install-hooks.sh)
# ============================================================================

set -e

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$REPO_ROOT" ]; then
    echo "Error: No estás dentro de un repositorio Git."
    exit 1
fi

HOOKS_DIR="$REPO_ROOT/.git/hooks"
SOURCE_DIR="$REPO_ROOT/scripts/git-hooks"

mkdir -p "$HOOKS_DIR"

for hook in "$SOURCE_DIR"/*; do
    if [ -f "$hook" ]; then
        name="$(basename "$hook")"
        cp "$hook" "$HOOKS_DIR/$name"
        chmod +x "$HOOKS_DIR/$name"
        echo "  ✓ Hook instalado: $name"
    fi
done

echo "🎉 ¡Git hooks de LÚMEN (pre-commit, pre-push, post-commit) instalados con éxito!"
