#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# Build LÚMEN WASM for WASI target (wasm32-wasip1)
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "╔════════════════════════════════════╗"
echo "║   LÚMEN WASI Build                 ║"
echo "╚════════════════════════════════════╝"
echo ""
echo "📦 Project: $PROJECT_DIR"
echo "🎯 Target:  wasm32-wasip1"
echo "📂 Crate:   lumen-wasm"
echo ""

cd "$PROJECT_DIR"

# ── Step 1: Check for WASI target ──────────────────────────
echo "1️⃣  Verificando target wasm32-wasip1..."
if ! rustup target list --installed | grep -q "wasm32-wasip1"; then
    echo "   ⚠️  Target no instalado. Instalando..."
    rustup target add wasm32-wasip1
else
    echo "   ✅ Target wasm32-wasip1 instalado"
fi

# ── Step 2: Build ─────────────────────────────────────────
echo ""
echo "2️⃣  Compilando lumen-wasm para WASI..."
cargo build -p lumen-wasm \
    --target wasm32-wasip1 \
    --no-default-features \
    --features wasi \
    --release

echo ""
echo "3️⃣  Verificando artefacto..."
WASM_FILE="$PROJECT_DIR/target/wasm32-wasip1/release/lumen_wasm.wasm"
if [ -f "$WASM_FILE" ]; then
    SIZE=$(du -h "$WASM_FILE" | cut -f1)
    echo "   ✅ WASM generado: $WASM_FILE"
    echo "   📏 Tamaño: $SIZE"
else
    echo "   ❌ ERROR: No se encontró el archivo .wasm"
    exit 1
fi

# ── Step 3: Check (quick) ─────────────────────────────────
echo ""
echo "4️⃣  Verificando con cargo check..."
cargo check -p lumen-wasm \
    --target wasm32-wasip1 \
    --no-default-features \
    --features wasi

echo ""
echo "╔════════════════════════════════════╗"
echo "║   WASI Build Completado ✅          ║"
echo "╚════════════════════════════════════╝"
echo ""
echo "Para ejecutar con un runtime WASI:"
echo "  wasmtime target/wasm32-wasip1/release/lumen_wasm.wasm"
echo "  wasmer run target/wasm32-wasip1/release/lumen_wasm.wasm"
echo "  node --experimental-wasi-unstable-preview1 ..."
