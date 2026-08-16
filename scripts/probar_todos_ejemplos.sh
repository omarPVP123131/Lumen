#!/usr/bin/env bash
# ============================================================================
# LÚMEN — Ejecutor y Verificador Masivo de Ejemplos en Bash (Linux / macOS)
# ============================================================================

set -e

ACTION="${1:-check}"

LUMEN_BIN="target/release/lumen"
if [ ! -f "$LUMEN_BIN" ]; then
    LUMEN_BIN="target/debug/lumen"
fi
if [ ! -f "$LUMEN_BIN" ]; then
    LUMEN_BIN="/home/user/lumen/lumen"
fi

echo ""
echo "🚀 LÚMEN — VERIFICACIÓN MASIVA DE EJEMPLOS (.nv)"
echo "• Binario activo: $LUMEN_BIN"
echo "• Modo: $ACTION"
echo "══════════════════════════════════════════════════════════════"

if [ "$ACTION" = "check" ] || [ "$ACTION" = "comprobar" ]; then
    $LUMEN_BIN check -L stdlib -L stdlib/compiler examples
    exit $?
fi

PASSED=0
FAILED=0

for file in $(find examples -name "*.nv" | sort); do
    if $LUMEN_BIN run -L stdlib -L stdlib/compiler "$file" >/dev/null 2>&1; then
        PASSED=$((PASSED + 1))
        echo "  ✓ OK: $file"
    else
        FAILED=$((FAILED + 1))
        echo "  ✗ FALLÓ: $file"
    fi
done

echo "══════════════════════════════════════════════════════════════"
echo "📊 RESUMEN: $PASSED pasaron, $FAILED fallaron"
echo ""
