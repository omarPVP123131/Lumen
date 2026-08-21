#!/bin/bash
# Runner del fuzzer de structs/listas/`prestado mut` (fuzz6.py, zona BUG-008).
#
# Tres comprobaciones por caso:
#   ORACULO  la VM contra la salida calculada en Python (detecta que ambos
#            backends coincidan en un resultado incorrecto — el modo de fallo
#            del BUG-008).
#   DIFF     la VM contra el binario nativo.
#   CRASH    señales del binario nativo (rc >= 132).
# Rutas relocalizables: el script se ejecuta igual en el repo, en un clon o en
# un runner de CI. RAIZ se puede fijar desde fuera (LUMEN_RAIZ) y por defecto se
# deduce de la posicion del propio script.
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAIZ="${LUMEN_RAIZ:-$(cd "$AQUI/.." && pwd)}"
LB="${LUMEN_BIN:-$RAIZ/lumen-src/target/release/lumen}"
[ -x "$LB" ] || LB="$RAIZ/target/release/lumen"


DIR="$AQUI/fz10"
BIN=/tmp/fz10bin
cd "$DIR" || exit 1
mkdir -p "$BIN"
T=0; O=0; D=0; C=0; R=0
for f in p*.nv; do
  b=${f%.nv}; T=$((T + 1))
  esp=$(cat "$b.exp")
  vm=$(timeout 10 "$LB" run "$f" 2>&1)
  if [ "$vm" != "$esp" ]; then
    O=$((O + 1))
    echo "ORACULO $b [$(cat "$b.tag")] esperado=[$(echo "$esp" | tr '\n' '~')] vm=[$(echo "$vm" | tr '\n' '~')]"
  fi
  if timeout 90 "$LB" build "$f" --native -o "$BIN/$b" >/dev/null 2>&1; then
    nat=$(cd /tmp && timeout 10 "$BIN/$b" 2>&1); nrc=$?
    if [ "$vm" != "$nat" ]; then
      D=$((D + 1))
      echo "DIFF $b [$(cat "$b.tag")] vm=[$(echo "$vm" | tr '\n' '~')] nat=[$(echo "$nat" | tr '\n' '~')]"
    fi
    if [ $nrc -ge 132 ]; then
      C=$((C + 1)); echo "CRASH $b rc=$nrc"
    fi
    rm -f "$BIN/$b"
  else
    R=$((R + 1))
  fi
done
echo "== total=$T oraculo=$O diffs=$D crashes=$C rechazados_aot=$R =="
