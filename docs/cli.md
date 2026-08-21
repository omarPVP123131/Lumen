# Referencia CLI de LÚMEN — v2.4.6

## Uso General

```bash
lumen <comando> [opciones/banderas] [archivo.nv]
```

---

## 🚀 Comandos Principales

### `run` — Ejecutar programa
```bash
lumen run programa.nv
lumen run -L stdlib programa.nv
lumen run --profile cloud servidor.nv
```
Compila y ejecuta en memoria con soporte para Hot JIT Tiering y Self-Healing Runtime.

### `build` — Compilación AOT y Multi-Target
```bash
lumen build programa.nv                  # Genera bytecode .nvc
lumen build --native programa.nv         # Genera binario nativo C99 (-O3)
lumen build --standalone programa.nv     # Genera binario independiente optimizado (-O3 -s)
lumen build --embedded programa.nv       # Genera binario Bare-Metal MCU (<32 KB)
lumen build --aot llvm programa.nv       # Emisión directa de LLVM IR (.ll)
lumen build --aot rust programa.nv       # Backend Cranelift AOT
lumen build --target aarch64-apple-darwin app.nv # Compilación cruzada para Apple Silicon
```

### `bundle` — Empaquetado Binario Standalone Zero-Dependencies
```bash
lumen bundle src/main.nv -o mi_app
```
Genera un único binario ejecutable independiente listo para producción.

### `check` — Comprobación Global de Proyecto
```bash
lumen check .                 # Analiza todos los archivos .nv del proyecto actual recursivamente
lumen check src/              # Comprueba todos los módulos dentro de src/
```

### `repl` — Entorno Interactivo Pro
```bash
lumen repl                    # Inicia el REPL interactivo con comandos :doc, :bench, :mem, :clear
```

### `config` — Gestor de Configuración y Perfiles
```bash
lumen config list                      # Muestra la configuración activa
lumen config profile release           # Conmuta al perfil de producción
lumen config profile hpc               # Conmuta al perfil de supercómputo SIMD y Zero-GC
```

### `ai` — Asistente IA Integrado en Terminal
```bash
lumen ai explain mi_programa.nv        # Explica AST, estructuras y complejidad
lumen ai fix mi_programa.nv            # Diagnóstica errores de tipos y sugiere correcciones
lumen ai test mi_programa.nv           # Genera suite de pruebas unitarias automáticas
lumen ai chat "Cómo usar PostgreSQL?" # Asistente conversacional de arquitectura
```

### `monitor` / `dashboard` — Telemetría en Tiempo Real
```bash
lumen monitor                          # Panel ASCII TUI de memoria, JIT y microservicios
```

### `doctor` / `info` — Diagnóstico del Entorno
```bash
lumen doctor                           # Diagnóstica hardware, SIMD, compiladores y stdlib
```

### `new` — Creación de Proyectos Estructurados
```bash
lumen new mi_proyecto --template web   # Plantillas: web | ia | game | default
```

### `install` / `add` — Gestor de Dependencias
```bash
lumen install paquete_oficial          # Instala desde el registro central
lumen install ./mi_libreria_local      # Instala desde carpeta local
lumen install ./paquete.lmp            # Instala desde archivo comprimido .lmp
lumen install cargo:serde_json         # Enlace automático a crates de Rust
lumen install c:sqlite3.h              # Generador automático de bindings C
```

### `publish` / `pack` / `unpack` — Distribución de Paquetes
```bash
lumen publish [directorio]             # Firma SHA-256 y publicación oficial
lumen pack [directorio]                # Empaqueta proyecto en archivo .lmp
lumen unpack paquete.lmp [destino]     # Desempaqueta archivo .lmp
```

### `serve` / `playground` — Servidor Web & WebGPU
```bash
lumen serve --port 8080                # Inicia el Playground Web interactivo
```

### `lsp` — Servidor Language Server Protocol
```bash
lumen lsp                              # Inicia servidor LSP Pro para VS Code / Neovim
```

---

## ⚙️ Banderas y Opciones Globales

| Bandera | Descripción |
| :--- | :--- |
| `--memory-model <modelo>` | Selecciona modelo: `nanbox`, `borrow-checker`, `arena`, `auto` |
| `--zero-gc` | Activa modo estricto Borrow Checker Zero-GC (`prestado`/`dueno`) |
| `--self-healing` | Activa runtime autorregenerativo con hot-patching |
| `--neuro-opt` | Activa superoptimizador neuro-simbólico en IR |
| `--profile <perfil>` | Perfil predefinido: `dev`, `release`, `hpc`, `mcu`, `cloud` |
| `--target <triple>` | Arquitectura destino: `x86_64-linux-gnu`, `aarch64-apple-darwin`, etc. |
| `-O, --opt-level <0-3>` | Nivel de optimización |
| `-L, --lib-dir <dir>` | Ruta personalizada de módulos stdlib |
| `-v, --version` | Muestra versión de LÚMEN |
| `-h, --help` | Muestra la ayuda de comandos |

---

*LÚMEN v2.4.6 — Documentación CLI Oficial Sincronizada.*


---

## Nuevos Comandos y Mejoras v2.4.6

### 1. `lumen bundle <archivo.nv> [salida.exe|salida]`
Empaqueta un script de LÚMEN en un único binario nativo autocontenido sin dependencias externas:
```bash
# Windows
lumen bundle examples/demo_matriz_simd_ia.nv mi_app.exe

# Linux
lumen bundle examples/demo_matriz_simd_ia.nv mi_app
```

### 2. `lumen debug <archivo.nv>` (Depurador Visual TUI)
Inicia la interfaz gráfica de terminal con ventana de código en vivo `▶▶▶`, puntos de interrupción `🔴 [B]`, inspector de variables y Time-Travel:
* `s` / `step`: Avanza 1 instrucción.
* `back` / `retroceder`: Time-Travel (retrocede 1 estado de la memoria).
* `b <línea>`: Alterna un punto de interrupción.
* `vars`: Inspecciona todas las variables locales y globales en memoria.
* `stack`: Muestra la pila de llamadas (*Call Stack*).

### 3. `lumen install <paquete@semver>` (Gestor de Paquetes con SemVer)
Instala dependencias y genera el archivo de bloqueo `lumen.lock` con hashes SHA-256 reproducibles:
```bash
lumen install ai_tensor@^2.0.0
lumen install http_router
```

### 4. `lumen doctor`
Diagnostica automáticamente tu usuario, tu sistema operativo, los núcleos de tu CPU disponibles para el scheduler M:N y tus toolchains de compiladores C/Rust.
