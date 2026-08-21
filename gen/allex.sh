#!/bin/bash
# Baseline de ejemplos. Separa los fallos REALES de los que sólo lo son en
# esta plataforma: 6 ejemplos son Windows-only (cargan msvcrt/ws2_32/user32.dll
# por FFI) y 5 son interactivos o bucles infinitos deliberados.
# Rutas relocalizables: el script se ejecuta igual en el repo, en un clon o en
# un runner de CI. RAIZ se puede fijar desde fuera (LUMEN_RAIZ) y por defecto se
# deduce de la posicion del propio script.
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAIZ="${LUMEN_RAIZ:-$(cd "$AQUI/.." && pwd)}"
LB="${LUMEN_BIN:-$RAIZ/lumen-src/target/release/lumen}"
[ -x "$LB" ] || LB="$RAIZ/target/release/lumen"

# El script vale tanto si RAIZ es el repo (contiene examples/) como si es el
# directorio que lo envuelve (contiene lumen-src/examples/). Antes se hacia
# `cd "$RAIZ/lumen-src"` a secas: cuando esa ruta no existia el cd fallaba, el
# script seguia en el directorio actual y "funcionaba" por casualidad. Un cd
# que falla en silencio es como no tenerlo.
if [ -d "$RAIZ/examples" ]; then cd "$RAIZ"
elif [ -d "$RAIZ/lumen-src/examples" ]; then cd "$RAIZ/lumen-src"
else echo "ERROR: no encuentro examples/ desde '$RAIZ'" >&2; exit 2; fi
[ -x "$LB" ] || { echo "ERROR: binario no ejecutable: $LB" >&2; exit 2; }
ok=0; fail=0; to=0; winonly=0; deuda=0
# BUG-161: esto era `examples/*.nv`, SIN recursion: cubria 178 de 393 ficheros.
# Todo `examples/compiler/`, `junior/`, `senior/`, `jr/`, `sr/`, `real/` y
# `stress/` quedaba fuera, y ahi vivian 8 ejemplos que no parseaban. Una
# baseline que ignora la mitad del banco da una tranquilidad falsa.
for f in $(find examples -name '*.nv' | sort); do
  b=$(basename "$f" .nv)
  case "$b" in tui_puro|tui_temas_demo|clock_demo|gui_ventana|debug_parser3) to=$((to+1)); continue;; esac
  case "$b" in test_connect_direct|test_ffi_debug|test_ffi_min|test_quick_connect|test_sistema_avanzado|test_sistema_directo)
      winonly=$((winonly+1)); continue;; esac
  # Deuda conocida: estos 8 fallan por errores de tipos EN EL PROPIO EJEMPLO
  # (`__map_get espera diccionario`, `ArrayLen requires array or string`,
  # `'+' requiere numeros o textos`), no por el compilador. Se verifico con el
  # binario anterior a BUG-161: ya fallaban igual, solo que la baseline
  # recortada nunca los ejecutaba. Se cuentan aparte para no ocultarlos ni
  # bloquear el CI con algo que no es una regresion.
  case "$b" in test_isolated|test_parser2|test_parser3|test_parser4|test_parser5|crypto_sr|graficos_tilemap_sr|sistema_sr)
      timeout 20 $LB run "$f" >/dev/null 2>&1
      if [ $? -eq 0 ]; then
        ok=$((ok+1)); echo "  ARREGLADO (quitar de la lista): $b"
      else
        deuda=$((deuda+1))
      fi
      continue;;
  esac
  timeout 20 $LB run "$f" >/dev/null 2>&1
  r=$?
  if [ $r -eq 0 ]; then ok=$((ok+1)); elif [ $r -eq 124 ]; then to=$((to+1)); else fail=$((fail+1)); echo "  FALLA: $b (rc=$r)"; fi
done
echo "=== ejemplos: OK=$ok  fallan=$fail  deuda_conocida=$deuda  timeout/interactivos=$to  windows-only=$winonly ==="
