# LÚMEN — Guía Completa de Herramientas y Ecosistema (DX)

**v3.0 — Herramientas Oficiales de Desarrollo, Depuración y Despliegue**

---

## 📑 Tabla de Contenidos

1. [CLI de LÚMEN (Comandos y Banderas)](#1-cli-de-lúmen-comandos-y-banderas)
2. [Centro de Configuración y Perfiles (`lumen config`)](#2-centro-de-configuración-y-perfiles-lumen-config)
3. [Empaquetador Standalone Zero-Dependencies (`lumen bundle`)](#3-empaquetador-standalone-zero-dependencies-lumen-bundle)
4. [REPL Interactivo Pro (`lumen repl`)](#4-repl-interactivo-pro-lumen-repl)
5. [Time-Travel Debugger (`lumen debug`)](#5-time-travel-debugger-lumen-debug)
6. [Asistente Inteligente IA (`lumen ai`)](#6-asistente-inteligente-ia-lumen-ai)
7. [Diagnóstico y Telemetría (`lumen doctor` & `lumen monitor`)](#7-diagnóstico-y-telemetría-lumen-doctor--lumen-monitor)
8. [Gestor de Paquetes (`lumen-pkg`)](#8-gestor-de-paquetes-lumen-pkgs)
9. [Servidor LSP Pro para Editores (`lumen-lsp`)](#9-servidor-lsp-pro-para-editores-lumen-lsp)
10. [Playground Web con WebGPU (`lumen serve`)](#10-playground-web-con-webgpu-lumen-serve)

---

## 1. CLI de LÚMEN (Comandos y Banderas)

```bash
lumen <comando> [opciones] <archivo.nv>
```

### Comandos Principales
* `lumen run <archivo>`: Ejecuta código fuente o bytecode en memoria con JIT hot tiering.
* `lumen build --native <archivo>`: Compilación AOT nativa C99/GCC `-O3`.
* `lumen bundle <archivo> -o <app>`: Genera un binario nativo **Zero-Dependencies**.
* `lumen check .`: Verificación semántica recursiva de todo el proyecto.
* `lumen new <nombre> --template <ia|web|game|default>`: Scaffolding de proyectos.
* `lumen fmt <archivo>`: Formateador automático de código fuente.
* `lumen test <archivo>`: Ejecución de suites de pruebas unitarias.
* `lumen bench <archivo>`: Micro-benchmarking de rendimiento y throughput.
* `lumen lint <archivo>`: Análisis estático (errores + avisos de estilo).
* `lumen fuzz <archivo>`: Mutación de literales y detección de fallos en ejecución.

---

## 1.1. Escribir pruebas (`lumen test`)

Una prueba es una **función sin parámetros cuyo nombre empieza por `test_`**. El
runner las descubre y las ejecuta una a una, cada una en su propio entorno.

```lumen
importar "testing.nv";

funcion void test_suma() {
    testing_afirmar_igual(2 + 2, 4);
}

funcion void test_lista_crece() {
    lista<entero> l = [1];
    agregar(l, 2);
    testing_afirmar_igual(largo(l), 2);
}
```

```console
$ lumen test tests/mis_pruebas.nv
  ✓ test_lista_crece ... OK
  ✓ test_suma ... OK
  Resultado: 2 pasaron, 0 fallaron
```

Las aserciones disponibles en `testing.nv` (con alias en inglés):

| Función | Comprueba |
|---|---|
| `testing_afirmar_verdadero(v)` | que `v` sea verdadero |
| `testing_afirmar_falso(v)` | que `v` sea falso |
| `testing_afirmar_igual(a, b)` | que `a == b` |
| `testing_afirmar_distinto(a, b)` | que `a != b` |

Cuando una prueba falla, el runner la marca con ✗, muestra el motivo y **el
proceso termina con código de salida 1**, de modo que se puede encadenar en un
CI. Las aserciones sueltas en el nivel superior del archivo también cuentan.

## 2. Centro de Configuración y Perfiles (`lumen config`)

```bash
# Conmutar perfiles predefinidos de optimización y memoria:
lumen config profile release   # -O3, Neuro-Optimizador activo, LTO
lumen config profile hpc       # AVX-512 SIMD, Zero-GC Borrow Checker, FMA Fused
lumen config profile mcu       # Bare-metal sin SO (<32 KB Freestanding)
lumen config profile cloud     # Nexus Web, Self-Healing Runtime, Postgres/Redis Wire
lumen config profile dev       # -O0, Time-Travel activo, JIT instantáneo

# Listar configuración activa:
lumen config list
```

---

## 3. Empaquetador Standalone Zero-Dependencies (`lumen bundle`)

Genera un único ejecutable binario autónomo que incluye el código compilado, las dependencias y la biblioteca estándar:

```bash
lumen bundle src/main.nv -o mi_servicio
./mi_servicio
```

La ruta de salida acepta las dos formas, `-o mi_servicio` y posicional
(`lumen bundle src/main.nv mi_servicio`), y puede incluir directorios que aún no
existan: se crean. Si por cualquier motivo el binario no llega a generarse, el
comando termina con código de salida 1 en vez de informar de un éxito falso
(BUG-073).

---

## 3.1. Análisis estático (`lumen lint`)

Ejecuta el análisis léxico, sintáctico y semántico completo —los mismos
diagnósticos que `lumen check`— y añade reglas de estilo:

| Regla | Descripción |
|---|---|
| Línea larga | Más de 120 caracteres |
| Espacios finales | Espacios o tabuladores al final de la línea |
| Tabulador literal | Sangría con tabuladores (usa `lumen fmt`) |
| Marca pendiente | Comentarios `TODO` / `FIXME` sin resolver |

```bash
lumen lint src/main.nv
# ✓ Análisis estático (lumen lint): 0 advertencias en 'src/main.nv'
```

Los **errores** hacen salir con código 1; las **advertencias** de estilo se
informan pero no rompen la compilación, de modo que se puede usar en CI.

---

## 3.2. Fuzzing (`lumen fuzz`)

Localiza los literales enteros del programa, genera mutaciones a valores límite
(`0`, `-1`, `1`, `i64::MIN`, `i64::MAX`, `2147483647`), y **compila y ejecuta
cada variante en la VM**, informando de los fallos reproducibles:

```bash
lumen fuzz src/main.nv
#   • Mutaciones generadas   : 114
#   • Ejecutadas en la VM    : 114
#   • Fallos detectados      : 0
```

Cuando encuentra un fallo lo muestra con la mutación que lo provoca y termina
con código 1:

```
✗ literal #2 → 0 → Error: División por cero
```

Las mutaciones que no compilan (porque cambian el significado del programa) se
descartan y se contabilizan aparte.

---

## 3.3. Bindings FFI (`lumen bindgen`)

Genera un módulo LÚMEN a partir de una cabecera C:

```bash
lumen bindgen mini.h        # → mini_bindings.nv
```

Para `int suma(int a, int b);` produce:

```lumen
texto _lib_handle = __ffi_cargar("mini.so");

funcion cualquiera suma(cualquiera arg1 = 0, cualquiera arg2 = 0) {
    retornar __ffi_llamar(_lib_handle, "suma", "entero,entero", [arg1, arg2], "entero");
}
```

El tipo de retorno se deduce de la cabecera (`double` → `decimal`, `void` →
`vacio`), y la cadena de tipos de los parámetros es **obligatoria**: la firma es
`__ffi_llamar(lib, nombre, "tipos", [args], "retorno")`.

Dos detalles al usarlo:

* **La ruta de la biblioteca.** Se genera como `"<nombre>.so"`; ajústala a la
  ruta real (absoluta o resoluble por el enlazador) antes de ejecutar.
* **Los módulos importados se prefijan con su nombre.** Tras
  `importar "mini_bindings.nv";` la función se llama `mini_bindings_suma(...)`,
  igual que `testing.nv` expone `testing_afirmar_igual`.

```lumen
importar "mini_bindings.nv";
imprimir("suma via FFI: ", mini_bindings_suma(20, 22));   // → 42
```

Memoria manual: `__ffi_asignar(tam)` / `__ffi_escribir` / `__ffi_leer` /
`__ffi_liberar`. Liberar dos veces el mismo puntero devuelve un error normal —ya
no aborta el proceso (BUG-079)—, pero sigue siendo un fallo del programa.

---

## 3.4. Compilación nativa y elección de backend (`lumen build`)

```bash
lumen build --native app.nv              # backend C (por defecto, recomendado)
lumen build --native --aot rust app.nv   # backend Cranelift
lumen build --native --aot llvm app.nv   # backend LLVM
```

### Qué backend usar

| Backend | Bandera | Cobertura del lenguaje | Cuándo usarlo |
|---|---|---|---|
| **C** | `--native` (defecto) | Completa | Uso general. Es el único con runtime completo. |
| **Cranelift** | `--aot rust` | **Parcial** | Compilación rápida de código numérico. |
| **LLVM** | `--aot llvm` | **Muy parcial** (12/42 opcodes) | Experimental: sólo código escalar. |

> ⚠️ **Limitación del backend Cranelift.** No implementa los builtins de
> colecciones y texto (`largo`, `agregar`, `a_texto`, `leer`, `__map_*`…) ni las
> construcciones de datos compuestos (structs, listas, tuplas, enums,
> `opcion`/`resultado`, llamadas indirectas).
> Hasta la v3.0 esas llamadas se compilaban como la constante `0` **sin ningún
> aviso**, así que el binario devolvía resultados incorrectos en silencio
> (BUG-084). Ahora la compilación se detiene y enumera lo que falta:
>
> ```
> ⚠  1 builtin(s) sin soporte en el backend Cranelift:
>      · largo
>
>   Estas funciones devolverían 0 en el binario, sin error.
>
>   Opciones:
>     · Compila con el backend C:     lumen build --native app.nv
>     · Ejecuta con la VM:            lumen run app.nv
>     · O asume el riesgo:            ... --permitir-no-soportados
> ```
>
> `--permitir-no-soportados` fuerza la compilación conservando el `0`; úsalo
> sólo si sabes que esas rutas no se ejecutan.

---

## 4. REPL Interactivo Pro (`lumen repl`)

Entorno interactivo con evaluación en caliente, 64-bit NaN-Boxing y comandos especiales:
* `:doc <simbolo>`: Documentación interactiva de funciones y módulos.
* `:bench <código>`: Medición de latencia de ejecución en microsegundos (`µs`).
* `:mem`: Inspección del estado del modelo de memoria.
* `:clear`: Reinicia el ámbito de variables acumuladas.
* `:help`: Muestra la guía de comandos interactivos.

---

## 5. Time-Travel Debugger (`lumen debug`)

Depurador con capacidad de retroceder en el tiempo:
* `step` / `s`: Avanzar una instrucción de bytecode.
* `back` / `step-back`: **Retroceder una instrucción restaurando el estado exacto de la pila y registros**.
* `history` / `timeline`: Muestra la línea de tiempo de instantáneas (*snapshots*) de memoria.
* `continue` / `c`: Continuar ejecución hasta el siguiente breakpoint.

---

## 6. Asistente Inteligente IA (`lumen ai`)

* `lumen ai explain <archivo>`: Análisis estático de AST, funciones, estructuras y complejidad.
* `lumen ai fix <archivo>`: Detección y sugerencias de corrección de tipos y memoria.
* `lumen ai test <archivo>`: Generación automática de pruebas unitarias con aserciones.
* `lumen ai chat "<pregunta>"`: Asistente interactivo de arquitectura y sintaxis.

---

## 7. Diagnóstico y Telemetría (`lumen doctor` & `lumen monitor`)

* `lumen doctor`: Diagnóstico integral de compiladores C, extensiones vectoriales SIMD, modelos de memoria y módulos de la stdlib.
* `lumen monitor`: Panel TUI en tiempo real con métricas de memoria, JIT tiering y microservicios.

---

## 8. Gestor de Paquetes (`lumen-pkg`)

* `lumen add <paquete>` / `lumen install <target>`: Instala paquetes locales, archivos `.lmp`, carpetas, repositorios GitHub, crates de Rust (`cargo:<crate>`) y cabeceras C (`c:<header.h>`).
* `lumen publish [dir]`: Firma criptográfica SHA-256 y empaquetado para distribución.
* `lumen pack` / `lumen unpack`: Empaquetado y descompresión de archivos `.lmp`.

---

## 9. Servidor LSP Pro para Editores (`lumen-lsp`)

Soporte oficial para Visual Studio Code, Neovim y JetBrains:
* **Semantic Highlighting**: 11 categorías de tokens semánticos en tiempo real.
* **Inlay Hints**: Tipos deducidos sobre variables `sea`/`let`.
* **Signature Help**: Resaltado de parámetros activos en llamadas.
* **Code Actions**: QuickFixes automáticos y formateo integral.

---

## 10. Playground Web con WebGPU (`lumen serve`)

Inicia un servidor local en `http://localhost:8080` con:
* Depurador visual Time-Travel interactivo con barra de snapshots.
* Simulador de partículas WebGPU en tiempo real a 60 FPS.
* Presets interactivos listos para ejecutar de Nexus Web, PostgreSQL Wire 3.0, Redis RESP3, UI Reactiva, IA INT8 y Self-Hosting.

---

*LÚMEN v3.0 — Documentación de Herramientas Sincronizada.*
