# LÚMEN — Exhaustive Source Code Audit Report

**Generated:** 2026-07-21  
**Version:** LÚMEN 1.2.0  
**Scope:** All 7 crates (lexer, parser, sema, ir, codegen, vm, cli)  
**Methodology:** Static analysis + code review by 3 autonomous review agents

> **Synced (2026-08-09):** Historical audit report. The `test_agents/K01-K20.nv` files are **not present** in the current repo — `test_agents/` contains 45 files (01-10, J01-J15, S01-S15, debug_*). Current state: v2.4.1 (release published, 4 multi-OS binaries), ~378 tests, double bootstrapping certified (SHA-256 3DA624D6…), fuego 117/117 · 112 CORRECT.

---

## Verdict: **NEEDS WORK** ⚠️

10 critical issues found. Production readiness requires addressing all critical + high issues before release.

---

## 1. CRITICAL ISSUES (10 total)

### C1 — `CallValue` missing builtin handlers
**File:** `crates/lumen-vm/src/vm.rs:767-817`  
**Severity:** CRITICAL  
**Description:** The `CallValue` handler (lambda/closure invocation) only handles `imprimir`/`print`, `leer`/`read`, and `__str_*` builtins. Missing: `largo`/`len`, `agregar`/`push`, `__file_read`, `__file_write`, `__file_exists`, `__time_now`, `__list_reverse`, `__list_sort`.  
**Impact:** Lambdas calling stdlib functions crash with "function not defined".  
**Fix:** Add all missing builtins to `CallValue` dispatch, or refactor into shared helper.

### C2 — `synchronize()` consumes match/struct/enum boundaries
**File:** `crates/lumen-parser/src/parser.rs:857-900, 2420-2454`  
**Severity:** CRITICAL  
**Description:** `synchronize()` does not stop on `Caso`/`Case`, `Defecto`/`Default`, `RightBrace`, `Estructura`/`Struct`, or `Enum`. After a parse error inside a match arm, synchronize eats subsequent case arms and the closing `}`.  
**Impact:** Cascading error reports after any syntax error near match/struct/enum.  
**Fix:** Add missing token kinds to `synchronize()` stop list.

### C3 — Generic type parameters not resolved inside compound types
**File:** `crates/lumen-sema/src/sema.rs:1819-1852, 1854-1916`  
**Severity:** CRITICAL  
**Description:** `resolve_type()` delegates non-struct types to `type_to_info()`, which has no access to `type_params`. `Type::Struct("T")` inside `lista<T>`, `resultado<T,E>`, `opcion<T>`, tuple types resolves to `TypeInfo::Struct { name: "T", fields: [] }` instead of `TypeInfo::TypeVar("T")`.  
**Impact:** `funcion T foo<T>(lista<T> items)` produces incorrect type info.  
**Fix:** Thread `type_params` through `type_to_info()`.

### C4 — `Store` uses `.unwrap()` on `locals.last_mut()` — potential panic
**File:** `crates/lumen-vm/src/vm.rs:594`  
**Severity:** CRITICAL  
**Description:** `self.locals.last_mut().unwrap().insert(name, val)` panics if `locals` stack is empty. Surrounding code uses `?` for error propagation.  
**Impact:** Corrupt bytecode causes panic instead of clean error.  
**Fix:** Replace `.unwrap()` with error propagation (`?` or `ok_or`).

### C5 — `usize` in bytecode breaks on 32-bit platforms
**File:** `crates/lumen-codegen/src/bytecode.rs:187, 371`; `codegen.rs:371`  
**Severity:** CRITICAL  
**Description:** `FuncMeta.start` is `usize` (platform-dependent). Encoding uses `to_le_bytes()` yielding 4 or 8 bytes; decoding always reads 8.  
**Impact:** Crashes on 32-bit platforms.  
**Fix:** Use fixed-size type (u64) in bytecode format.

### C6 — Fragile instruction read-ahead in Call/StructNew/EnumCtor
**Files:** `crates/lumen-vm/src/vm.rs:597-608, 830-840, 909-935`  
**Severity:** CRITICAL  
**Description:** These opcodes read subsequent instructions as inline metadata by advancing `self.ip`. No validation that read-ahead instructions are of expected form. EnumCtor consumes 2 extra instructions.  
**Impact:** Silent misbehavior or crashes on malformed bytecode.  
**Fix:** Add validation and error propagation.

### C7 — `Ret` at top level pollutes stack and continues execution
**File:** `crates/lumen-vm/src/vm.rs:327-333`  
**Severity:** CRITICAL  
**Description:** When `Ret` executes in main function (empty `call_stack`), return value is pushed but execution continues.  
**Impact:** Duplicated stack values, incorrect results.  
**Fix:** Halt execution on top-level `Ret` or don't push value.

### C8 — `Stmt::ForEach` does not increment `loop_depth`
**File:** `crates/lumen-sema/src/sema.rs:606-637`  
**Severity:** CRITICAL  
**Description:** Unlike `While` and `For`, `ForEach` handler never sets `loop_depth += 1`.  
**Impact:** `romper`/`continuar` inside for-each loops produce spurious errors.  
**Fix:** Add `self.loop_depth += 1/self.loop_depth -= 1` around body analysis.

### C9 — `Expr::List` type mismatch detection has empty error body
**File:** `crates/lumen-sema/src/sema.rs:1350-1356`  
**Severity:** CRITICAL  
**Description:** List element type mismatch check has empty body — no error emitted.  
**Impact:** `[1, "hola"]` passes semantic analysis silently.  
**Fix:** Add error emission for type mismatch.

### C10 — Duplicate module flattening due to premature `visited.remove()`
**File:** `crates/lumen-sema/src/loader.rs:68`  
**Severity:** CRITICAL  
**Description:** `visited.remove()` is called after each recursive `flatten()`. Module imported by two others loads twice with different prefixes.  
**Impact:** Duplicate declarations, confusing sema errors.  
**Fix:** Remove only after all imports of the current module are processed.

---

## 2. HIGH ISSUES (15 total)

### H1 — `synchronize()` missing struct/enum/option tokens
**File:** `crates/lumen-parser/src/parser.rs:2420-2454`  
Missing: `Estructura`/`Struct`, `Enum`, `Opcion`/`Option` in synchronize stop list.

### H2 — Missing Spanish/English pairs in `token_matches()`
**File:** `crates/lumen-parser/src/parser.rs:2463-2525`  
Missing: `Opcion`↔`Option`, `Algun`↔`Some`, `Ninguno`↔`None`.

### H3 — `as_str()` returns empty string for operators/delimiters
**File:** `crates/lumen-lexer/src/token.rs:293-294`  
Catch-all `_ => ""` for non-keyword variants. Used in error messages.

### H4 — Unrecognized escape sequences silently pass through
**File:** `crates/lumen-lexer/src/lexer.rs:79`  
`\x`, `\q` silently produce literal `x`, `q` instead of warning.

### H5 — `Lista`/`Array` without type param defaults to `Lista<Decimal>`
**File:** `crates/lumen-parser/src/parser.rs:1917`  
Silent type assumption rather than requiring explicit type.

### H6 — `parse_for()` silently discards errors
**File:** `crates/lumen-parser/src/parser.rs:694-705`  
Returns `None` without error for missing `(`, `;`, `)`.

### H7 — `Decl::Variable` double-evaluates init expression
**File:** `crates/lumen-sema/src/sema.rs:206-221`  
`analyze_expr(init)` called twice — duplicate work.

### H8 — `Stmt::FieldAssign` for chained access fails to store back
**File:** `crates/lumen-ir/src/builder.rs:251-265`  
`a.b.c = 42` — LHS not recognized as variable, no Store emitted.

### H9 — Missing LessEqual/GreaterEqual in mixed int/float folding
**File:** `crates/lumen-ir/src/ir.rs:802-826`  
Mixed Int/Float comparisons skip `<=` and `>=` folding.

### H10 — EnumCtor variant encoding brittle
**File:** `crates/lumen-codegen/src/codegen.rs:331-348`, `vm.rs:908-946`  
Inline Nop data — breaks if any pass inserts/removes instructions.

### H11 — Massive code duplication between Call and CallValue
**File:** `crates/lumen-vm/src/vm.rs:614-745` vs `767-817`  
~130 lines copy-pasted. Already diverged (missing builtins).

### H12 — `decode` maps unknown opcodes to Nop silently
**File:** `crates/lumen-codegen/src/bytecode.rs:404`  
Unknown opcode → `Nop` instead of error.

### H13 — `show_error` underline ignores multi-byte characters
**File:** `crates/lumen-cli/src/main.rs:213-217`  
Caret position assumes 1 byte = 1 column. Fails with `ñ`, `ü`, emoji.

### H14 — `list_sort` coerces non-numeric types to `f64::MAX`
**File:** `crates/lumen-vm/src/vm.rs:722-725`  
Sorting strings/structs silently replaces data with `f64::MAX`.

### H15 — `target.var_type` ignored in assignment destructure
**File:** `crates/lumen-sema/src/sema.rs:639-694`  
Typed destructure assignment doesn't verify type annotations.

---

## 3. MEDIUM ISSUES (21 total)

### Lexer/Parser (8)
- M1: `parse_destructure_assign_stmt()` doesn't support `_` wildcard
- M2: `check_next_is_tuple_type()` doesn't accept generic struct idents
- M3: `check_ident_next_is_generic_type()` doesn't verify ident is known type
- M4: `parse_import()` accepts keywords as module names
- M5: `prev_pos()` saturates at col 1 instead of checked subtraction
- M6: Error code E012 used for multiple different error types
- M7: `NumLiteral("1.")` with trailing dot accepted
- M8: `is_ident_start/continue` is ASCII-only (no `ñ`, `á`, `é`)

### Sema/IR/Codegen (7)
- M9: `Decl::Variable` fallback uses `TypeInfo::Numero` instead of `Void`
- M10: EnumCtor argc encoded as float in nums pool
- M11: VM silently handles wrong instruction type (defaults to 0)
- M12: `Instr::Read` is completely dead code
- M13: `prefix_type` doesn't handle generic enum variant types
- M14: `resolve_type` for Opcion/Tuple doesn't handle type params
- M15: `can_assign` for Resultado/Opcion allows Void to match anything

### VM/CLI (6)
- M16: `Ret` creates garbage stack entries for every Call
- M17: Enum variant lookup uses raw index without validation
- M18: `encode` uses first pool entry as fallback for missing numbers
- M19: VM has no stack depth limit (OOM on deep recursion)
- M20: `decode` silently truncates corrupt function metadata
- M21: `Opcode::Read` (24) defined but never handled in VM

---

## 4. LOW ISSUES (19 total)

| # | Issue | File |
|---|-------|------|
| L1 | Lexer None branch unreachable | `lexer.rs:230-231` |
| L2 | `\r\n` not handled as single newline | `lexer.rs:250-252` |
| L3 | `parse_function()` redundant type_params stack mgmt | `parser.rs:278-286` |
| L4-5 | Keyword tests incomplete | `lexer.rs:327-374` |
| L6 | Error code E012 overloaded | `parser.rs` multiple |
| L7 | Trailing dot in number accepted | `lexer.rs:196-201` |
| L8 | ASCII-only identifiers | `lexer.rs:289-294` |
| L9 | Nop dual-use as no-op and data carrier | `codegen.rs:197,280,343` |
| L10 | `num_cache` partially redundant | `codegen.rs:11,57-65` |
| L11 | `Expr::Ident` fallback type Numero vs Void | `sema.rs:714` |
| L12 | `for` loop update not validated | `sema.rs:467` |
| L13 | `Func::params` no default value storage in IR | `ir.rs:76-81` |
| L14 | `disasm_file` uses `print!` instead of `println!` | `main.rs:356` |
| L15 | `-0.0` truthiness (IEEE 754) | `value.rs:65` |
| L16 | Test file name convention inconsistency | `vm.rs:975` |
| L17 | No `--` separator handling in CLI | `main.rs:19-65` |
| L18 | `source.lines()` misaligns `\r\n` columns | `main.rs:207` |
| L19 | `-v`/`--version` not in early return check | `main.rs:69-73` |

---

## 5. RECOMMENDATIONS

### Immediate (Fix Before Next Release)
1. **C1**: Add missing builtins to `CallValue` dispatch
2. **C2**: Fix `synchronize()` stop list
3. **C4**: Fix `Store` unwrap → error propagation
4. **C8**: Add `loop_depth` to ForEach
5. **C9**: Fix list type mismatch detection

### Short-term
6. **C3**: Thread type_params through type_to_info()
7. **C6**: Validate instruction read-ahead in VM
8. **C7**: Handle top-level Ret properly
9. **C10**: Fix module visited.remove() placement
10. **H1-H15**: Address all high-severity issues

### Long-term
11. **C5**: Fix usize in bytecode for 32-bit support
12. Deduplicate Call/CallValue builtin dispatch
13. Add stack depth limit to VM
14. UTF-8 aware error display
15. Complete Spanish boolean aliases (`y`, `o`, `no`)

---

## Files Changed in This Audit

```
crates/lumen-lexer/src/token.rs        + is_english(), is_spanish() methods
crates/lumen-parser/src/parser.rs      + english_mode, detect_import_english, check() filter
crates/lumen-sema/src/loader.rs        + skip import ingles in flatten()
docs/language.md                        + importar ingles documentation
docs/spec/error-codes.md                + E070 error code
AGENTS.md                               + Phase 32b documentation
test_agents/*.nv                        + importar ingles; to all 65 files
test_agents/K01_K20.nv                  + 20 new test files (NOT in repo — see sync note)
reports/                                + This audit report suite
```

---

*Audit conducted by 3 autonomous code review agents*  
*65 issues found (10 Critical, 15 High, 21 Medium, 19 Low)*
