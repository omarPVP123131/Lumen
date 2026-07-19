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
    Void,                                       // Ausencia de valor
}
```

## Opcodes (37 total)

### Core (0-27)
| Code | Name | Operands | Stack Effect | Description |
|------|------|----------|-------------|-------------|
| 0 | Halt | — | — | Stop execution |
| 1 | PushStr | u16 index | → Str | Push string from pool |
| 2 | PushInt | u16 index | → Int | Push int from pool |
| 3 | PushNum | u16 index | → Float | Push float from pool |
| 4 | PushBool | u8 value | → Bool | Push true(1)/false(0) |
| 5 | PushVoid | — | → Void | Push void |
| 6 | Pop | — | Value → | Pop and discard |
| 7 | Dup | — | V → V V | Duplicate top |
| 8 | Add | — | A B → A+B | Addition |
| 9 | Sub | — | A B → A-B | Subtraction |
| 10 | Mul | — | A B → A*B | Multiplication |
| 11 | Div | — | A B → A/B | Division |
| 12 | Eq | — | A B → A==B | Equality |
| 13 | Neq | — | A B → A≠B | Not equal |
| 14 | Lt | — | A B → A<B | Less than |
| 15 | Lte | — | A B → A≤B | Less or equal |
| 16 | Gt | — | A B → A>B | Greater than |
| 17 | Gte | — | A B → A≥B | Greater or equal |
| 18 | And | — | A B → A∧B | Logical AND |
| 19 | Or | — | A B → A∨B | Logical OR |
| 20 | Not | — | A → ¬A | Logical NOT |
| 21 | Neg | — | A → -A | Arithmetic negate |
| 22 | Load | u16 index | → Value | Load local by index |
| 23 | Store | u16 index | Value → | Store local by index |
| 24 | Jmp | i16 offset | — | Unconditional jump |
| 25 | JmpIfFalse | i16 offset | Bool → | Jump if false |
| 26 | Print | — | Value → | Print value to stdout |
| 27 | Ret | — | Value → | Return to caller |

### Call & Functions (28-34)
| Code | Name | Operands | Description |
|------|------|----------|-------------|
| 28 | Call | u8 argc | Call function by name from pool |
| 29 | FuncRef | u16 str_idx | Push function reference |
| 30 | CallValue | u8 argc | Call function from Value::Func |
| 31 | EnterScope | u16 num_locals | Allocate locals for new scope |
| 32 | ExitScope | u16 num_locals | Deallocate locals |

### Arrays (33-37 earlier now shifted — adjust numbering)

Actually the current opcode assignments in bytecode.rs are:
- 0-27: Core (as above)
- 28-29: ArrayNew, ArrayGet
- 30-32: ArraySet, ArrayLen, ArrayPush
- 33-34: FuncRef, CallValue
- 35-37: StructNew, StructGet, StructSet

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
- **Version**: 5 (uint8)
- **Sections**: strings pool, ints pool, floats pool, names pool, function metadata, instruction chunks
- See `docs/spec/bytecode-format.md` for full byte-level specification
