# Fixpoint self-hosting — Sat Aug 29 09:04:43 UTC 2026
host: Linux e2b.local 6.1.158+ #1 SMP PREEMPT_DYNAMIC Fri Jul 17 14:31:34 UTC 2026 x86_64 GNU/Linux
binario: LÚMEN v3.5.7
## Paso 1: compiler_v4.nvc (compilado por Rust)...
Bytecode generado: stdlib/compiler/compiler_v4.nvc
## Paso 2 (STAGE 1): autocompilar compiler_v4.nv (~5 s con lexer nativo)...
ENTRADA: stdlib/compiler/compiler_v4.nv
SALIDA: /tmp/v4_self.nvc
Source: 149916 bytes
Tokens: 30596
AST: Programa
[codegen] 0%  funcion 0/115
[codegen] 6%  funcion 8/115
[codegen] 13%  funcion 16/115
[codegen] 20%  funcion 24/115
[codegen] 27%  funcion 32/115
[codegen] 34%  funcion 40/115
[codegen] 41%  funcion 48/115
[codegen] 83%  funcion 96/115
[codegen] 90%  funcion 104/115
[codegen] 97%  funcion 112/115
Instrs: 18541
OK: /tmp/v4_self.nvc (170985 bytes)
v4_self.nvc: 170985 bytes
funciones duplicadas en v4_self: [NINGUNA ✔]
total funciones: 66
## Paso 3 (PROBE): v4_self.nvc compila selfhost_probe.nv (esperado: 42)...
probe via SELF-COMPILED = [42] (esperado 42)
## Paso 4 (STAGE 2): v4_self.nvc recompila compiler_v4.nv (~5 s con lexer nativo)...
ENTRADA: stdlib/compiler/compiler_v4.nv
SALIDA: /tmp/v4_self2.nvc
Source: 149916 bytes
Tokens: 30596
AST: Programa
[codegen] 0%  funcion 0/115
[codegen] 6%  funcion 8/115
[codegen] 13%  funcion 16/115
[codegen] 20%  funcion 24/115
[codegen] 27%  funcion 32/115
[codegen] 34%  funcion 40/115
[codegen] 41%  funcion 48/115
[codegen] 83%  funcion 96/115
[codegen] 90%  funcion 104/115
[codegen] 97%  funcion 112/115
Instrs: 18541
OK: /tmp/v4_self2.nvc (170985 bytes)
## ✅ FIXPOINT: v4_self.nvc y v4_self2.nvc BYTE-IDENTICAL (170985 B)
sha256: 02b0460db823c143ad272054eb2ea4e4e16554b63ac37bad3c6cac762f2d4d5b
fin: Sat Aug 29 09:04:51 UTC 2026
