#!/usr/bin/env bash
# ============================================================================
# LÚMEN — Verificación del fixpoint self-hosting (v3.5.7)
#
# Corre desde la RAÍZ del repo Lumen:
#     bash scripts/verificar_fixpoint.sh
#
# Qué hace:
#   0. Construye el binario release si no existe.
#   1. Genera compiler_v4.nvc (compilador self-hosted compilado por RUST).
#   2. STAGE 1: compiler_v4.nvc compila compiler_v4.nv -> v4_self.nvc
#   3. Verifica que v4_self.nvc NO tenga funciones duplicadas (el bug del
#      import "lexer.nv" redundante duplicaba lexer_tokenizar/tokenize).
#   4. PROBE: v4_self.nvc compila fuzz/selfhost_probe.nv -> debe imprimir 42.
#   5. STAGE 2: v4_self.nvc recompila compiler_v4.nv -> v4_self2.nvc
#   6. Compara v4_self.nvc == v4_self2.nvc (byte-identical = FIXPOINT).
#
# Resultado: escribe docs/informes/fixpoint_status.md e imprime un resumen.
# Duración aprox: 1-2 h (dos autocompilaciones de ~150KB cada una).
# ============================================================================
set -u
cd "$(dirname "$0")/.."   # raíz del repo
ROOT="$(pwd)"
LUMEN="$ROOT/target/release/lumen"
P=docs/informes/fixpoint_status.md
mkdir -p docs/informes
: > "$P"
log(){ echo "$1" | tee -a "$P"; }

log "# Fixpoint self-hosting — $(date)"
log "host: $(uname -a)"

# 0) binario release
if [ ! -x "$LUMEN" ]; then
  log "## Binario release ausente — construyendo (cargo build --release)..."
  ( cd "$ROOT" && cargo build --release --bin lumen ) >> "$P" 2>&1 || { log "FALLO: cargo build --release"; exit 1; }
fi
log "binario: $($LUMEN --version 2>/dev/null || echo presente)"

# 1) compiler_v4.nvc via RUST
log "## Paso 1: compiler_v4.nvc (compilado por Rust)..."
"$LUMEN" build stdlib/compiler/compiler_v4.nv >> "$P" 2>&1 || { log "FALLO paso 1"; exit 1; }

# 2) STAGE 1
log "## Paso 2 (STAGE 1): autocompilar compiler_v4.nv (~5 s con lexer nativo)..."
printf 'stdlib/compiler/compiler_v4.nv\n/tmp/v4_self.nvc\n' > stdlib/compiler/target.txt
rm -f /tmp/v4_self.nvc /tmp/v4_self2.nvc
time "$LUMEN" run stdlib/compiler/compiler_v4.nvc 2>&1 | tee -a "$P" | sed 's/^/  | /'
[ -f /tmp/v4_self.nvc ] || { log "FALLO: stage1 no genero v4_self.nvc"; exit 1; }
log "v4_self.nvc: $(wc -c < /tmp/v4_self.nvc) bytes"

# 3) duplicados?
"$LUMEN" disasm /tmp/v4_self.nvc > /tmp/dis_self.txt 2>&1
dups=$(grep -oE "name=[a-zA-Z_0-9]+" /tmp/dis_self.txt | sort | uniq -d | tr '\n' ' ')
log "funciones duplicadas en v4_self: [${dups:-NINGUNA ✔}]"
log "total funciones: $(grep -c 'func\[' /tmp/dis_self.txt)"

# 4) PROBE via self
log "## Paso 3 (PROBE): v4_self.nvc compila selfhost_probe.nv (esperado: 42)..."
mkdir -p /tmp/fpc/stdlib/compiler /tmp/fpc/fuzz
cp fuzz/selfhost_probe.nv /tmp/fpc/fuzz/
printf 'fuzz/selfhost_probe.nv\nfuzz/p.nvc\n' > /tmp/fpc/stdlib/compiler/target.txt
( cd /tmp/fpc && "$LUMEN" run /tmp/v4_self.nvc ) > /tmp/probe_comp.log 2>&1
probe=$( (cd /tmp/fpc && "$LUMEN" run fuzz/p.nvc) 2>&1 )
log "probe via SELF-COMPILED = [${probe}] (esperado 42)"

# 5) STAGE 2
log "## Paso 4 (STAGE 2): v4_self.nvc recompila compiler_v4.nv (~5 s con lexer nativo)..."
printf 'stdlib/compiler/compiler_v4.nv\n/tmp/v4_self2.nvc\n' > stdlib/compiler/target.txt
rm -f /tmp/v4_self2.nvc
time "$LUMEN" run /tmp/v4_self.nvc 2>&1 | tee -a "$P" | sed 's/^/  | /'

# 6) comparar
if [ -f /tmp/v4_self2.nvc ]; then
  if cmp -s /tmp/v4_self.nvc /tmp/v4_self2.nvc; then
    log "## ✅ FIXPOINT: v4_self.nvc y v4_self2.nvc BYTE-IDENTICAL ($(wc -c < /tmp/v4_self.nvc) B)"
    log "sha256: $(sha256sum /tmp/v4_self.nvc | cut -d' ' -f1)"
  else
    log "## ❌ FIXPOINT ROTO: self=$(wc -c < /tmp/v4_self.nvc) B vs self2=$(wc -c < /tmp/v4_self2.nvc) B"
    "$LUMEN" disasm /tmp/v4_self2.nvc > /tmp/dis_self2.txt 2>&1
    dups2=$(grep -oE "name=[a-zA-Z_0-9]+" /tmp/dis_self2.txt | sort | uniq -d | tr '\n' ' ')
    log "duplicadas en self2: [${dups2:-ninguna}]"
  fi
else
  log "## ❌ FALLO: stage2 no genero v4_self2.nvc (revisar fin del log)"
fi
log "fin: $(date)"
echo "────────────────────────────────────────────"
cat "$P"
