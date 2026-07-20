# LÚMEN Virtual Machine Specification v1.0

## Architecture
Stack-based virtual machine with:
- **Value stack**: runtime operand stack for all operations
- **Call stack**: frames with locals, return addresses, and function metadata
- **Shared constant pools**: deduplicated strings, ints, floats, and names

## Value Representation
```rust
enum Value {
    Int(i64),                                   // Entero de 64 bits
    Float(f64),                                 // Decimal IEEE-754
    Str(String),                                // Texto UTF-8
    Bool(bool),                                 // Booleano
    Array(Vec<Value>),                          // Lista dinámica
    Func(String),                               // Referencia a función
    Struct { name: String, fields: Vec<(String, Value)> },  // Estructura
    Exito(Box<Value>),                          // Resultado exitoso
    Error(Box<Value>),                          // Resultado de error
    Void,                                       // Ausencia de valor
}
```

## Opcodes (41 total)

### Core (0-27)
| Code | Name | Operands | Stack Effect | Description |
|------|------|----------|-------------|-------------|
| 0 | Halt | — | — | Stop execution |
| 1 | PushInt | idx | → Int | Push int from pool |
| 2 | PushNum | idx | → Float | Push float from pool |
| 3 | PushStr | idx | → Str | Push string from pool |
| 4 | PushBool | u8 | → Bool | Push true(1)/false(0) |
| 5 | Load | idx | → Value | Load local by name |
| 6 | Store | idx | Value → | Store local by name |
| 7 | Add | — | A B → A+B | Addition/Concat |
| 8 | Sub | — | A B → A-B | Subtraction |
| 9 | Mul | — | A B → A*B | Multiplication |
| 10 | Div | — | A B → A/B | Division |
| 11 | Eq | — | A B → A==B | Equality |
| 12 | Neq | — | A B → A≠B | Not equal |
| 13 | Lt | — | A B → A<B | Less than |
| 14 | Le | — | A B → A≤B | Less or equal |
| 15 | Gt | — | A B → A>B | Greater than |
| 16 | Ge | — | A B → A≥B | Greater or equal |
| 17 | And | — | A B → A∧B | Logical AND |
| 18 | Or | — | A B → A∨B | Logical OR |
| 19 | Neg | — | A → -A | Arithmetic negate |
| 20 | Not | — | A → ¬A | Logical NOT |
| 21 | Call | idx | Args → Ret | Call named function |
| 22 | Ret | — | V → | Return V to caller |
| 23 | Print | — | V → | Print V to output |
| 24 | Read | — | → Str | Read from stdin |
| 25 | Jmp | idx | — | Jump to instruction |
| 26 | JmpIf | idx | Bool → | Jump if false |
| 27 | Nop | — | — | No operation |

### Arrays (28-32)
| Code | Name | Operands | Description |
|------|------|----------|-------------|
| 28 | ArrayNew | u8 count | Create array from N stack values |
| 29 | ArrayGet | — | Get element at index |
| 30 | ArraySet | — | Set element at index |
| 31 | ArrayLen | — | Get array length |
| 32 | ArrayPush | — | Push element to array |

### Closures (33-34)
| Code | Name | Operands | Description |
|------|------|----------|-------------|
| 33 | FuncRef | str_idx | Push function reference |
| 34 | CallValue | u8 argc | Call function from Value::Func |

### Structs (35-37)
| Code | Name | Operands | Description |
|------|------|----------|-------------|
| 35 | StructNew | str_idx | Create struct N fields from stack |
| 36 | StructGet | — | Get field by name |
| 37 | StructSet | — | Set field by name |

### Result (38-40)
| Code | Name | Operands | Description |
|------|------|----------|-------------|
| 38 | ResultOk | — | V → Exito(V) |
| 39 | ResultErr | — | V → Error(V) |
| 40 | TryUnwrap | — | Exito(V)→V, Error(V)→Ret(V) |

## Execution Model
1. Fetch next instruction from bytecode stream
2. Decode opcode and operands
3. Execute: read/write stack, modify locals, jump
4. Repeat until Halt or end of stream

## Call Frames
Each function call creates a frame with:
- Return address (instruction pointer to resume caller)
- Locals: named variables for this scope
- Function name (for stack traces)

## Error Handling
- Division by zero → runtime error with function name and line
- Stack underflow → runtime error
- Type errors → runtime error with explanation
- Undefined variable → runtime error
- All errors include call stack trace (future: Fase 30)

## Bytecode Format (.nvc)
- **Magic**: `LUMN` (4 bytes)
- **Version**: 6 (uint8)
- **Sections**: strings pool, ints pool, floats pool, names pool, function metadata, instruction chunks
- See `docs/spec/bytecode-format.md` for full byte-level specification
