# Referencia CLI de LÚMEN — v1.6.0

## Uso General

```bash
lumen <comando> [opciones] <archivo>
```

---

## Comandos

### `run` — Ejecutar programa

```bash
lumen run programa.nv       # Ejecuta fuente .nv
lumen run programa.nvc      # Ejecuta bytecode compilado
lumen run -L ./libs prog.nv # Con directorio de librerías
```

Compila y ejecuta en un solo paso. El pipeline completo se ejecuta en memoria.

---

### `build` — Compilar a bytecode

```bash
lumen build programa.nv     # Genera programa.nvc
```

---

### `check` — Verificar sintaxis y semántica

```bash
lumen check programa.nv     # Análisis completo sin ejecutar
```

---

### `disasm` — Desensamblar bytecode

```bash
lumen disasm programa.nvc   # Muestra instrucciones en texto legible
```

Útil para aprendizaje: cada instrucción de la VM con sus operandos.

---

### `fmt` — Formatear código

```bash
lumen fmt archivo.nv        # Formatea en su lugar
lumen fmt --check archivo.nv # Solo verifica sin modificar
```

Soporta configuración vía `.lumen-fmt.toml`:

```toml
indent_spaces = 4
max_line_length = 100
trailing_newline = true
```

---

### `repl` — REPL interactivo

```bash
lumen repl
```

Características: historial persistente, edición multilínea, resaltado de sintaxis,
autocompletado de símbolos.

---

### `new` — Crear proyecto

```bash
lumen new mi_proyecto
```

Genera scaffolding con `lumen.toml`, `src/main.nv`, y estructura de proyecto estándar.

---

### `test` — Ejecutar tests

```bash
lumen test tests.nv
```

Ejecuta todas las funciones que empiecen con `test_`. Usa `afirmar(expr)` para assertions.

---

### `lint` — Análisis estático

```bash
lumen lint programa.nv
```

Detecta: código muerto, variables no usadas, complejidad ciclomática alta, imports no usados.

---

### `doc` — Generar documentación

```bash
lumen doc programa.nv       # Genera HTML en ./docs/
lumen doc programa.nv -o mi_docs/
```

Extrae comentarios `///` y genera documentación HTML estática.

---

### `install` — Gestor de paquetes

```bash
lumen install coleccion        # Instala desde registry
lumen install --local ./ruta   # Instala desde directorio local
lumen install --help           # Ayuda del comando
```

Gestiona dependencias externas. Requiere conexión a internet para descargar del registry.

---

### `debug` — Depurador interactivo

```bash
lumen debug programa.nv
```

Comandos en el depurador:
- `break <linea>` — Añadir breakpoint
- `step` / `s` — Ejecutar siguiente instrucción
- `continue` / `c` — Continuar hasta siguiente breakpoint
- `inspect <var>` — Ver valor de variable
- `quit` — Salir

---

### `serve` — Servidor de desarrollo

```bash
lumen serve programa.nv
```

Inicia servidor con hot reload. Recarga automáticamente al detectar cambios en `.nv`.

---

### `lsp` — Servidor LSP

```bash
lumen lsp
```

Servidor LSP para VS Code y editores compatibles. Provee:
- Diagnósticos en tiempo real (`publishDiagnostics`)
- Autocompletado (`textDocument/completion`)
- Ir a definición (`textDocument/definition`)
- Hover con tipos (`textDocument/hover`)

---

## Opciones Globales

| Opción | Descripción |
|--------|-------------|
| `-L <dir>` / `--lib-dir <dir>` | Directorio de búsqueda para importar módulos |
| `--version` | Versión del compilador |
| `--help` | Mensaje de ayuda |

---

## Códigos de Salida

| Código | Significado |
|--------|-------------|
| 0 | Éxito |
| 1 | Error del usuario (sintaxis, semántica, runtime) |
| 2 | Error interno (bug del compilador) |

---

## Ejemplos

```bash
# Ejecutar un programa
lumen run examples/hello.nv

# Compilar y luego ejecutar bytecode
lumen build examples/func.nv
lumen run examples/func.nvc

# Verificar sin ejecutar
lumen check examples/loop.nv

# Desensamblar
lumen disasm examples/func.nvc

# Formatear todos los .nv
lumen fmt src/*.nv

# Ejecutar tests
lumen test tests/unit.nv

# Programa con imports desde directorio
lumen run -L ./stdlib programa.nv

# Generar docs
lumen doc src/main.nv -o ./docs

# Depurar
lumen debug programa.nv
```
