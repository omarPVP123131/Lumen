# Especificación del Bytecode de LÚMEN

## Formato del Bytecode

El bytecode de LÚMEN usa un formato binario simple y eficiente.

### Estructura General

```
[OPCODE] [ARGUMENTOS...] [OPCODE] [ARGUMENTOS...] ... [HALT]
```

### Formato de Datos

- **OpCode**: 1 byte
- **i32**: 4 bytes (little-endian)
- **usize**: 4 bytes (little-endian, como u32)

---

## Set de Instrucciones

### Instrucciones Básicas (Capa 1)

#### PUSH_NUM (0x01)
Empuja un número entero al stack.

**Formato:**
```
[0x01] [i32: 4 bytes]
```

**Ejemplo:**
```
0x01 0x0A 0x00 0x00 0x00  // PUSH_NUM 10
```

**Efecto:**
- Stack antes: `[]`
- Stack después: `[10]`

---

#### ADD (0x02)
Suma los dos valores superiores del stack.

**Formato:**
```
[0x02]
```

**Efecto:**
- Stack antes: `[a, b]`
- Stack después: `[a + b]`

---

#### PRINT (0x03)
Imprime el valor superior del stack (sin extraerlo).

**Formato:**
```
[0x03]
```

**Efecto:**
- Imprime el valor superior
- Stack antes: `[value]`
- Stack después: `[]`

---

#### HALT (0xFF)
Detiene la ejecución de la VM.

**Formato:**
```
[0xFF]
```

**Efecto:**
- `running = false`

---

### Aritmética Extendida (Capa 2)

#### SUB (0x04)
Resta dos valores.

**Formato:**
```
[0x04]
```

**Efecto:**
- Stack antes: `[a, b]`
- Stack después: `[a - b]`

---

#### MUL (0x05)
Multiplica dos valores.

**Formato:**
```
[0x05]
```

**Efecto:**
- Stack antes: `[a, b]`
- Stack después: `[a * b]`

---

#### DIV (0x06)
Divide dos valores.

**Formato:**
```
[0x06]
```

**Efecto:**
- Stack antes: `[a, b]`
- Stack después: `[a / b]`
- **Error**: Si `b == 0`

---

### Variables (Capa 2)

#### STORE (0x10)
Guarda el valor superior del stack en una dirección de memoria.

**Formato:**
```
[0x10] [addr: 4 bytes]
```

**Ejemplo:**
```
0x10 0x00 0x00 0x00 0x00  // STORE 0
```

**Efecto:**
- Stack antes: `[value]`
- Stack después: `[]`
- `memory[addr] = value`

---

#### LOAD (0x11)
Carga un valor desde memoria al stack.

**Formato:**
```
[0x11] [addr: 4 bytes]
```

**Ejemplo:**
```
0x11 0x00 0x00 0x00 0x00  // LOAD 0
```

**Efecto:**
- Stack antes: `[]`
- Stack después: `[memory[addr]]`

---

### Comparaciones (Capa 2)

Todas las comparaciones retornan:
- `1` si la condición es verdadera
- `0` si la condición es falsa

#### EQ (0x20)
Compara si dos valores son iguales.

**Formato:**
```
[0x20]
```

**Efecto:**
- Stack antes: `[a, b]`
- Stack después: `[a == b ? 1 : 0]`

---

#### LT (0x21)
Compara si el primer valor es menor que el segundo.

**Formato:**
```
[0x21]
```

**Efecto:**
- Stack antes: `[a, b]`
- Stack después: `[a < b ? 1 : 0]`

---

#### GT (0x22)
Compara si el primer valor es mayor que el segundo.

**Formato:**
```
[0x22]
```

**Efecto:**
- Stack antes: `[a, b]`
- Stack después: `[a > b ? 1 : 0]`

---

### Control de Flujo (Capa 2)

#### JMP (0x30)
Salto incondicional a una dirección.

**Formato:**
```
[0x30] [addr: 4 bytes]
```

**Ejemplo:**
```
0x30 0x14 0x00 0x00 0x00  // JMP 20
```

**Efecto:**
- `IP = addr`

---

#### JMP_IF_FALSE (0x31)
Salto condicional si el valor del stack es 0.

**Formato:**
```
[0x31] [addr: 4 bytes]
```

**Ejemplo:**
```
0x31 0x28 0x00 0x00 0x00  // JMP_IF_FALSE 40
```

**Efecto:**
- Stack antes: `[condition]`
- Stack después: `[]`
- Si `condition == 0`: `IP = addr`
- Si `condition != 0`: continúa normalmente

---

## Ejemplos Completos

### Ejemplo 1: Suma Simple

**Código:**
```lumen
numero a = 5
numero b = 3
imprimir(a + b)
```

**Bytecode (hexadecimal):**
```
01 05 00 00 00  // PUSH_NUM 5
10 00 00 00 00  // STORE 0
01 03 00 00 00  // PUSH_NUM 3
10 01 00 00 00  // STORE 1
11 00 00 00 00  // LOAD 0
11 01 00 00 00  // LOAD 1
02              // ADD
03              // PRINT
FF              // HALT
```

**Tamaño:** 29 bytes

---

### Ejemplo 2: Condicional

**Código:**
```lumen
numero x = 10
si (x > 5) {
    imprimir(1)
} sino {
    imprimir(0)
}
```

**Bytecode (comentado):**
```
01 0A 00 00 00  // PUSH_NUM 10
10 00 00 00 00  // STORE 0 (x)
11 00 00 00 00  // LOAD 0
01 05 00 00 00  // PUSH_NUM 5
22              // GT
31 1F 00 00 00  // JMP_IF_FALSE 31 (a else)
01 01 00 00 00  // PUSH_NUM 1
03              // PRINT
30 24 00 00 00  // JMP 36 (a fin)
01 00 00 00 00  // PUSH_NUM 0  [posición 31]
03              // PRINT
FF              // HALT  [posición 36]
```

---

### Ejemplo 3: Loop While

**Código:**
```lumen
numero i = 0
mientras (i < 3) {
    imprimir(i)
    i = i + 1
}
```

**Bytecode (comentado):**
```
01 00 00 00 00  // PUSH_NUM 0
10 00 00 00 00  // STORE 0 (i)
                // [loop_start = 10]
11 00 00 00 00  // LOAD 0
01 03 00 00 00  // PUSH_NUM 3
21              // LT
31 2D 00 00 00  // JMP_IF_FALSE 45 (fin)
11 00 00 00 00  // LOAD 0
03              // PRINT
11 00 00 00 00  // LOAD 0
01 01 00 00 00  // PUSH_NUM 1
02              // ADD
10 00 00 00 00  // STORE 0
30 0A 00 00 00  // JMP 10 (loop_start)
FF              // HALT  [posición 45]
```

---

## Consideraciones de Implementación

### Límites

- **Stack máximo:** 1024 elementos
- **Memoria:** HashMap ilimitado (en práctica, limitado por RAM)
- **Tamaño de bytecode:** Sin límite teórico

### Endianness

Todos los valores multi-byte usan **little-endian**.

**Ejemplo:** El número `300` (0x12C) se codifica como:
```
0x2C 0x01 0x00 0x00
```

### Validaciones

La VM valida:
- ✅ OpCodes válidos
- ✅ Stack underflow/overflow
- ✅ División por cero
- ✅ Saltos a direcciones válidas
- ✅ Acceso a memoria inicializada

---

## Formato de Archivo .nvc

Los archivos bytecode compilados usan la extensión `.nvc` (NaVa Code).

### Estructura del archivo

```
[HEADER: 8 bytes]
[BYTECODE: N bytes]
```

### Header (futuro - no implementado aún)

```
Offset  Size  Descripción
0       4     Magic number: 0x4E564300 ("NVC\0")
4       2     Versión mayor
6       2     Versión menor
```

**Actualmente:** Los archivos `.nvc` son bytecode puro sin header.

---

## Depuración

### Instrucción DEBUG_STACK (0xFE)

**Uso interno:** Imprime el estado actual del stack.

```
[0xFE]
```

**Efecto:**
```
[DEBUG] Stack: [1, 2, 3]
```

Esta instrucción NO se genera desde el compilador, solo se usa manualmente para debugging.

---

## Evolución Futura

### Versión 1.0
- Header en archivos `.nvc`
- Verificación de versiones

### Versión 2.0
- Instrucciones de punto flotante
- Strings y arrays
- Funciones y llamadas
- Opcodes de I/O

---

**Documento actualizado:** Febrero 2026  
**Versión de LÚMEN:** 0.1.0
