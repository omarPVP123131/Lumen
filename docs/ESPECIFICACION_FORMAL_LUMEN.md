# 📜 Especificación Formal del Lenguaje LÚMEN (ISO/IEC-Style)

**Estándar Internacional LÚMEN-2026 — v3.0**

---

## 1. Alcance y Filosofía de Diseño
LÚMEN es un lenguaje de programación bilingüe de sistemas de propósito general, tipado estático, con soporte para paradigmas funcional, procedimental y de actores concurrentes. Esta especificación define formalmente la sintaxis, semántica operacional, modelo de memoria y reglas de tipado del estándar oficial.

---

## 2. Sintaxis Léxica Formal (EBNF)

```ebnf
Programa        ::= ( Declaracion | Sentencia )* <EOF>

Declaracion     ::= DeclVariable | DeclFuncion | DeclEstructura | DeclEnum | DeclRasgo | DeclImpl
DeclVariable    ::= ( "sea" | "let" | Tipo ) Identificador [ "=" Expresion ] ";"
DeclFuncion     ::= [ "puro" | "pure" ] [ "async" ] "funcion" Tipo Identificador "(" Parametros? ")" Bloque
DeclEstructura  ::= "estructura" Identificador "{" ( Identificador ":" Tipo ","? )* "}"
DeclEnum        ::= "enum" Identificador "{" ( Identificador [ "(" ( Tipo ","? )* ")" ] ","? )* "}"

Tipo            ::= TipoPrimitivo 
                  | "lista" "<" Tipo ">" 
                  | "opcion" "<" Tipo ">" 
                  | "resultado" "<" Tipo "," Tipo ">"
                  | "prestado" [ "mut" ] Tipo
                  | "dueno" Tipo
                  | Tipo "?"

TipoPrimitivo   ::= "entero" | "decimal" | "texto" | "booleano" | "numero" | "vacio"
```

---

## 3. Semántica de Memoria y Teoría de Tipos

### 3.1 Modelo NaN-Boxing de 64 bits (`NanVal`)
Cualquier valor en tiempo de ejecución en la Máquina Virtual se representa mediante una palabra escalar de 64 bits (`u64`) bajo el estándar IEEE 754:
* `Máscara QNAN`: `0x7ff8_0000_0000_0000`
* `TAG_INT`  = `0x7ff9_0000_0000_0000`
* `TAG_BOOL` = `0x7ffa_0000_0000_0000`
* `TAG_VOID` = `0x7ffb_0000_0000_0000`
* `TAG_PTR`  = `0x7ffc_0000_0000_0000` (48 bits inferiores contienen la dirección física del heap).

### 3.2 Reglas del Borrow Checker (Tipos Afines)
1. **Regla de Exclusión Mutua (XOR Aliasing)**: Para cualquier recurso $R$, puede existir $N$ referencias `prestado R` **O** exactamente 1 referencia `prestado mut R`, pero nunca ambas simultáneamente.
2. **Transferencia de Propiedad (Move Semantics)**: Al asignar un valor `dueno T`, la variable de origen queda invalidada para accesos posteriores en tiempo de compilación.

---

## 4. Semántica Operacional de Concurrencia y Resiliencia

* **Actores OTP**: Procesos aislados que se comunican exclusivamente por paso de mensajes asíncronos en colas FIFO (*mailboxes*).
* **Self-Healing Runtime**: Si una instrucción lanza una excepción no capturada, el runtime intercepta el fallo en la frontera de la fibra, ejecuta los bloques `posponer` en orden LIFO y transfiere la ejecución a la versión de parche registrada (`hot-patch`).

---

*LÚMEN Standard Specification — Certificación Oficial 2026.*
