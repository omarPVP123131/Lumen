#!/usr/bin/env bash
# ============================================================================
# LÚMEN v3.5.5 — Verificación de fixpoint self-hosting (compiler_v4)
#
# Cadena: compiler_v4.nvc (Rust) compila compiler_v4.nv  -> /tmp/v4_self.nvc
#         /tmp/v4_self.nvc recompila compiler_v4.nv      -> /tmp/v4_self2.nvc
#         v4_self.nvc == v4_self2.nvc (byte-identical)   -> FIXPOINT OK
# Además: check funcional — el self-compilado compila selfhost_probe.nv -> 42.
#
# Uso (desde la raíz del repo):  bash scripts/fixpoint_3_5_5.sh
# ============================================================================
set -u
LUMEN=./target/release/lumen
[ -x "$LUMEN" ] || LUMEN=./target/debug/lumen
SRC=stdlib/compiler/compiler_v4.nv
SELF1=/tmp/v4_self.nvc
SELF2=/tmp/v4_self2.nvc

echo "== Stage 1: compiler_v4.nvc compila $SRC =="
if [ ! -f "$SELF1" ]; then
  printf '%s\n%s\n' "$SRC" "$SELF1" > stdlib/compiler/target.txt
  "$LUMEN" run stdlib/compiler/compiler_v4.nvc | tail -2
fi
[ -f "$SELF1" ] || { echo "❌ Stage 1 no produjo $SELF1"; exit 1; }
echo "   OK: $SELF1 ($(wc -c < "$SELF1") bytes)"

echo "== Stage 2: $SELF1 recompila $SRC =="
printf '%s\n%s\n' "$SRC" "$SELF2" > stdlib/compiler/target.txt
"$LUMEN" run "$SELF1" | tail -2
[ -f "$SELF2" ] || { echo "❌ Stage 2 no produjo $SELF2"; exit 1; }
echo "   OK: $SELF2 ($(wc -c < "$SELF2") bytes)"

echo "== Comparación byte a byte =="
if cmp -s "$SELF1" "$SELF2"; then
  echo "✅ FIXPOINT CONFIRMADO: v4_self.nvc y v4_self2.nvc byte-IDENTICAL"
  sha256sum "$SELF1" "$SELF2"
else
  echo "❌ FIXPOINT ROTO: los bytes difieren"
  exit 1
fi

echo "== Check funcional: el self-compilado compila la probe prestado mut =="
printf 'fuzz/selfhost_probe.nv\nfuzz/selfhost_probe_fix.nvc\n' > stdlib/compiler/target.txt
"$LUMEN" run "$SELF1" >/dev/null 2>&1
OUT=$("$LUMEN" run fuzz/selfhost_probe_fix.nvc 2>&1)
if [ "$OUT" = "42" ]; then
  echo "✅ probe vía self-compilado: 42 (write-back prestado mut correcto)"
else
  echo "❌ probe vía self-compilado devolvió: '$OUT' (esperado 42)"
  exit 1
fi

echo "== Higiene: target.txt restaurado =="
printf 'stdlib/coleccion.nv\nstdlib/compile.nvc\n' > stdlib/compiler/target.txt
echo "🎉 Fixpoint v3.5.5 verificado."
