# LÚMEN Virtual Machine Specification v1.0

## Architecture
Stack-based virtual machine with:
- Value stack: runtime operand stack
- Call stack: frames with locals and return addresses
- Heap: arena allocator for strings (bump allocator)

## Value Representation
```rust
enum Value {
    Num(f64),     // IEEE-754 double precision
    Str(String),  // Heap-allocated string
    Bool(bool),   // Boolean
    Void,         // No value
}
```

## Execution Model
1. Fetch next instruction from bytecode stream
2. Decode opcode and operands
3. Execute: read/write stack, modify locals, jump
4. Repeat until Halt or end of stream

## Error Handling
- Division by zero → Runtime error with message
- Stack underflow → Runtime error
- Undefined variable → Runtime error
- Type errors → Runtime error with explanation
