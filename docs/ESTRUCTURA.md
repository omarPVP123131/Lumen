# Estructura del Proyecto LÚMEN v0.1

```
lumen/
│
├── 📄 Cargo.toml                # Configuración Rust
├── 📘 README.md                 # Documentación principal
├── 📗 BYTECODE.md               # Especificación técnica del bytecode
│
├── 📁 src/                      # Código fuente
│   │
│   ├── 🎯 main.rs              # Punto de entrada + 6 ejemplos
│   │                            # • Bytecode directo
│   │                            # • Variables
│   │                            # • Control de flujo
│   │                            # • Compilador simple
│   │                            # • Condicionales
│   │                            # • Loops
│   │
│   ├── ⚙️ CAPA 1: Máquina Virtual
│   │   ├── vm.rs               # VM completa con error handling
│   │   ├── stack.rs            # Stack LIFO con límites
│   │   └── instructions.rs     # Definición de 15 opcodes
│   │
│   └── 🔧 CAPA 2 y 3: Compilador
│       └── compiler/
│           ├── mod.rs          # Pipeline principal
│           ├── lexer.rs        # Tokenizador
│           ├── ast.rs          # Definición del AST
│           ├── parser.rs       # Parser recursivo
│           └── codegen.rs      # Generador de bytecode
│
└── 📁 ejemplos/                 # Programas de ejemplo
    ├── factorial.lumen         # Cálculo de factorial
    └── fibonacci.lumen         # Serie de Fibonacci
```

## 🎨 Capas Implementadas

### ✅ Capa 1: Máquina Virtual (4 archivos)
- `vm.rs` - 150 líneas - VM completa
- `stack.rs` - 50 líneas - Stack management
- `instructions.rs` - 80 líneas - OpCode definitions

**Funcionalidad:** Ejecuta bytecode binario

---

### ✅ Capa 2: Instrucciones Extendidas (dentro de Capa 1)
- 11 nuevos opcodes
- Variables (STORE/LOAD)
- Comparaciones (EQ/LT/GT)
- Control de flujo (JMP/JMP_IF_FALSE)
- Aritmética (SUB/MUL/DIV)

**Funcionalidad:** Lenguaje Turing-completo

---

### ✅ Capa 3: Compilador (5 archivos)
- `compiler/mod.rs` - 30 líneas - Orquestador
- `compiler/lexer.rs` - 180 líneas - Análisis léxico
- `compiler/ast.rs` - 40 líneas - Estructuras de datos
- `compiler/parser.rs` - 250 líneas - Parser completo
- `compiler/codegen.rs` - 180 líneas - Generación de código

**Funcionalidad:** Compila texto español a bytecode

---

## 📊 Estadísticas del Proyecto

| Componente        | Archivos | Líneas | Estado |
|-------------------|----------|--------|--------|
| Capa 1 (VM)       | 3        | ~280   | ✅     |
| Capa 2 (Ext)      | +0       | +0     | ✅     |
| Capa 3 (Compiler) | 5        | ~680   | ✅     |
| Ejemplos          | 1+2      | ~150   | ✅     |
| **TOTAL**         | **11**   | **~1110** | **75%** |

---

## 🔄 Pipeline de Ejecución

### Modo 1: Bytecode Directo
```
Bytecode manual → VM → Salida
```

### Modo 2: Compilación + Ejecución
```
Código .lumen → Lexer → Parser → CodeGen → Bytecode → VM → Salida
```

---

## 🎯 Próximos Pasos (hacia v1.0)

### 🔲 Capa 4: Multiidioma (pendiente)
**Archivos a crear:**
- `compiler/keywords.rs` - Tabla de keywords
- `compiler/lang_es.rs` - Keywords español
- `compiler/lang_en.rs` - Keywords inglés

**Estimado:** 100 líneas adicionales

---

### 🔲 Capa 5: CLI (pendiente)
**Archivos a crear:**
- `cli/mod.rs` - Parser de argumentos
- `cli/commands.rs` - Comandos run/build/check

**Estimado:** 150 líneas adicionales

---

## 🚀 Cómo Usar

### Compilar el proyecto
```bash
cargo build --release
```

### Ejecutar ejemplos
```bash
cargo run --release
```

### (Futuro) Compilar un programa
```bash
lumen build ejemplos/factorial.lumen
lumen run ejemplos/factorial.lumen
```

---

## 📝 Archivos de Documentación

| Archivo      | Propósito                          |
|--------------|------------------------------------|
| README.md    | Documentación principal del proyecto |
| BYTECODE.md  | Especificación técnica completa    |
| Cargo.toml   | Configuración de Rust              |

---

## 🎓 Para Estudiantes

**Orden recomendado de lectura del código:**

1. `instructions.rs` - Ver qué instrucciones existen
2. `stack.rs` - Entender el stack
3. `vm.rs` - Ver cómo se ejecuta bytecode
4. `main.rs` - Ejemplos de uso
5. `lexer.rs` - Cómo se tokeniza
6. `ast.rs` - Estructuras del lenguaje
7. `parser.rs` - Cómo se construye el AST
8. `codegen.rs` - Cómo se genera bytecode

**Tiempo estimado:** 2-3 horas para entender completamente

---

**LÚMEN v0.1** - Un lenguaje completo en ~1100 líneas de código.
