# Guía de Contribución

## Requisitos

- Rust 1.70+
- `cargo test --all` debe pasar completo
- 0 warnings de compilación

## Configuración

```bash
git clone https://github.com/omarPVP123131/Lumen.git
cd Lumen
cargo build --all
cargo test --all
```

## Desarrollo

### Estructura del proyecto

El compilador está organizado en 7 crates dentro de `crates/`:

| Crate | Responsabilidad |
|-------|----------------|
| `lumen-lexer` | Análisis léxico |
| `lumen-parser` | Parsing y AST |
| `lumen-sema` | Análisis semántico + ModuleLoader |
| `lumen-ir` | Representación intermedia + optimizaciones |
| `lumen-codegen` | Generación de bytecode |
| `lumen-vm` | Máquina virtual |
| `lumen-cli` | Interfaz de línea de comandos |

### Flujo para añadir una feature

1. **AST**: Añadir variante en `Expr`, `Stmt`, o `Decl` (`lumen-parser/src/ast.rs`)
2. **Tokens**: Si se necesita nueva palabra clave (`lumen-lexer/src/token.rs`)
3. **Parser**: Parsear la nueva sintaxis (`lumen-parser/src/parser.rs`)
4. **Sema**: Type checking y validación (`lumen-sema/src/sema.rs`)
5. **IR**: Compilar a instrucciones intermedias (`lumen-ir/src/builder.rs`)
6. **Codegen**: Emitir bytecode (`lumen-codegen/src/codegen.rs`)
7. **VM**: Ejecutar la nueva instrucción (`lumen-vm/src/vm.rs`)
8. **Value**: Si se necesita un nuevo tipo de valor (`lumen-vm/src/value.rs`)

### Estándares

- Código limpio, sin comentarios superfluos
- Seguir patrones existentes
- Tests obligatorios para código nuevo
- 0 warnings en todos los crates

## Pull Requests

1. Una feature por PR
2. Tests que pasan en CI
3. Documentación actualizada
4. Sin regresiones (todos los tests existentes deben pasar)

## Reportar Issues

Incluir:
- Código mínimo que reproduce el problema
- Comportamiento esperado vs actual
- Versión de LÚMEN (`lumen --version`)
