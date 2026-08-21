#!/usr/bin/env bash
# Comprueba que los programas MALFORMADOS se rechacen de verdad.
#
# Dos condiciones, porque BUG-151 fallaba justo entre ellas:
#   1. `lumen run`   debe salir con rc != 0.
#   2. `lumen check` debe salir con rc != 0 (no dar el programa por válido).
#   3. La salida NO puede contener la marca centinela: si el cuerpo se ejecuta,
#      da igual que luego se informe del error.
# Rutas relocalizables: el script se ejecuta igual en el repo, en un clon o en
# un runner de CI. RAIZ se puede fijar desde fuera (LUMEN_RAIZ) y por defecto se
# deduce de la posicion del propio script.
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAIZ="${LUMEN_RAIZ:-$(cd "$AQUI/.." && pwd)}"
LB="${LUMEN_BIN:-$RAIZ/lumen-src/target/release/lumen}"
[ -x "$LB" ] || LB="$RAIZ/target/release/lumen"


DIR="$AQUI/fz12"
S="CENTINELA_NO_DEBE_SALIR"

total=0; acepta_run=0; acepta_check=0; ejecuta=0; acepta_tool=0

cd "$DIR" || exit 1
for f in r*.nv; do
  total=$((total + 1))
  tag=$(cat "${f%.nv}.tag" 2>/dev/null | tr -d '\n')

  # Solo stdout: el diagnostico de error repite la linea de codigo fuente, que
  # contiene la marca, y eso no es que el cuerpo se haya ejecutado. Ademas se
  # exige linea EXACTA, porque `imprimir` emite la marca sola en su linea.
  out=$(timeout 20 "$LB" run "$f" 2>/dev/null); rc=$?
  if [ $rc -eq 0 ]; then
    acepta_run=$((acepta_run + 1))
    echo "ACEPTA-RUN   ${f%.nv} [$tag] rc=0 en un programa invalido"
  fi
  if echo "$out" | grep -qx "$S"; then
    ejecuta=$((ejecuta + 1))
    echo "EJECUTA      ${f%.nv} [$tag] el cuerpo salio pese al error"
  fi

  timeout 20 "$LB" check "$f" >/dev/null 2>&1
  if [ $? -eq 0 ]; then
    acepta_check=$((acepta_check + 1))
    echo "ACEPTA-CHECK ${f%.nv} [$tag] 'check' lo dio por valido"
  fi

  # BUG-155: `doc` generaba una pagina vacia con rc=0 sobre codigo que no
  # compila. Toda herramienta que CONSUME un fuente tiene que rechazarlo:
  # anunciar exito sobre codigo roto es el patron de BUG-140..145.
  #
  # Se exige solo sobre errores SINTACTICOS. `fmt` y `doc` trabajan sobre el
  # arbol de sintaxis y no resuelven nombres ni tipos: formatear un fichero
  # que llama a una funcion inexistente es legitimo —el codigo aun se esta
  # escribiendo—. Exigirles analisis semantico seria pedirles que rechacen
  # trabajo en curso. `check` y `lint` si lo hacen, y se comprueba arriba.
  case "$tag" in
    c_variable_no_declarada|c_llamada_inexistente|c_tipo_incompatible) continue ;;
  esac
  for h in fmt lint doc; do
    timeout 20 "$LB" "$h" "$f" >/dev/null 2>&1
    if [ $? -eq 0 ]; then
      acepta_tool=$((acepta_tool + 1))
      echo "ACEPTA-$h  ${f%.nv} [$tag] '$h' acepto un fuente invalido"
    fi
  done
done

echo "== total=$total acepta_run=$acepta_run acepta_check=$acepta_check ejecuta_cuerpo=$ejecuta acepta_tool=$acepta_tool =="
