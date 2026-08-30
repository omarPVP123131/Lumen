# ⚡ Arquitectura del JIT de LÚMEN (estado real, v3.5.7 + rondas v3.5.31→v3.5.37)

> Documento técnico del motor JIT de `crates/lumen-vm/src/jit.rs` (+ helpers en `vm.rs`).
> Compilado por defecto en builds con la feature `aot`; se desactiva con `LUMEN_JIT=0`
> (intérprete puro, usado como referencia de paridad y en el fixpoint).

## Principio rector

El JIT **no reimplementa semántica**: cada operación no nativa delega en los
MISMOS handlers del intérprete vía shims `extern "C"` (`lj_*`). El código nativo
solo elimina el costo de dispatch/decode y ejecuta saltos de forma nativa.
La paridad se verifica byte-a-byte contra el intérprete en cada ronda
(956 tests ×2, ci_gate ×2, edge tests y fixpoint self-hosting).

## Los tres niveles

| Nivel | Qué hace | Cuándo se elige |
|---|---|---|
| **Tier-R** (v3.5.34) | Funciones auto-recursivas puras de enteros (fib) → función CLIF **recursiva en registros**: cero frames, cero shims, cero tráfico de pila por nivel | 1 parámetro Int; cuerpo de {FusedCmpKJmp, Load del param, PushInt, Add/Sub/Mul, Jmp, Call auto argc=1, Nop, Ret}; pila vacía en cada salto; Ret final |
| **Tier-2** (v3.5.31+) | Bucles con aritmética/arrays/textos → **código nativo directo** sobre la arena de slots (`flat`), con bail-out al intérprete si algo no encaja | Pre-scan de elegibilidad + verifier estático de pila + gate de tipos |
| **Tier-1** | Cualquier función caliente → bucle nativo que **delega cada instrucción** a los handlers del intérprete por shims (elimina solo el dispatch) | Fallback universal |

## Tier-2 en detalle

**Pre-scan** (elegibilidad + conjuntos): `store_names`, `read_names` (SOLO
operandos Int de Fused arith/cmp — nunca Load genérico), `d_names`,
`written`, `dyn_written` (nombres escritos por rutas dinámicas: ArrayNew→StoreLocal,
StoreLocal suelto, resultado de llamada), `dyn_arith` (cuerpo que mueve textos),
`alloc_possible` (el cuerpo puede asignar slots en runtime), `backward_targets`,
`has_scope_push`, `ret_seen`.

**Verifier estático de pila**: alturas por posición + convergencia de saltos
(efectos: PushStr +1, ArrayPushVar −2, Call −argc+1, JmpIf −1 en destino y efecto).
Gate final: `read_names ∩ dyn_written → rechazo`.

**Análisis de tipos (VTag, v3.5.37)**: fixpoint monótono de pocas pasadas sobre
el cuerpo ya elegible — etiquetas por posición de pila y por nombre
(`Int`, `Str`, `Arr(ETag)` para arrays con etiqueta de elementos, `Any`).
Decide en el walker:
- `Add/Sub/Mul` en modo texto: Int+Int → nativo; Add con Str → shim `lj_concat`
  (concat rápido, espejo exacto del arm Add del intérprete); mixto → shim genérico.
- `Load/Store` nativos por etiqueta en cuerpos sin `ScopePush` (slot resuelto
  en prólogo, sin escrituras dinámicas).
- `ArrayGet` + aritmética nativa cuando el análisis prueba elemento `Int`.

**Contrato de retorno**: `0` OK · `1` error (propagado) · `2` bail-out
(invalidar + re-ejecutar el MISMO frame en el intérprete).

**Correcciones de soundness aprendidas en las rondas** (reglas permanentes):
- `flat` puede REALOCARSE (calls a usuario, StoreLocal nuevos) → la base se
  re-obtiene tras cada op asignadora y al re-entrar bucles con `alloc_possible`.
- El prólogo es 1 call por nombre: `lj_probe_int` (existe + es Int) — vale
  porque `lookup_names ⊆ guard` siempre.
- Arms de aritmética Div/Mod nativa exigen divisor > 0 (bail si no) — paridad
  con `rem_euclid` y el Div del intérprete.
- Nunca reutilizar shims de variante KC para la variante KK (constante vs nombre).

## Tier-1 en detalle

Bucle nativo por bloques (leaders) que llama un shim por instrucción
(`lj_simple`, `lj_with_idx`, `lj_with_num`, `lj_call`/`lj_call_fast`,
`lj_fused_*`, `lj_truth`, `lj_ret`…). La elección `lj_call` vs `lj_call_fast`
es ESTÁTICA en compilación vía `builtin_name_set()` (352 nombres):
`lj_call_fast` solo se emite para nombres no-builtin (salta el pre-filtro O(1)).

## Super-opcodes (Fused)

El codegen fusiona secuencias calientes en instrucciones únicas que el JIT
compila a código nativo compacto:
- **Fused 3-instr**: `FusedBin`/`FusedBinK` (a op b → d con k constante).
- **Fused cmp+jmp**: `FusedCmpKJmp`, `FusedCmpJmp`.
- **Fused 6-instr** (v3.5.31): `FusedBinCmpJmp`, `FusedBinKCmpJmp`,
  `FusedBinKKCmpJmp` (tags 9/10/11 en el bytecode).

## Tier-R en detalle (recursión en registros)

`try_compile_recursive`: wrapper JitFn (probe del parámetro → carga del slot →
llamada recursiva con profundidad 0 → push + `lj_ret`) + función recursiva
CLIF (vm, n, depth) con bloques por ip y el parámetro en REGISTRO.
El límite de profundidad replica `MAX_CALL_STACK_DEPTH` con un contador:
al superarlo devuelve código 2 → el intérprete produce el MISMO error
"Desbordamiento de pila (>10000 llamadas)" (paridad verificada con prof(15000)).

## Números (min-of-15, release, JIT default-ON)

| Tarea | JIT ON | Intérprete | Ganancia |
|---|---|---|---|
| sum | 28.1 ms | 1138.4 ms | 41× |
| fib | 4.4 ms | 100.4 ms | 23× |
| primes | 11.8 ms | 34.5 ms | 2.9× |
| strings | 162.3 ms | 177.2 ms | 1.09× |
| arrays | 60.5 ms | 91.5 ms | 1.51× |
| **TOTAL** | **267.1 ms** | **1541.9 ms** | **5.8×** |

Evolución del total: 590 → 383 → 343.5 → 275.7 → 272.6 → **267.1 ms**.

## Bugs reales encontrados por el trabajo de las rondas (y arreglados)

1. **Constant folder IR** (`i64::MIN / -1` panic + `%` truncante; v3.5.33).
2. **Folder optimize.rs** — delta NETO de pila: `f(3) + 1` borraba el argumento
   y el Add → "Stack underflow" (v3.5.34).
3. **Flat obsoleto en Tier-2** — realocación del Vec de slots por llamadas a
   usuario → lecturas nativas a memoria liberada (v3.5.35).
4. **Indexado de `slots[nidx]` sin guard** en Load/Store nativos por etiqueta
   → panic en el fixpoint (v3.5.37).
