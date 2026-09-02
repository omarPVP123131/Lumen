#!/usr/bin/env bash
# v3.5.17 — Sweep de paridad VM vs C vs Cranelift sobre ejemplos
# deterministas. Compara salida exacta (stdout normalizado).
set -u
cd "$(dirname "$0")/.."
B=target/release/lumen
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

FILES="examples/hello.nv examples/arrays.nv examples/break.nv examples/condicional.nv \
examples/continue.nv examples/destructuring.nv examples/enums.nv examples/foreach.nv \
examples/func.nv examples/lambda.nv examples/loop.nv examples/match.nv examples/math.nv \
examples/opcion.nv examples/params_default.nv examples/resultado.nv examples/strings.nv \
examples/structs.nv examples/tuplas.nv examples/recursion.nv examples/jr_concurrencia.nv \
fuzz/hilos_cranelift.nv fuzz/hilos_stress.nv fuzz/hilos_canal.nv fuzz/capturas_reg.nv fuzz/glob_write.nv fuzz/cap_param.nv \
fuzz/cr_colision_global.nv fuzz/cr_divmod_entero.nv fuzz/arrays_nativos.nv"

OK=0; DIVERGE=0; RECHAZO_CR=0; RECHAZO_C=0; FALLO_VM=0
for f in $FILES; do
  [ -f "$f" ] || continue
  base=$(basename "$f" .nv)
  timeout 30 $B run -L stdlib -L stdlib/compiler "$f" > "$TMP/$base.vm" 2>/dev/null
  vm_rc=$?
  if [ $vm_rc -ne 0 ]; then
    echo "✗ VM falla: $f"; FALLO_VM=$((FALLO_VM+1)); continue
  fi

  # backend C
  c_ok=1
  cp "$f" "$TMP/$base.nv"
  if timeout 60 $B build --c "$TMP/$base.nv" > /dev/null 2>&1; then
    timeout 30 "$TMP/$base" > "$TMP/$base.c" 2>/dev/null || c_ok=0
    rm -f "$TMP/$base"
  else
    c_ok=0; RECHAZO_C=$((RECHAZO_C+1)); echo "  (C no compila: $f)"
  fi

  # backend Cranelift
  cr_ok=1
  if timeout 60 $B build --rust "$TMP/$base.nv" > /dev/null 2>&1; then
    timeout 30 "$TMP/$base" > "$TMP/$base.cr" 2>/dev/null || cr_ok=0
    rm -f "$TMP/$base"
  else
    cr_ok=0; RECHAZO_CR=$((RECHAZO_CR+1)); echo "  (Cranelift rechaza: $f)"
  fi

  d=0
  [ $c_ok -eq 1 ] && ! diff -q "$TMP/$base.vm" "$TMP/$base.c" >/dev/null && { echo "✗ DIVERGE C: $f"; d=1; }
  [ $cr_ok -eq 1 ] && ! diff -q "$TMP/$base.vm" "$TMP/$base.cr" >/dev/null && { echo "✗ DIVERGE CRANELIFT: $f"; d=1; }
  if [ $d -eq 0 ]; then OK=$((OK+1)); else DIVERGE=$((DIVERGE+1)); fi
done

echo "═══════════════════════════════════════════════"
echo "Paridad: $OK OK | $DIVERGE divergen | C no compila: $RECHAZO_C | Cranelift rechaza: $RECHAZO_CR | VM falla: $FALLO_VM"
