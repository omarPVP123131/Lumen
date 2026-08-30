# Bytecode Format (.nvc) v1.0

## Magic Number
4 bytes: `0x4C 0x55 0x4D 0x4E` ("LUMN")

## Header
- Magic: 4 bytes
- Version: u32 LE (currently 6)
- String count: u32 LE
- Strings: for each: length (u32 LE) + UTF-8 bytes
- Number count: u32 LE
- Numbers: for each: f64 LE (8 bytes)
- Name count: u32 LE
- Names: for each: length (u32 LE) + UTF-8 bytes
- Instruction count: u32 LE

## Instruction Encoding
Each instruction starts with a tag byte:
- 0x00: Simple opcode (1 byte opcode after tag)
- 0x01: WithNum (opcode + u32 LE index into nums table)
- 0x02: WithStr (opcode + u32 LE index into strings table)
- 0x03: WithBool (opcode + 1 byte bool)
- 0x04: WithIdx (opcode + u32 LE index)

## Opcodes
| Code | Name | Operands | Stack effect |
|------|------|----------|--------------|
| 0 | Nop | - | - |
| 1 | PushNum | num_idx | → num |
| 2 | PushStr | str_idx | → str |
| 3 | PushBool | bool_val | → bool |
| 4 | Load | name_idx | → value |
| 5 | Store | name_idx | value → |
| 6 | Add | - | a b → a+b |
| 7 | Sub | - | a b → a-b |
| 8 | Mul | - | a b → a*b |
| 9 | Div | - | a b → a/b |
| 10 | Eq | - | a b → bool |
| 11 | Neq | - | a b → bool |
| 12 | Lt | - | a b → bool |
| 13 | Le | - | a b → bool |
| 14 | Gt | - | a b → bool |
| 15 | Ge | - | a b → bool |
| 16 | And | - | a b → bool |
| 17 | Or | - | a b → bool |
| 18 | Neg | - | a → -a |
| 19 | Not | - | a → !a |
| 20 | Call | name_idx | args... → result |
| 21 | Ret | - | val → |
| 22 | Print | - | val → |
| 23 | Read | - | → str |
| 24 | Jmp | target_idx | - |
| 25 | JmpIf | target_idx | cond → |
| 26 | Halt | - | - |
| 27-46 | Extended | See VM source | Arrays, Closures, Structs, Result, Option, Enum, Tuple, Mod |

Full opcode list (0-46) in `crates/lumen-vm/src/vm.rs`.

---

## v3.5.31+: super-opcodes fusionados (peepholes)

El codegen fusiona secuencias calientes en instrucciones únicas (etiquetas de
instrucción nuevas; ver `crates/lumen-codegen/src/bytecode.rs`):

| Tag | Variante | Forma |
|---|---|---|
| 9 | `FusedBinCmpJmp` | (op1, op2, a, b, c, target) — (a op1 b) op2 c → salto |
| 10 | `FusedBinKCmpJmp` | (op1, op2, a, b, k, target) — b es índice de NOMBRE |
| 11 | `FusedBinKKCmpJmp` | (op1, op2, a, b, k, target) — b es CONSTANTE (shim propio, no reutilizar el de KC) |

Y los de 3 operandos previos: `FusedBin`/`FusedBinK` (a op b → d, k constante),
`FusedCmpKJmp`/`FusedCmpJmp` (comparación + salto). El JIT los compila a código
nativo compacto (ver [../arquitectura/jit.md](../arquitectura/jit.md)); el
intérprete tiene handlers con la MISMA semántica (paridad byte-a-byte).
