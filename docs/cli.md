# Referencia CLI de LÚMEN

## Uso General

```bash
lumen <comando> [opciones] <archivo>
```

## Comandos

### run — Ejecutar programa

```bash
lumen run programa.nv     # Ejecuta fuente .nv
lumen run programa.nvc    # Ejecuta bytecode compilado
lumen run -L ./libs prog.nv  # Con ruta de librerías
```

Compila y ejecuta en un solo paso. Para fuente `.nv`, el pipeline completo (lexer → parser → módulos → sema → IR → codegen → VM) se ejecuta en memoria.

### build — Compilar a bytecode

```bash
lumen build programa.nv   # Genera programa.nvc
```

### check — Verificar sintaxis y semántica

```bash
lumen check programa.nv   # Solo análisis, sin ejecución
```

### disasm — Desensamblar bytecode

```bash
lumen disasm programa.nvc  # Muestra instrucciones en texto legible
```

Útil para aprendizaje: muestra cada instrucción de la VM con sus operandos.

## Opciones Globales

| Opción | Descripción |
|--------|-------------|
| `-L <dir>` / `--lib-dir <dir>` | Directorio de búsqueda para importar módulos |
| `--version` | Versión del compilador |
| `--help` | Mensaje de ayuda |

## Códigos de Salida

| Código | Significado |
|--------|-------------|
| 0 | Éxito |
| 1 | Error del usuario (sintaxis, semántica, runtime) |
| 2 | Error interno (bug del compilador) |

## Ejemplos

```bash
# Ejecutar ejemplo
lumen run examples/hello.nv

# Compilar y luego ejecutar bytecode
lumen build examples/func.nv
lumen run examples/func.nvc

# Verificar sin ejecutar
lumen check examples/loop.nv

# Desensamblar
lumen disasm examples/func.nvc

# Programa con imports
lumen run -L ./librerias programa.nv
```
