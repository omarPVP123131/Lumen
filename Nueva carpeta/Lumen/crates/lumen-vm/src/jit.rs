//! JIT Tier-1 de LÚMEN (v3.5.9) — bytecode caliente → código nativo Cranelift.
//!
//! Diseño "correcto por construcción": el código nativo NO reimplementa ninguna
//! semántica; cada opcode se delega a los MISMOS handlers del intérprete a
//! través de helpers `extern "C"` (lj_*). El JIT solo elimina el costo de
//! dispatch/decodificación y ejecuta los saltos (Jmp/JmpIf/Ret) de forma nativa.
//!
//! v3.5.31: PREDETERMINADO (builds con feature `aot`); `LUMEN_JIT=0` para
//! desactivarlo y correr en el intérprete puro (diagnóstico/fixpoint).
//! Tier-2 compila bucles Int puros a código nativo directo; Tier-1 delega
//! cada instrucción a los handlers compartidos con el intérprete.

use cranelift::codegen::ir::{BlockArg, FuncRef};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use lumen_codegen::bytecode::{Bytecode, Instruction, Opcode};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::vm::{VmError, VM};

/// v3.5.37: etiqueta de tipo para el análisis estático de pila del Tier-2
/// en modo texto (dyn_arith) — decide si Add/Sub/Mul son aritmética Int
/// nativa, concat rápido o shim genérico.
/// Etiqueta de elementos de un array (para ArrayGet/ArrayPushVar).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ETag {
    Any,
    Int,
    Str,
}

#[inline(always)]
fn merge_etag(a: ETag, b: ETag) -> ETag {
    if a == b {
        a
    } else {
        ETag::Any
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum VTag {
    Any,
    Int,
    Str,
    /// v3.5.37: array con etiqueta de elementos conocida (Int → ArrayGet
    /// seguido de aritmética Int nativa; Any → conservador).
    Arr(ETag),
}

#[inline(always)]
fn merge_tag(a: VTag, b: VTag) -> VTag {
    match (a, b) {
        (VTag::Arr(x), VTag::Arr(y)) => VTag::Arr(merge_etag(x, y)),
        (x, y) if x == y => x,
        _ => VTag::Any,
    }
}

fn merge_stacks(a: &[VTag], b: &[VTag]) -> Vec<VTag> {
    let n = a.len().max(b.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(merge_tag(
            a.get(i).copied().unwrap_or(VTag::Any),
            b.get(i).copied().unwrap_or(VTag::Any),
        ));
    }
    out
}

/// Firma nativa: `fn(vm: *mut VM) -> i64` — 0 = ok, 1 = error (ver `VM::jit_error`).
pub type JitFn = unsafe extern "C" fn(*mut std::ffi::c_void) -> i64;

unsafe fn vm_ref<'a>(p: *mut std::ffi::c_void) -> &'a mut VM {
    &mut *(p as *mut VM)
}

fn op_from(i: i64) -> Option<Opcode> {
    Opcode::from_u8(i as u8)
}

// ──────────────────────────────────────────────────────────────────────
// Runtime helpers — el código nativo llama a estos; ellos delegan a los
// handlers exactos del intérprete (paridad de semántica garantizada).
// ──────────────────────────────────────────────────────────────────────

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
/// v3.5.37: concat rápido (Add con algún operando Str) — reproduce el
/// arm Add del intérprete con UN shim en vez del handler genérico.
pub extern "C" fn lj_concat(p: *mut std::ffi::c_void) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.concat_pub() {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn lj_simple(p: *mut std::ffi::c_void, op: i64, ip: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.set_ip_pub(ip as usize);
    match op_from(op) {
        Some(o) => match vm.execute_simple_pub(o) {
            Ok(()) => 0,
            Err(e) => {
                vm.set_jit_error(e);
                1
            }
        },
        None => {
            vm.set_jit_error(VmError::Runtime("JIT: opcode inválido".into()));
            1
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_with_num(p: *mut std::ffi::c_void, op: i64, n: f64, ip: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.set_ip_pub(ip as usize);
    match op_from(op) {
        Some(o) => match vm.execute_with_num_pub(o, n) {
            Ok(()) => 0,
            Err(e) => {
                vm.set_jit_error(e);
                1
            }
        },
        None => {
            vm.set_jit_error(VmError::Runtime("JIT: opcode inválido".into()));
            1
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_with_bool(p: *mut std::ffi::c_void, op: i64, b: i64, ip: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.set_ip_pub(ip as usize);
    match op_from(op) {
        Some(o) => match vm.execute_with_bool_pub(o, b != 0) {
            Ok(()) => 0,
            Err(e) => {
                vm.set_jit_error(e);
                1
            }
        },
        None => {
            vm.set_jit_error(VmError::Runtime("JIT: opcode inválido".into()));
            1
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_with_idx(p: *mut std::ffi::c_void, op: i64, idx: i64, ip: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.set_ip_pub(ip as usize);
    match op_from(op) {
        Some(o) => match vm.execute_with_idx_pub(o, idx as usize) {
            Ok(()) => 0,
            Err(e) => {
                vm.set_jit_error(e);
                1
            }
        },
        None => {
            vm.set_jit_error(VmError::Runtime("JIT: opcode inválido".into()));
            1
        }
    }
}

/// WithStr: el string vive en `Bytecode::instructions`, que es inmutable y vive
/// tanto como la VM → el puntero crudo es estable durante toda la ejecución.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_with_str(
    p: *mut std::ffi::c_void,
    op: i64,
    ptr: i64,
    len: i64,
    ip: i64,
) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.set_ip_pub(ip as usize);
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    let s = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => {
            vm.set_jit_error(VmError::Runtime("JIT: string inválido".into()));
            return 1;
        }
    };
    match op_from(op) {
        Some(o) => match vm.execute_with_str_pub(o, s) {
            Ok(()) => 0,
            Err(e) => {
                vm.set_jit_error(e);
                1
            }
        },
        None => {
            vm.set_jit_error(VmError::Runtime("JIT: opcode inválido".into()));
            1
        }
    }
}

/// Llamada a función (secuencia Call+argc del bytecode). Reproduce el opcode
/// Call completo: builtins, funciones JIT-compileadas (recursión nativa) y
/// funciones interpretadas (ejecución anidada hasta el retorno).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_call(p: *mut std::ffi::c_void, nidx: i64, argc: i64, ip: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.set_ip_pub(ip as usize);
    match vm.perform_call_idx(nidx as usize, argc as usize) {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
}

/// v3.5.32: llamada SIN pre-filtro de builtins — el JIT solo la emite
/// cuando el nombre NO es builtin (decisión estática en compilación).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_call_fast(p: *mut std::ffi::c_void, nidx: i64, argc: i64, ip: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.set_ip_pub(ip as usize);
    match vm.perform_call_fast(nidx as usize, argc as usize) {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
}

/// v3.5.32 (Tier-2): lookup + guarda de tipo fusionadas — slot si existe y
/// es Int, -1 si no (un solo call de prólogo por nombre).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_probe_int(p: *mut std::ffi::c_void, name_idx: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.probe_int_pub(name_idx as usize)
}

/// JmpIf: pop + truthiness. 0/1 = valor, -1 = error.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_truth(p: *mut std::ffi::c_void) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.pop_pub() {
        Ok(v) => v.is_truthy() as i64,
        Err(e) => {
            vm.set_jit_error(e);
            -1
        }
    }
}

/// Ret: delega al handler real (write-back de refs, frames, corutinas...).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_ret(p: *mut std::ffi::c_void) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.execute_simple_pub(Opcode::Ret) {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// v3.5.31: helpers de super-opcodes — el JIT ejecuta los Fused* nativos
// delegando al MISMO handler del intérprete (paridad de semántica), y los
// saltos condicionales los resuelve en CLIF con el resultado devuelto.
// ──────────────────────────────────────────────────────────────────────

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_bink(p: *mut std::ffi::c_void, op: i64, a: i64, k: i64, d: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.exec_fused_bink_pub(op as u8, a as usize, k, d as usize) {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_bin(p: *mut std::ffi::c_void, op: i64, a: i64, b: i64, d: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.exec_fused_bin_pub(op as u8, a as usize, b as usize, d as usize) {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
}

/// v3.5.41 (bug #10): variantes de DECLARACIÓN (StoreLocal) — el destino
/// se escribe en el scope actual del frame, nunca en scopes de frames
/// ancestros (paridad con el intérprete: mismos helpers de store).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_bink_local(
    p: *mut std::ffi::c_void,
    op: i64,
    a: i64,
    k: i64,
    d: i64,
) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.exec_fused_bink_local_pub(op as u8, a as usize, k, d as usize) {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_bin_local(
    p: *mut std::ffi::c_void,
    op: i64,
    a: i64,
    b: i64,
    d: i64,
) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.exec_fused_bin_local_pub(op as u8, a as usize, b as usize, d as usize) {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
}

/// Solo evalúa `a op k`: -1 = error, 0 = falso, 1 = verdadero (el JIT salta).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_cmpk(p: *mut std::ffi::c_void, op: i64, a: i64, k: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.fused_cmpk_pub(op as u8, a as usize, k) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            -1
        }
    }
}

/// Solo evalúa `a op b`: -1 = error, 0 = falso, 1 = verdadero (el JIT salta).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_cmp(p: *mut std::ffi::c_void, op: i64, a: i64, b: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.fused_cmp_pub(op as u8, a as usize, b as usize) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            -1
        }
    }
}

/// v3.5.31: condición de (a op1 b) op2 c — tri-state (-1 error/0/1).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_bincmp(
    p: *mut std::ffi::c_void,
    op1: i64,
    op2: i64,
    a: i64,
    b: i64,
    c: i64,
) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.fused_bincmp_pub(op1 as u8, op2 as u8, a as usize, b as usize, c as usize) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            -1
        }
    }
}

/// v3.5.31: condición de (a op1 b) op2 k — tri-state (-1 error/0/1).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_binkcmp(
    p: *mut std::ffi::c_void,
    op1: i64,
    op2: i64,
    a: i64,
    b: i64,
    k: i64,
) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.fused_binkcmp_pub(op1 as u8, op2 as u8, a as usize, b as usize, k) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            -1
        }
    }
}

/// v3.5.32 (Tier-1): variante KK del super-opcode de 6 — `b` es CONSTANTE
/// (a op1 b_const) op2 k_const. NO reutiliza el shim KC (ahí `b` es nombre).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_fused_binkkcmp(
    p: *mut std::ffi::c_void,
    op1: i64,
    op2: i64,
    a: i64,
    b: i64,
    k: i64,
) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.fused_binkkcmp_pub(op1 as u8, op2 as u8, a as usize, b, k) {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            -1
        }
    }
}

/// v3.5.31: push de un Value::Bool (epílogo PushBool→Ret del Tier-2).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_push_bool(p: *mut std::ffi::c_void, b: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.push_pub(crate::value::Value::Bool(b != 0));
    0
}

// ──────────────────────────────────────────────────────────────────────
// v3.5.31 (Tier-2): helpers de bajo nivel para el bucle nativo.
// ──────────────────────────────────────────────────────────────────────

/// Puntero base al `flat` de la VM (estable durante el bucle: el cuerpo
/// elegible no asigna slots → el Vec no se realoca).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_flat_ptr(p: *mut std::ffi::c_void) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.flat_ptr_pub() as i64
}

/// ¿El slot contiene Value::Int? (guarda de tipos del Tier-2).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_slot_is_int(p: *mut std::ffi::c_void, slot: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    if vm.flat_slot_is_int_pub(slot as usize) {
        1
    } else {
        0
    }
}

/// Resuelve (o asigna) el slot de un nombre — mirror de StoreLocal.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_resolve_slot(p: *mut std::ffi::c_void, name_idx: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.resolve_slot_pub(name_idx as usize) {
        Ok(s) => s as i64,
        Err(e) => {
            vm.set_jit_error(e);
            -1
        }
    }
}

/// Resuelve destino de escritura (mirror de do_store_by_idx): busca en todos
/// los scopes; si falta, asigna en el superior.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_resolve_store(p: *mut std::ffi::c_void, name_idx: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    match vm.resolve_store_slot_pub(name_idx as usize) {
        Ok(s) => s as i64,
        Err(e) => {
            vm.set_jit_error(e);
            -1
        }
    }
}

/// Lookup SIN asignar: slot existente o -1 (→ bail-out al intérprete).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_slot_lookup(p: *mut std::ffi::c_void, name_idx: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.lookup_slot_pub(name_idx as usize)
}

/// v3.5.31 (Tier-2): pop Int de la pila de valores. i64::MIN si el tope no
/// es Int (el JIT hace bail-out → intérprete; seguro, solo desopta ese camino).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_pop_int(p: *mut std::ffi::c_void) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.pop_int_pub()
}

/// Push de un Value::Int (epílogo: Load→push→Ret).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn lj_push_int(p: *mut std::ffi::c_void, v: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    vm.push_pub(crate::value::Value::Int(v));
    0
}

// ──────────────────────────────────────────────────────────────────────

// ══════════════════════════════════════════════════════════════════════
// v3.5.39: INLINING de callees simples (Tier-2)
// ══════════════════════════════════════════════════════════════════════

/// Análisis de promoción para un callee candidato a inlining: mismas
/// reglas que el Tier-2 (params o locales sembradas con Push+StoreLocal,
/// leídas/definidas SOLO por ops Fused), pero sin depender de los sets
/// del pre-scan del caller.
fn analyze_promotion_inline(
    bc: &Bytecode,
    cs: usize,
    ce: usize,
    params: &BTreeSet<usize>,
) -> (Vec<usize>, HashMap<usize, Vec<usize>>) {
    let instrs = &bc.instructions;
    let mut prom_ok: BTreeSet<usize> = BTreeSet::new();
    let mut prom_bad: BTreeSet<usize> = BTreeSet::new();
    let mut prom_seed: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut j = cs;
    while j < ce {
        match &instrs[j] {
            Instruction::WithIdx(Opcode::PushInt, _)
            | Instruction::WithBool(Opcode::PushBool, _) => {
                if let Some(Instruction::WithIdx(Opcode::StoreLocal, nidx)) = instrs.get(j + 1) {
                    prom_seed.entry(*nidx).or_default().push(j);
                    prom_ok.insert(*nidx);
                    j += 1;
                }
            }
            Instruction::WithIdx(Opcode::StoreLocal, nidx) => {
                prom_bad.insert(*nidx);
            }
            Instruction::FusedBinK { a, d, .. } => {
                prom_ok.insert(*a);
                prom_ok.insert(*d);
            }
            Instruction::FusedBin { a, b, d, .. } => {
                prom_ok.insert(*a);
                prom_ok.insert(*b);
                prom_ok.insert(*d);
            }
            // v3.5.41 (bug #10): en las variantes de DECLARACIÓN el destino
            // se resuelve en el scope actual del frame → NO es promovible a
            // registro (el store local debe materializarse en el scope).
            Instruction::FusedBinKLocal { a, d, .. } => {
                prom_ok.insert(*a);
                prom_bad.insert(*d);
            }
            Instruction::FusedBinLocal { a, b, d, .. } => {
                prom_ok.insert(*a);
                prom_ok.insert(*b);
                prom_bad.insert(*d);
            }
            Instruction::FusedCmpKJmp { a, .. } => {
                prom_ok.insert(*a);
            }
            Instruction::FusedCmpJmp { a, b, .. } => {
                prom_ok.insert(*a);
                prom_ok.insert(*b);
            }
            Instruction::FusedBinCmpJmp { a, b, c, .. } => {
                prom_ok.insert(*a);
                prom_ok.insert(*b);
                prom_ok.insert(*c);
            }
            Instruction::FusedBinKCmpJmp { a, b, .. } => {
                prom_ok.insert(*a);
                prom_ok.insert(*b);
            }
            Instruction::FusedBinKKCmpJmp { a, .. } => {
                prom_ok.insert(*a);
            }
            _ => {}
        }
        j += 1;
    }
    let mut promoted = Vec::new();
    for n in prom_ok {
        if prom_bad.contains(&n) {
            continue;
        }
        let is_param = params.contains(&n);
        let seeded = prom_seed.contains_key(&n);
        if !is_param && !seeded {
            continue;
        }
        promoted.push(n);
        if promoted.len() >= 8 {
            break;
        }
    }
    (promoted, prom_seed)
}

/// ¿El callee es elegible para inlining? Devuelve
/// (start, end, promovidos, params, orden-de-params) o None.
/// Reglas: sin llamadas ni scopes; instrucciones del subconjunto
/// registrable; Rets con el valor YA en pila (push inmediato antes);
/// pila NEUTRA en cada salto interno (el Ret del intérprete trunca la
/// pila, el inline no — la profundidad relativa debe ser 0 en cada Jmp
/// y 1 en cada Ret); todas las lecturas/defs Fused promovibles.
#[allow(clippy::type_complexity)]
fn try_inline_plan(
    bc: &Bytecode,
    caller_idx: usize,
    callee_name: &str,
) -> Option<(usize, usize, Vec<usize>, BTreeSet<usize>, Vec<usize>)> {
    if crate::vm::builtin_name_set().contains(callee_name) {
        return None;
    }
    let callee_idx = bc.funcs.iter().position(|f| f.name == callee_name)?;
    if callee_idx == caller_idx {
        return None;
    }
    let (cs, ce) = body_range(bc, callee_idx);
    let size = ce.saturating_sub(cs);
    if size == 0 || size > 64 {
        return None;
    }
    let instrs = &bc.instructions;
    let mut param_order: Vec<usize> = Vec::new();
    let mut param_set: BTreeSet<usize> = BTreeSet::new();
    for p in &bc.funcs[callee_idx].params {
        let i = bc.names.iter().position(|n| n == p)?;
        param_set.insert(i);
        param_order.push(i);
    }
    if param_order.len() > 4 {
        return None;
    }
    // El cuerpo debe terminar en Ret.
    if !matches!(instrs.get(ce - 1), Some(Instruction::Simple(Opcode::Ret))) {
        return None;
    }
    // Elegibilidad + stack-neutralidad (fixpoint por posición). La
    // profundidad relativa al inicio del callee debe ser 0 en cada Jmp
    // (interno o Fused) y 0 antes de cada Ret (el push del valor de
    // retorno es la única contribución neta al stack del caller).
    let mut depth: HashMap<usize, i32> = HashMap::new();
    depth.insert(cs, 0);
    let mut changed = true;
    let mut neutral = true;
    while changed && neutral {
        changed = false;
        let mut j = cs;
        while j < ce && neutral {
            let d = match depth.get(&j) {
                Some(d) => *d,
                None => {
                    j += 1;
                    continue;
                }
            };
            let (succs, nd): (Vec<usize>, Option<i32>) = match &instrs[j] {
                Instruction::WithIdx(Opcode::PushInt, _)
                | Instruction::WithBool(Opcode::PushBool, _) => match instrs.get(j + 1) {
                    Some(Instruction::WithIdx(Opcode::StoreLocal, _)) => (vec![j + 2], Some(d)),
                    Some(Instruction::Simple(Opcode::Ret)) => {
                        if d != 0 {
                            neutral = false;
                        }
                        (vec![], None)
                    }
                    _ => (vec![j + 1], Some(d + 1)),
                },
                Instruction::WithIdx(Opcode::StoreLocal, _) | Instruction::Simple(Opcode::Ret) => {
                    neutral = false;
                    (vec![], None)
                }
                Instruction::FusedBin { .. }
                | Instruction::FusedBinK { .. }
                | Instruction::FusedBinKLocal { .. }
                | Instruction::FusedBinLocal { .. }
                | Instruction::WithIdx(Opcode::Nop, _) => (vec![j + 1], Some(d)),
                Instruction::FusedCmpKJmp { target, .. }
                | Instruction::FusedCmpJmp { target, .. }
                | Instruction::FusedBinCmpJmp { target, .. }
                | Instruction::FusedBinKCmpJmp { target, .. }
                | Instruction::FusedBinKKCmpJmp { target, .. } => {
                    let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                    if !(cs..ce).contains(&t) || d != 0 {
                        neutral = false;
                    }
                    (vec![j + 1, t], Some(d))
                }
                Instruction::WithIdx(Opcode::Jmp, idx) => {
                    let t = bc.nums.get(*idx).copied().unwrap_or(0.0) as usize;
                    if !(cs..ce).contains(&t) || d != 0 {
                        neutral = false;
                    }
                    (vec![t], Some(d))
                }
                _ => {
                    neutral = false;
                    (vec![], None)
                }
            };
            if let Some(nd) = nd {
                for t in succs {
                    match depth.get(&t) {
                        Some(&old) if old == nd => {}
                        Some(_) => neutral = false,
                        None => {
                            depth.insert(t, nd);
                            changed = true;
                        }
                    }
                }
            }
            j += 1;
        }
    }
    if !neutral {
        return None;
    }
    // Promoción: toda lectura/def Fused debe ser promovible.
    let (promoted, prom_seed) = analyze_promotion_inline(bc, cs, ce, &param_set);
    let mut j = cs;
    while j < ce {
        let ok = match &instrs[j] {
            Instruction::FusedBinK { a, d, .. } => promoted.contains(a) && promoted.contains(d),
            Instruction::FusedBin { a, b, d, .. } => {
                promoted.contains(a) && promoted.contains(b) && promoted.contains(d)
            }
            // v3.5.41 (bug #10): el store local no puede traducirse a
            // registro (escribe en el scope del frame) → fuera del inline.
            Instruction::FusedBinKLocal { .. } | Instruction::FusedBinLocal { .. } => false,
            Instruction::FusedCmpKJmp { a, .. } => promoted.contains(a),
            Instruction::FusedCmpJmp { a, b, .. } => promoted.contains(a) && promoted.contains(b),
            Instruction::FusedBinCmpJmp { a, b, c, .. } => {
                promoted.contains(a) && promoted.contains(b) && promoted.contains(c)
            }
            Instruction::FusedBinKCmpJmp { a, b, .. } => {
                promoted.contains(a) && promoted.contains(b)
            }
            Instruction::FusedBinKKCmpJmp { a, .. } => promoted.contains(a),
            _ => true,
        };
        if !ok {
            return None;
        }
        j += 1;
    }
    // DOMINANCIA: cada uso Fused de un nombre promovido NO-parámetro debe
    // estar dominado por algún seed (los early-outs antes de la semilla
    // no rompen la dominancia del bucle principal).
    let mut leaders: BTreeSet<usize> = BTreeSet::new();
    leaders.insert(cs);
    {
        let mut j = cs;
        while j < ce {
            let t = match &instrs[j] {
                Instruction::WithIdx(Opcode::Jmp, idx) => {
                    Some(bc.nums.get(*idx).copied().unwrap_or(0.0) as usize)
                }
                Instruction::FusedCmpKJmp { target, .. }
                | Instruction::FusedCmpJmp { target, .. }
                | Instruction::FusedBinCmpJmp { target, .. }
                | Instruction::FusedBinKCmpJmp { target, .. }
                | Instruction::FusedBinKKCmpJmp { target, .. } => {
                    Some(bc.nums.get(*target).copied().unwrap_or(0.0) as usize)
                }
                _ => None,
            };
            if let Some(t) = t {
                leaders.insert(t);
                leaders.insert(j + 1);
            }
            j += 1;
        }
    }
    let leader_list: Vec<usize> = leaders.iter().copied().collect();
    let idx_of: HashMap<usize, usize> = leader_list
        .iter()
        .enumerate()
        .map(|(i, l)| (*l, i))
        .collect();
    let n_blocks = leader_list.len();
    let mut succs: Vec<Vec<usize>> = vec![Vec::new(); n_blocks];
    for bi in 0..n_blocks {
        let nl = if bi + 1 < n_blocks {
            leader_list[bi + 1]
        } else {
            ce
        };
        let last = nl - 1;
        match &instrs[last] {
            Instruction::Simple(Opcode::Ret) => {}
            Instruction::WithIdx(Opcode::Jmp, idx) => {
                let t = bc.nums.get(*idx).copied().unwrap_or(0.0) as usize;
                succs[bi].push(idx_of[&t]);
            }
            Instruction::FusedCmpKJmp { target, .. }
            | Instruction::FusedCmpJmp { target, .. }
            | Instruction::FusedBinCmpJmp { target, .. }
            | Instruction::FusedBinKCmpJmp { target, .. }
            | Instruction::FusedBinKKCmpJmp { target, .. } => {
                let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                succs[bi].push(bi + 1);
                succs[bi].push(idx_of[&t]);
            }
            _ => succs[bi].push(bi + 1),
        }
    }
    let mut preds: Vec<Vec<usize>> = vec![Vec::new(); n_blocks];
    for (bi, ss) in succs.iter().enumerate() {
        for t in ss {
            preds[*t].push(bi);
        }
    }
    let mut dom: Vec<BTreeSet<usize>> = vec![(0..n_blocks).collect(); n_blocks];
    dom[0] = BTreeSet::from([0usize]);
    let mut changed = true;
    while changed {
        changed = false;
        for b in 1..n_blocks {
            let mut inter: Option<BTreeSet<usize>> = None;
            for p in &preds[b] {
                inter = Some(match inter {
                    None => dom[*p].clone(),
                    Some(prev) => prev.intersection(&dom[*p]).copied().collect(),
                });
            }
            let mut nd = inter.unwrap_or_default();
            nd.insert(b);
            if nd != dom[b] {
                dom[b] = nd;
                changed = true;
            }
        }
    }
    let block_of = |pos: usize| -> Option<usize> {
        leaders
            .range(..=pos)
            .next_back()
            .and_then(|l| idx_of.get(l))
            .copied()
    };
    let mut j = cs;
    while j < ce {
        let reads: Vec<usize> = match &instrs[j] {
            Instruction::FusedBinK { a, .. } => vec![*a],
            Instruction::FusedBin { a, b, .. } => vec![*a, *b],
            Instruction::FusedBinKLocal { a, .. } => vec![*a],
            Instruction::FusedBinLocal { a, b, .. } => vec![*a, *b],
            Instruction::FusedCmpKJmp { a, .. } => vec![*a],
            Instruction::FusedCmpJmp { a, b, .. } => vec![*a, *b],
            Instruction::FusedBinCmpJmp { a, b, c, .. } => vec![*a, *b, *c],
            Instruction::FusedBinKCmpJmp { a, b, .. } => vec![*a, *b],
            Instruction::FusedBinKKCmpJmp { a, .. } => vec![*a],
            _ => Vec::new(),
        };
        for n in reads {
            if !promoted.contains(&n) || param_set.contains(&n) {
                continue;
            }
            let use_b = block_of(j)?;
            let ok = prom_seed.get(&n).is_some_and(|seeds| {
                seeds.iter().any(|s| {
                    block_of(*s)
                        .map(|sb| dom[use_b].contains(&sb))
                        .unwrap_or(false)
                })
            });
            if !ok {
                return None;
            }
        }
        j += 1;
    }
    Some((cs, ce, promoted, param_set, param_order))
}

/// Compila el cuerpo del callee INLINE (sin slots: locales en registros
/// SSA). Los argumentos ya fueron extraídos de la pila como Values del
/// caller; cada bloque lleva params [args…, promovidos…]; los Ret saltan
/// a `cont_block` pasando los registros del caller (el valor de retorno
/// ya quedó en la pila de la VM).
#[allow(clippy::too_many_arguments)]
fn emit_inline_body(
    builder: &mut FunctionBuilder,
    bc: &Bytecode,
    cs: usize,
    ce: usize,
    cprom: &[usize],
    param_order: &[usize],
    arg_vals: &[Value],
    caller_args: &[BlockArg],
    cont_block: Block,
    bail_block: Block,
    err_block: Block,
    i64t: Type,
    vm_ptr: Value,
    r_pushint: FuncRef,
    r_pushbool: FuncRef,
) -> Result<(), String> {
    let instrs = &bc.instructions;
    // Líderes (mismo criterio que el walker Tier-2; solo Jmp/Fused*Jmp).
    let mut leaders: BTreeSet<usize> = BTreeSet::new();
    leaders.insert(cs);
    let mut j = cs;
    while j < ce {
        let t = match &instrs[j] {
            Instruction::WithIdx(Opcode::Jmp, idx) => {
                Some(bc.nums.get(*idx).copied().unwrap_or(0.0) as usize)
            }
            Instruction::FusedCmpKJmp { target, .. }
            | Instruction::FusedCmpJmp { target, .. }
            | Instruction::FusedBinCmpJmp { target, .. }
            | Instruction::FusedBinKCmpJmp { target, .. }
            | Instruction::FusedBinKKCmpJmp { target, .. } => {
                Some(bc.nums.get(*target).copied().unwrap_or(0.0) as usize)
            }
            _ => None,
        };
        if let Some(t) = t {
            leaders.insert(t);
            leaders.insert(j + 1);
        }
        j += 1;
    }
    let nargs = arg_vals.len();
    let mut blocks: HashMap<usize, Block> = HashMap::new();
    for l in &leaders {
        let b = builder.create_block();
        for _ in 0..nargs {
            builder.append_block_param(b, i64t);
        }
        for _ in cprom {
            builder.append_block_param(b, i64t);
        }
        blocks.insert(*l, b);
    }
    let mut regs: HashMap<usize, Value> = HashMap::new();
    let mut arg_cur: Vec<Value> = arg_vals.to_vec();
    macro_rules! iargs {
        () => {{
            arg_cur
                .iter()
                .copied()
                .map(BlockArg::Value)
                .chain(cprom.iter().map(|n| BlockArg::Value(regs[n])))
                .collect::<Vec<BlockArg>>()
        }};
    }
    let init: Vec<BlockArg> = arg_vals
        .iter()
        .copied()
        .map(BlockArg::Value)
        .chain(
            cprom
                .iter()
                .map(|_| BlockArg::Value(builder.ins().iconst(i64t, 0))),
        )
        .collect();
    builder.ins().jump(blocks[&cs], &init);

    let mut dead = true;
    let mut pos = cs;
    while pos < ce {
        if leaders.contains(&pos) {
            if !dead {
                let b = blocks[&pos];
                let args = iargs!();
                builder.ins().jump(b, &args);
            }
            dead = false;
            builder.switch_to_block(blocks[&pos]);
            builder.ensure_inserted_block();
            let ps = builder.block_params(blocks[&pos]);
            arg_cur = ps[..nargs].to_vec();
            regs.clear();
            for (n, p) in cprom.iter().zip(ps[nargs..].iter()) {
                regs.insert(*n, *p);
            }
            for (pn, pa) in param_order.iter().zip(arg_cur.iter()) {
                regs.insert(*pn, *pa);
            }
        }
        // Tras un terminador (Ret/Jmp/brif) el resto del bloque queda
        // INALCANZABLE: los Jmp muertos que el compilador deja como
        // continuación de los `si` no deben emitirse en el bloque ya
        // terminado (doble terminator → verifier). Se saltan hasta el
        // siguiente líder.
        if dead && !leaders.contains(&pos) {
            pos += 1;
            continue;
        }
        match &instrs[pos] {
            Instruction::WithIdx(Opcode::PushInt, idx) => {
                let kv = builder
                    .ins()
                    .iconst(i64t, bc.ints.get(*idx).copied().unwrap_or(0));
                if let Some(Instruction::WithIdx(Opcode::StoreLocal, nidx)) = instrs.get(pos + 1) {
                    if cprom.contains(nidx) {
                        regs.insert(*nidx, kv);
                    }
                    pos += 1;
                } else {
                    let call = builder.ins().call(r_pushint, &[vm_ptr, kv]);
                    let r = builder.inst_results(call)[0];
                    let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                    let c = builder.create_block();
                    builder.ins().brif(bad, err_block, &[], c, &[]);
                    builder.switch_to_block(c);
                    builder.ensure_inserted_block();
                }
            }
            Instruction::WithBool(Opcode::PushBool, b) => {
                if let Some(Instruction::WithIdx(Opcode::StoreLocal, nidx)) = instrs.get(pos + 1) {
                    let kv = builder.ins().iconst(i64t, if *b { 1 } else { 0 });
                    if cprom.contains(nidx) {
                        regs.insert(*nidx, kv);
                    }
                    pos += 1;
                } else {
                    let bv = builder.ins().iconst(i64t, if *b { 1 } else { 0 });
                    let call = builder.ins().call(r_pushbool, &[vm_ptr, bv]);
                    let r = builder.inst_results(call)[0];
                    let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                    let c = builder.create_block();
                    builder.ins().brif(bad, err_block, &[], c, &[]);
                    builder.switch_to_block(c);
                    builder.ensure_inserted_block();
                }
            }
            Instruction::FusedCmpKJmp { op, a, k, target } => {
                let av = regs[a];
                let kv = builder.ins().iconst(i64t, *k);
                let cond = match op {
                    7 => builder.ins().icmp(IntCC::Equal, av, kv),
                    8 => builder.ins().icmp(IntCC::NotEqual, av, kv),
                    9 => builder.ins().icmp(IntCC::SignedLessThan, av, kv),
                    10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, av, kv),
                    11 => builder.ins().icmp(IntCC::SignedGreaterThan, av, kv),
                    _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, av, kv),
                };
                let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                let tb = blocks[&t];
                let fall = blocks[&(pos + 1)];
                let args = iargs!();
                builder.ins().brif(cond, fall, &args, tb, &args);
                dead = true;
            }
            Instruction::FusedCmpJmp { op, a, b, target } => {
                let av = regs[a];
                let bv = regs[b];
                let cond = match op {
                    7 => builder.ins().icmp(IntCC::Equal, av, bv),
                    8 => builder.ins().icmp(IntCC::NotEqual, av, bv),
                    9 => builder.ins().icmp(IntCC::SignedLessThan, av, bv),
                    10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, av, bv),
                    11 => builder.ins().icmp(IntCC::SignedGreaterThan, av, bv),
                    _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, av, bv),
                };
                let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                let tb = blocks[&t];
                let fall = blocks[&(pos + 1)];
                let args = iargs!();
                builder.ins().brif(cond, fall, &args, tb, &args);
                dead = true;
            }
            Instruction::FusedBin { op, a, b, d } => {
                let av = regs[a];
                let bv = regs[b];
                let res = match op {
                    1 => builder.ins().iadd(av, bv),
                    3 => builder.ins().isub(av, bv),
                    _ => builder.ins().imul(av, bv),
                };
                regs.insert(*d, res);
            }
            Instruction::FusedBinK { op, a, k, d } => {
                let av = regs[a];
                let kv = builder.ins().iconst(i64t, *k);
                let res = match op {
                    1 => builder.ins().iadd(av, kv),
                    3 => builder.ins().isub(av, kv),
                    _ => builder.ins().imul(av, kv),
                };
                regs.insert(*d, res);
            }
            Instruction::FusedBinCmpJmp {
                op1,
                op2,
                a,
                b,
                c,
                target,
            } => {
                let av = regs[a];
                let bv = regs[b];
                let t = match op1 {
                    1 => builder.ins().iadd(av, bv),
                    3 => builder.ins().isub(av, bv),
                    4 => builder.ins().imul(av, bv),
                    5 | 6 => {
                        let zero = builder.ins().iconst(i64t, 0);
                        let ok = builder.ins().icmp(IntCC::SignedGreaterThan, bv, zero);
                        let cont = builder.create_block();
                        builder.ins().brif(ok, cont, &[], bail_block, &[]);
                        builder.switch_to_block(cont);
                        builder.ensure_inserted_block();
                        if *op1 == 5 {
                            builder.ins().sdiv(av, bv)
                        } else {
                            builder.ins().srem(av, bv)
                        }
                    }
                    _ => return Err("op1 inesperado en FusedBinCmpJmp inline".into()),
                };
                let cv = regs[c];
                let cond = match op2 {
                    7 => builder.ins().icmp(IntCC::Equal, t, cv),
                    8 => builder.ins().icmp(IntCC::NotEqual, t, cv),
                    9 => builder.ins().icmp(IntCC::SignedLessThan, t, cv),
                    10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, t, cv),
                    11 => builder.ins().icmp(IntCC::SignedGreaterThan, t, cv),
                    _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, t, cv),
                };
                let tp = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                let tb = blocks[&tp];
                let fall = blocks[&(pos + 1)];
                let args = iargs!();
                builder.ins().brif(cond, fall, &args, tb, &args);
                dead = true;
            }
            Instruction::FusedBinKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                target,
            } => {
                let av = regs[a];
                let bv = regs[b];
                let t = match op1 {
                    1 => builder.ins().iadd(av, bv),
                    3 => builder.ins().isub(av, bv),
                    4 => builder.ins().imul(av, bv),
                    5 | 6 => {
                        let zero = builder.ins().iconst(i64t, 0);
                        let ok = builder.ins().icmp(IntCC::SignedGreaterThan, bv, zero);
                        let cont = builder.create_block();
                        builder.ins().brif(ok, cont, &[], bail_block, &[]);
                        builder.switch_to_block(cont);
                        builder.ensure_inserted_block();
                        if *op1 == 5 {
                            builder.ins().sdiv(av, bv)
                        } else {
                            builder.ins().srem(av, bv)
                        }
                    }
                    _ => return Err("op1 inesperado en FusedBinKCmpJmp inline".into()),
                };
                let kv = builder.ins().iconst(i64t, *k);
                let cond = match op2 {
                    7 => builder.ins().icmp(IntCC::Equal, t, kv),
                    8 => builder.ins().icmp(IntCC::NotEqual, t, kv),
                    9 => builder.ins().icmp(IntCC::SignedLessThan, t, kv),
                    10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, t, kv),
                    11 => builder.ins().icmp(IntCC::SignedGreaterThan, t, kv),
                    _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, t, kv),
                };
                let tp = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                let tb = blocks[&tp];
                let fall = blocks[&(pos + 1)];
                let args = iargs!();
                builder.ins().brif(cond, fall, &args, tb, &args);
                dead = true;
            }
            Instruction::FusedBinKKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                target,
            } => {
                let av = regs[a];
                let bv = builder.ins().iconst(i64t, *b);
                let t = match op1 {
                    1 => builder.ins().iadd(av, bv),
                    3 => builder.ins().isub(av, bv),
                    4 => builder.ins().imul(av, bv),
                    5 | 6 => {
                        let zero = builder.ins().iconst(i64t, 0);
                        let ok = builder.ins().icmp(IntCC::SignedGreaterThan, bv, zero);
                        let cont = builder.create_block();
                        builder.ins().brif(ok, cont, &[], bail_block, &[]);
                        builder.switch_to_block(cont);
                        builder.ensure_inserted_block();
                        if *op1 == 5 {
                            builder.ins().sdiv(av, bv)
                        } else {
                            builder.ins().srem(av, bv)
                        }
                    }
                    _ => return Err("op1 inesperado en FusedBinKKCmpJmp inline".into()),
                };
                let kv = builder.ins().iconst(i64t, *k);
                let cond = match op2 {
                    7 => builder.ins().icmp(IntCC::Equal, t, kv),
                    8 => builder.ins().icmp(IntCC::NotEqual, t, kv),
                    9 => builder.ins().icmp(IntCC::SignedLessThan, t, kv),
                    10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, t, kv),
                    11 => builder.ins().icmp(IntCC::SignedGreaterThan, t, kv),
                    _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, t, kv),
                };
                let tp = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                let tb = blocks[&tp];
                let fall = blocks[&(pos + 1)];
                let args = iargs!();
                builder.ins().brif(cond, fall, &args, tb, &args);
                dead = true;
            }
            Instruction::WithIdx(Opcode::Jmp, idx) => {
                let t = bc.nums.get(*idx).copied().unwrap_or(0.0) as usize;
                let tb = blocks[&t];
                let args = iargs!();
                builder.ins().jump(tb, &args);
                dead = true;
            }
            Instruction::Simple(Opcode::Ret) => {
                builder.ins().jump(cont_block, caller_args);
                dead = true;
            }
            Instruction::WithIdx(Opcode::Nop, _) => {}
            _ => return Err("instrucción inesperada en cuerpo inline".into()),
        }
        pos += 1;
    }
    if !dead {
        // Terminador de seguridad (inalcanzable: el cuerpo termina en Ret).
        builder.ins().jump(cont_block, caller_args);
    }
    Ok(())
}

// Motor JIT
// ──────────────────────────────────────────────────────────────────────

const MAX_JIT_BODY: usize = 60_000;

/// v3.5.31 (Tier-2): layout en memoria del enum `Value` (no repr(C)), medido
/// en tiempo de ejecución y validado con round-trip antes de usarse.
struct ValueLayout {
    size: usize,
    disc_off: usize,
    disc_int: u8,
    payload_off: usize,
}

impl ValueLayout {
    fn probe() -> Option<ValueLayout> {
        let magic: u64 = 0x0102_0304_0506_0708;
        let magic2: u64 = 0x0f0e_0d0c_0b0a_0908;
        let v = crate::value::Value::Int(magic as i64);
        let vb = &v as *const crate::value::Value as *const u8;
        let void = crate::value::Value::Void;
        let voidb = &void as *const crate::value::Value as *const u8;
        let size = std::mem::size_of::<crate::value::Value>();
        // 1) byte del discriminante: primer byte que difiere de Void.
        //    disc_int = byte de INT (es el que se escribe al construir).
        let mut disc_off = usize::MAX;
        let mut disc_int = 0u8;
        for i in 0..size {
            let (a, b) = unsafe { (*vb.add(i), *voidb.add(i)) };
            if a != b {
                disc_off = i;
                disc_int = a;
                break;
            }
        }
        // 2) offset del payload i64: los 8 bytes del magic en little-endian.
        let be = magic.to_le_bytes();
        let mut payload_off = usize::MAX;
        for i in 0..(size - 7) {
            let slice = unsafe { std::slice::from_raw_parts(vb.add(i), 8) };
            if slice == be {
                payload_off = i;
                break;
            }
        }
        let layout = ValueLayout {
            size,
            disc_off,
            disc_int,
            payload_off,
        };
        // 3) validación semántica end-to-end: partiendo de los bytes REALES
        // de un Int, reescribir disc y payload con magic2 y LEERLO de vuelta
        // como Value con las reglas del compilador — debe ser Int(magic2).
        // (El padding es inestable entre instancias; la comparación de
        // valores es por campos, no por bytes.)
        if disc_off >= size || payload_off == usize::MAX || payload_off + 8 > size {
            return None;
        }
        let void_disc = unsafe { *voidb.add(disc_off) };
        if void_disc == disc_int {
            return None; // el byte no discrimina Int vs Void → posición errónea
        }
        let mut buf = vec![0u8; size];
        buf.copy_from_slice(unsafe { std::slice::from_raw_parts(vb, size) });
        buf[disc_off] = disc_int;
        buf[payload_off..payload_off + 8].copy_from_slice(&magic2.to_le_bytes());
        let readback = unsafe { std::ptr::read(buf.as_ptr() as *const crate::value::Value) };
        if matches!(readback, crate::value::Value::Int(x) if x == magic2 as i64) {
            Some(layout)
        } else {
            None
        }
    }
}

pub struct VmJit {
    fbc: cranelift::frontend::FunctionBuilderContext,
    module: JITModule,
    /// v3.5.31: funciones compiladas, indexadas por func_idx (O(1) sin hash
    /// en el camino caliente de cada llamada).
    fns: Vec<Option<JitFn>>,
    /// v3.5.31: marcador de no-reintento por func_idx.
    failed: Vec<bool>,
    counter: usize,
    /// v3.5.31 (Tier-2): None si el round-trip del layout falló (→ solo Tier-1).
    tier2: Option<ValueLayout>,
}

impl VmJit {
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| e.to_string())?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| e.to_string())?;
        flag_builder
            .set("opt_level", "speed")
            .map_err(|e| e.to_string())?;
        let isa_builder = cranelift_native::builder().map_err(|e| e.to_string())?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| e.to_string())?;
        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        // Registrar las direcciones de los helpers (portable: no depende de
        // la exportación de símbolos del binario — importante en Windows).
        builder.symbol("lj_simple", lj_simple as *const u8);
        builder.symbol("lj_with_num", lj_with_num as *const u8);
        builder.symbol("lj_with_bool", lj_with_bool as *const u8);
        builder.symbol("lj_with_idx", lj_with_idx as *const u8);
        builder.symbol("lj_with_str", lj_with_str as *const u8);
        builder.symbol("lj_call", lj_call as *const u8);
        builder.symbol("lj_call_fast", lj_call_fast as *const u8);
        builder.symbol("lj_concat", lj_concat as *const u8);
        builder.symbol("lj_probe_int", lj_probe_int as *const u8);
        builder.symbol("lj_truth", lj_truth as *const u8);
        builder.symbol("lj_ret", lj_ret as *const u8);
        // v3.5.31: super-opcodes delegados (bucle caliente sin dispatch).
        builder.symbol("lj_fused_bink", lj_fused_bink as *const u8);
        builder.symbol("lj_fused_bin", lj_fused_bin as *const u8);
        builder.symbol("lj_fused_bink_local", lj_fused_bink_local as *const u8);
        builder.symbol("lj_fused_bin_local", lj_fused_bin_local as *const u8);
        builder.symbol("lj_fused_cmpk", lj_fused_cmpk as *const u8);
        builder.symbol("lj_fused_cmp", lj_fused_cmp as *const u8);
        // v3.5.31 (Tier-2): bucle nativo con aritmética Int directa.
        builder.symbol("lj_flat_ptr", lj_flat_ptr as *const u8);
        builder.symbol("lj_slot_is_int", lj_slot_is_int as *const u8);
        builder.symbol("lj_resolve_slot", lj_resolve_slot as *const u8);
        builder.symbol("lj_resolve_store", lj_resolve_store as *const u8);
        builder.symbol("lj_slot_lookup", lj_slot_lookup as *const u8);
        builder.symbol("lj_push_int", lj_push_int as *const u8);
        builder.symbol("lj_push_bool", lj_push_bool as *const u8);
        builder.symbol("lj_pop_int", lj_pop_int as *const u8);
        builder.symbol("lj_fused_bincmp", lj_fused_bincmp as *const u8);
        builder.symbol("lj_fused_binkcmp", lj_fused_binkcmp as *const u8);
        builder.symbol("lj_fused_binkkcmp", lj_fused_binkkcmp as *const u8);
        let module = JITModule::new(builder);
        Ok(Self {
            fbc: cranelift::frontend::FunctionBuilderContext::new(),
            module,
            fns: Vec::new(),
            failed: Vec::new(),
            counter: 0,
            tier2: ValueLayout::probe(),
        })
    }

    pub fn get(&self, func_idx: usize) -> Option<JitFn> {
        self.fns.get(func_idx).copied().flatten()
    }

    pub fn should_try(&self, func_idx: usize) -> bool {
        self.fns.get(func_idx).is_none_or(|f| f.is_none())
            && !self.failed.get(func_idx).copied().unwrap_or(false)
    }

    /// Intenta compilar la función `func_idx`. Si no pertenece al subconjunto
    /// seguro, la marca como fallida y no se reintenta.
    pub fn try_compile(&mut self, bc: &Bytecode, func_idx: usize) {
        if self.fns.len() < bc.funcs.len() {
            self.fns.resize(bc.funcs.len(), None);
            self.failed.resize(bc.funcs.len(), false);
        }
        if !self.should_try(func_idx) {
            return;
        }
        // v3.5.34 (Tier-R): recursión auto-nativa en registros — funciones
        // auto-recursivas puras (fib) NO pagan frame ni shims por nivel.
        if self.tier2.is_some() {
            if let Ok(f) = self.try_compile_recursive(bc, func_idx) {
                self.fns[func_idx] = Some(f);
                if std::env::var_os("LUMEN_JIT_LOG").is_some() {
                    eprintln!(
                        "[jit] ✅ recursión nativa (registros): '{}' ({} instrs)",
                        bc.funcs[func_idx].name,
                        self.body_len(bc, func_idx)
                    );
                }
                return;
            }
        }
        // v3.5.31 (Tier-2): bucle Int puro → código nativo directo; si el
        // cuerpo no es elegible, cae al Tier-1 (delegación por shims).
        if self.tier2.is_some() {
            match self.compile_tier2(bc, func_idx) {
                Ok(f) => {
                    self.fns[func_idx] = Some(f);
                    if std::env::var_os("LUMEN_JIT_LOG").is_some() {
                        eprintln!(
                            "[jit] ✅ Tier-2 bucle nativo: '{}' ({} instrs)",
                            bc.funcs[func_idx].name,
                            self.body_len(bc, func_idx)
                        );
                    }
                    return;
                }
                Err(why) => {
                    if std::env::var_os("LUMEN_JIT_LOG").is_some() {
                        eprintln!(
                            "[jit] ℹ️ '{}' no elegible para Tier-2 ({}): intentando Tier-1",
                            bc.funcs[func_idx].name, why
                        );
                    }
                }
            }
        }
        match self.compile(bc, func_idx) {
            Ok(f) => {
                self.fns[func_idx] = Some(f);
                if std::env::var_os("LUMEN_JIT_LOG").is_some() {
                    eprintln!(
                        "[jit] ✅ compilada nativamente: '{}' ({} instrs)",
                        bc.funcs[func_idx].name,
                        self.body_len(bc, func_idx)
                    );
                }
            }
            Err(why) => {
                if std::env::var_os("LUMEN_JIT_LOG").is_some() {
                    eprintln!(
                        "[jit] ❌ no compilable '{}': {}",
                        bc.funcs[func_idx].name, why
                    );
                }
                self.failed[func_idx] = true;
            }
        }
    }

    /// v3.5.31 (Tier-2): invalida una compilación tras un bail-out (la guarda
    /// de tipos falló) — no se reintenta para esa función.
    pub fn invalidate(&mut self, func_idx: usize) {
        if let Some(slot) = self.fns.get_mut(func_idx) {
            *slot = None;
        }
        if func_idx < self.failed.len() {
            self.failed[func_idx] = true;
        }
    }

    fn body_len(&self, bc: &Bytecode, func_idx: usize) -> usize {
        let (s, e) = body_range(bc, func_idx);
        e.saturating_sub(s)
    }

    fn compile(&mut self, bc: &Bytecode, func_idx: usize) -> Result<JitFn, String> {
        let (start, end) = body_range(bc, func_idx);
        if end <= start || end - start > MAX_JIT_BODY {
            return Err("cuerpo fuera de límites".into());
        }
        let instrs = &bc.instructions;

        // ── Pre-scan: condiciones de salida del subconjunto seguro ──
        let mut i = start;
        while i < end {
            match &instrs[i] {
                Instruction::Simple(Opcode::Halt)
                | Instruction::Simple(Opcode::PushHandler)
                | Instruction::Simple(Opcode::PopHandler) => {
                    return Err("opcode no soportado por el JIT".into())
                }
                // v3.5.31: los super-opcodes SON soportados (helpers lj_fused_*);
                // solo se valida que sus saltos queden dentro del cuerpo.
                Instruction::FusedCmpKJmp { target, .. }
                | Instruction::FusedCmpJmp { target, .. }
                | Instruction::FusedBinCmpJmp { target, .. }
                | Instruction::FusedBinKCmpJmp { target, .. }
                | Instruction::FusedBinKKCmpJmp { target, .. } => {
                    let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("salto fuera del cuerpo (fused)".into());
                    }
                }
                Instruction::FusedBinK { .. } | Instruction::FusedBin { .. } => {}
                Instruction::FusedBinKLocal { .. } | Instruction::FusedBinLocal { .. } => {}
                Instruction::WithIdx(Opcode::Call, _) => {
                    // El par Call debe ir seguido de WithIdx(_, argc)
                    match instrs.get(i + 1) {
                        Some(Instruction::WithIdx(_, _)) => {}
                        _ => return Err("secuencia Call/argc malformada".into()),
                    }
                }
                _ => {}
            }
            i += 1;
        }

        // ── Líderes de bloque ──
        let mut leaders = BTreeSet::new();
        leaders.insert(start);
        // Resuelve el destino de un salto (Jmp/JmpIf pueden venir como
        // WithNum directo o WithIdx → pool de nums; los FusedCmp* saltan
        // vía `target` → pool de nums).
        let jmp_target = |ins: &Instruction| -> Option<usize> {
            match ins {
                Instruction::WithNum(Opcode::Jmp, n) | Instruction::WithNum(Opcode::JmpIf, n) => {
                    Some(*n as usize)
                }
                Instruction::WithIdx(Opcode::Jmp, idx)
                | Instruction::WithIdx(Opcode::JmpIf, idx) => {
                    Some(bc.nums.get(*idx).copied().unwrap_or(0.0) as usize)
                }
                _ => None,
            }
        };
        let is_jmp = |ins: &Instruction| -> bool {
            matches!(
                ins,
                Instruction::WithNum(Opcode::Jmp, _)
                    | Instruction::WithNum(Opcode::JmpIf, _)
                    | Instruction::WithIdx(Opcode::Jmp, _)
                    | Instruction::WithIdx(Opcode::JmpIf, _)
            )
        };
        // v3.5.31: los FusedCmp* también bifurcan → sus destinos y el
        // fallthrough son líderes de bloque.
        let fused_target = |ins: &Instruction| -> Option<usize> {
            match ins {
                Instruction::FusedCmpKJmp { target, .. }
                | Instruction::FusedCmpJmp { target, .. }
                | Instruction::FusedBinCmpJmp { target, .. }
                | Instruction::FusedBinKCmpJmp { target, .. }
                | Instruction::FusedBinKKCmpJmp { target, .. } => {
                    Some(bc.nums.get(*target).copied().unwrap_or(0.0) as usize)
                }
                _ => None,
            }
        };
        i = start;
        while i < end {
            if is_jmp(&instrs[i]) {
                let t = jmp_target(&instrs[i]).unwrap();
                if !(start..end).contains(&t) {
                    return Err("salto fuera del cuerpo".into());
                }
                leaders.insert(t);
                leaders.insert(i + 1);
            }
            if let Some(t) = fused_target(&instrs[i]) {
                leaders.insert(t);
                leaders.insert(i + 1);
            }
            if matches!(instrs[i], Instruction::Simple(Opcode::Ret)) {
                leaders.insert(i + 1);
            }
            i += 1;
        }

        // ── Firmas ──
        let i64t = types::I64;
        let f64t = types::F64;
        let mut sig_main = self.module.make_signature();
        sig_main.params.push(AbiParam::new(i64t));
        sig_main.returns.push(AbiParam::new(i64t));

        let mut sig_2 = self.module.make_signature(); // (vm, op, ip)
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.returns.push(AbiParam::new(i64t));

        let mut sig_wn = self.module.make_signature(); // (vm, op, f64, ip)
        sig_wn.params.push(AbiParam::new(i64t));
        sig_wn.params.push(AbiParam::new(i64t));
        sig_wn.params.push(AbiParam::new(f64t));
        sig_wn.params.push(AbiParam::new(i64t));
        sig_wn.returns.push(AbiParam::new(i64t));

        let mut sig_ws = self.module.make_signature(); // (vm, op, ptr, len, ip)
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.returns.push(AbiParam::new(i64t));

        let mut sig_call = self.module.make_signature(); // (vm, nidx, argc, ip)
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.returns.push(AbiParam::new(i64t));

        let mut sig_1 = self.module.make_signature(); // (vm)
        sig_1.params.push(AbiParam::new(i64t));
        sig_1.returns.push(AbiParam::new(i64t));

        let mut sig_3 = self.module.make_signature(); // (vm, op, idx, ip)
        sig_3.params.push(AbiParam::new(i64t));
        sig_3.params.push(AbiParam::new(i64t));
        sig_3.params.push(AbiParam::new(i64t));
        sig_3.params.push(AbiParam::new(i64t));
        sig_3.returns.push(AbiParam::new(i64t));

        // v3.5.31: helpers de super-opcodes.
        let mut sig_fbk = self.module.make_signature(); // (vm, op, a, k, d)
        for _ in 0..5 {
            sig_fbk.params.push(AbiParam::new(i64t));
        }
        sig_fbk.returns.push(AbiParam::new(i64t));
        let mut sig_fcmpk = self.module.make_signature(); // (vm, op, a, k)
        for _ in 0..4 {
            sig_fcmpk.params.push(AbiParam::new(i64t));
        }
        sig_fcmpk.returns.push(AbiParam::new(i64t));
        let mut sig_fbincmp = self.module.make_signature(); // (vm, op1, op2, a, b, c)
        for _ in 0..6 {
            sig_fbincmp.params.push(AbiParam::new(i64t));
        }
        sig_fbincmp.returns.push(AbiParam::new(i64t));
        let mut sig_fbinkcmp = self.module.make_signature(); // (vm, op1, op2, a, b, k)
        for _ in 0..6 {
            sig_fbinkcmp.params.push(AbiParam::new(i64t));
        }
        sig_fbinkcmp.returns.push(AbiParam::new(i64t));

        let decl = |module: &mut JITModule, name: &str, sig: &Signature| {
            module
                .declare_function(name, Linkage::Import, sig)
                .map_err(|e| e.to_string())
        };
        let f_simple = decl(&mut self.module, "lj_simple", &sig_2)?;
        let f_wn = decl(&mut self.module, "lj_with_num", &sig_wn)?;
        let f_wb = decl(&mut self.module, "lj_with_bool", &sig_2)?;
        let f_wi = decl(&mut self.module, "lj_with_idx", &sig_3)?;
        let f_ws = decl(&mut self.module, "lj_with_str", &sig_ws)?;
        let f_call = decl(&mut self.module, "lj_call", &sig_call)?;
        let f_call_fast = decl(&mut self.module, "lj_call_fast", &sig_call)?;
        let f_truth = decl(&mut self.module, "lj_truth", &sig_1)?;
        let f_ret = decl(&mut self.module, "lj_ret", &sig_1)?;
        let f_fbk = decl(&mut self.module, "lj_fused_bink", &sig_fbk)?;
        let f_fb = decl(&mut self.module, "lj_fused_bin", &sig_fbk)?;
        let f_fbkl = decl(&mut self.module, "lj_fused_bink_local", &sig_fbk)?;
        let f_fbl = decl(&mut self.module, "lj_fused_bin_local", &sig_fbk)?;
        let f_fcmpk = decl(&mut self.module, "lj_fused_cmpk", &sig_fcmpk)?;
        let f_fcmp = decl(&mut self.module, "lj_fused_cmp", &sig_fcmpk)?;
        let f_fbincmp = decl(&mut self.module, "lj_fused_bincmp", &sig_fbincmp)?;
        let f_fbinkcmp = decl(&mut self.module, "lj_fused_binkcmp", &sig_fbinkcmp)?;
        let f_fbinkkcmp = decl(&mut self.module, "lj_fused_binkkcmp", &sig_fbinkcmp)?;

        // ── Función destino ──
        self.counter += 1;
        let fname = format!("lumen_jit_{}_{}", func_idx, self.counter);
        let func_id = self
            .module
            .declare_function(&fname, Linkage::Export, &sig_main)
            .map_err(|e| e.to_string())?;

        let mut ctx = self.module.make_context();
        ctx.func.signature = sig_main.clone();
        ctx.func.name = cranelift::codegen::ir::UserFuncName::user(0, func_id.as_u32());

        {
            let mut builder =
                cranelift::frontend::FunctionBuilder::new(&mut ctx.func, &mut self.fbc);

            // IMPORTANTE: el entry real DEBE ser el primer bloque creado (Cranelift
            // exige que el bloque de entrada de la función sea el primero). Recibe el
            // vm ptr como parámetro y salta al inicio del cuerpo (así `start` puede
            // ser destino de saltos sin block-params).
            // v3.5.31: SIN ensure_inserted_block() aquí — marcaba el entry como
            // "Partial" (instrucciones sin terminador) y el switch a err_block
            // violaba el debug_assert de switch_to_block en builds de prueba.
            let entry = builder.create_block();
            builder.switch_to_block(entry);
            builder.append_block_param(entry, i64t); // vm ptr
            let vm_ptr = builder.block_params(entry)[0];

            let mut blocks: HashMap<usize, Block> = HashMap::new();
            for l in &leaders {
                let b = builder.create_block();
                blocks.insert(*l, b);
            }
            let err_block = builder.create_block();

            let r_simple = self.module.declare_func_in_func(f_simple, builder.func);
            let r_wn = self.module.declare_func_in_func(f_wn, builder.func);
            let r_wb = self.module.declare_func_in_func(f_wb, builder.func);
            let r_wi = self.module.declare_func_in_func(f_wi, builder.func);
            let r_ws = self.module.declare_func_in_func(f_ws, builder.func);
            let r_call = self.module.declare_func_in_func(f_call, builder.func);
            let r_call_fast = self.module.declare_func_in_func(f_call_fast, builder.func);
            let r_truth = self.module.declare_func_in_func(f_truth, builder.func);
            let r_ret = self.module.declare_func_in_func(f_ret, builder.func);
            let r_fbk = self.module.declare_func_in_func(f_fbk, builder.func);
            let r_fb = self.module.declare_func_in_func(f_fb, builder.func);
            let r_fbkl = self.module.declare_func_in_func(f_fbkl, builder.func);
            let r_fbl = self.module.declare_func_in_func(f_fbl, builder.func);
            let r_fcmpk = self.module.declare_func_in_func(f_fcmpk, builder.func);
            let r_fcmp = self.module.declare_func_in_func(f_fcmp, builder.func);
            let r_fbincmp = self.module.declare_func_in_func(f_fbincmp, builder.func);
            let r_fbinkcmp = self.module.declare_func_in_func(f_fbinkcmp, builder.func);
            let r_fbinkkcmp = self.module.declare_func_in_func(f_fbinkkcmp, builder.func);

            // Epílogo de error: return 1
            // Trampolín: entry salta al inicio del cuerpo. Esto APENDA entry
            // como PRIMER bloque del layout (el verifier exige que el entry
            // con sus block-params sea el primero) y lo deja "Filled", así el
            // switch a err_block no viola el debug_assert de switch_to_block
            // en builds de prueba (v3.5.31).
            builder.ensure_inserted_block();
            builder.ins().jump(blocks[&start], &[]);
            // Epílogo de error: return 1
            builder.switch_to_block(err_block);
            let one = builder.ins().iconst(i64t, 1);
            builder.ins().return_(&[one]);

            // Helper local: emite `r = call(...); si r != 0 → err_block`
            macro_rules! check {
                ($call:expr) => {{
                    let r = $call;
                    let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                    let next = builder.create_block();
                    builder.ins().brif(bad, err_block, &[], next, &[]);
                    builder.switch_to_block(next);
                    builder.ensure_inserted_block();
                }};
            }

            // Epílogo de retorno (Ret nativo): lj_ret + check + return 0
            macro_rules! ret_epilogue {
                () => {{
                    let call = builder.ins().call(r_ret, &[vm_ptr]);
                    let r = builder.inst_results(call)[0];
                    let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                    let okb = builder.create_block();
                    builder.ins().brif(bad, err_block, &[], okb, &[]);
                    builder.switch_to_block(okb);
                    let zero = builder.ins().iconst(i64t, 0);
                    builder.ins().return_(&[zero]);
                }};
            }

            let mut dead = true; // el trampolín ya terminó su bloque
            i = start;
            while i < end {
                if leaders.contains(&i) {
                    if !dead {
                        // caída desde el bloque anterior
                        let b = blocks[&i];
                        builder.ins().jump(b, &[]);
                    }
                    dead = false;
                    builder.switch_to_block(blocks[&i]);
                    builder.ensure_inserted_block();
                }
                if dead {
                    i += 1;
                    continue;
                }
                match &instrs[i] {
                    Instruction::Simple(Opcode::Ret) => {
                        ret_epilogue!();
                        dead = true;
                    }
                    Instruction::Simple(op) => {
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let ipv = builder.ins().iconst(i64t, (i + 1) as i64);
                        let call = builder.ins().call(r_simple, &[vm_ptr, opv, ipv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::WithNum(Opcode::Jmp, n) => {
                        let t = blocks[&(*n as usize)];
                        builder.ins().jump(t, &[]);
                        dead = true;
                    }
                    Instruction::WithNum(Opcode::JmpIf, n) => {
                        let call = builder.ins().call(r_truth, &[vm_ptr]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        let fall = blocks[&(i + 1)];
                        let target = blocks[&(*n as usize)];
                        // truthy → cae (i+1); falsy → salta al destino
                        builder.ins().brif(r, fall, &[], target, &[]);
                        dead = true;
                    }
                    Instruction::WithNum(op, n) => {
                        let nv = builder.ins().f64const(*n);
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let ipv = builder.ins().iconst(i64t, (i + 1) as i64);
                        let call = builder.ins().call(r_wn, &[vm_ptr, opv, nv, ipv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::WithBool(op, b) => {
                        let bv = builder.ins().iconst(i64t, if *b { 1 } else { 0 });
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let ipv = builder.ins().iconst(i64t, (i + 1) as i64);
                        let call = builder.ins().call(r_wb, &[vm_ptr, opv, bv, ipv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::WithStr(op, s) => {
                        let pv = builder.ins().iconst(i64t, s.as_ptr() as i64);
                        let lv = builder.ins().iconst(i64t, s.len() as i64);
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let ipv = builder.ins().iconst(i64t, (i + 1) as i64);
                        let call = builder.ins().call(r_ws, &[vm_ptr, opv, pv, lv, ipv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::WithIdx(Opcode::Jmp, tidx) => {
                        let t = bc.nums.get(*tidx).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        builder.ins().jump(tb, &[]);
                        dead = true;
                    }
                    Instruction::WithIdx(Opcode::JmpIf, tidx) => {
                        let call = builder.ins().call(r_truth, &[vm_ptr]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        let fall = blocks[&(i + 1)];
                        let t = bc.nums.get(*tidx).copied().unwrap_or(0.0) as usize;
                        let target = blocks[&t];
                        // truthy → cae (i+1); falsy → salta al destino
                        builder.ins().brif(r, fall, &[], target, &[]);
                        dead = true;
                    }
                    Instruction::WithIdx(Opcode::Call, nidx) => {
                        // instrs[i+1] garantizado por el pre-scan
                        let argc = match &instrs[i + 1] {
                            Instruction::WithIdx(_, aidx) => {
                                bc.nums.get(*aidx).copied().unwrap_or(0.0) as i64
                            }
                            _ => 0,
                        };
                        let nv = builder.ins().iconst(i64t, *nidx as i64);
                        let av = builder.ins().iconst(i64t, argc);
                        let ipv = builder.ins().iconst(i64t, (i + 1) as i64);
                        // v3.5.32: nombre NO builtin (set estático) → call
                        // rápido sin pre-filtro; builtin → ruta completa.
                        let is_builtin = crate::vm::builtin_name_set()
                            .contains(bc.names.get(*nidx).map(|s| s.as_str()).unwrap_or(""));
                        let f_use = if is_builtin { r_call } else { r_call_fast };
                        let call = builder.ins().call(f_use, &[vm_ptr, nv, av, ipv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                        i += 1; // consumir la instrucción argc
                    }
                    Instruction::WithIdx(op, idx) => {
                        let iv = builder.ins().iconst(i64t, *idx as i64);
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let ipv = builder.ins().iconst(i64t, (i + 1) as i64);
                        let call = builder.ins().call(r_wi, &[vm_ptr, opv, iv, ipv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    // v3.5.31: super-opcodes — delegan al MISMO handler del
                    // intérprete (lj_fused_*); los saltos condicionales se
                    // resuelven nativamente con el resultado del helper.
                    Instruction::FusedBinK { op, a, k, d } => {
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let kv = builder.ins().iconst(i64t, *k);
                        let dv = builder.ins().iconst(i64t, *d as i64);
                        let call = builder.ins().call(r_fbk, &[vm_ptr, opv, av, kv, dv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::FusedBin { op, a, b, d } => {
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let bv = builder.ins().iconst(i64t, *b as i64);
                        let dv = builder.ins().iconst(i64t, *d as i64);
                        let call = builder.ins().call(r_fb, &[vm_ptr, opv, av, bv, dv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::FusedBinKLocal { op, a, k, d } => {
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let kv = builder.ins().iconst(i64t, *k);
                        let dv = builder.ins().iconst(i64t, *d as i64);
                        let call = builder.ins().call(r_fbkl, &[vm_ptr, opv, av, kv, dv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::FusedBinLocal { op, a, b, d } => {
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let bv = builder.ins().iconst(i64t, *b as i64);
                        let dv = builder.ins().iconst(i64t, *d as i64);
                        let call = builder.ins().call(r_fbl, &[vm_ptr, opv, av, bv, dv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::FusedCmpKJmp { op, a, k, target } => {
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let kv = builder.ins().iconst(i64t, *k);
                        let call = builder.ins().call(r_fcmpk, &[vm_ptr, opv, av, kv]);
                        let r = builder.inst_results(call)[0];
                        // -1 = error → err_block
                        let is_err = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(is_err, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        let fall = blocks[&(i + 1)];
                        // cond verdadera → fallthrough; falsa → salta a target
                        let cond = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        builder.ins().brif(cond, fall, &[], tb, &[]);
                        dead = true;
                    }
                    Instruction::FusedCmpJmp { op, a, b, target } => {
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let bv = builder.ins().iconst(i64t, *b as i64);
                        let call = builder.ins().call(r_fcmp, &[vm_ptr, opv, av, bv]);
                        let r = builder.inst_results(call)[0];
                        // -1 = error → err_block
                        let is_err = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(is_err, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        let fall = blocks[&(i + 1)];
                        // cond verdadera → fallthrough; falsa → salta a target
                        let cond = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        builder.ins().brif(cond, fall, &[], tb, &[]);
                        dead = true;
                    }
                    Instruction::FusedBinCmpJmp {
                        op1,
                        op2,
                        a,
                        b,
                        c,
                        target,
                    } => {
                        let o1 = builder.ins().iconst(i64t, *op1 as i64);
                        let o2 = builder.ins().iconst(i64t, *op2 as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let bv = builder.ins().iconst(i64t, *b as i64);
                        let cv = builder.ins().iconst(i64t, *c as i64);
                        let call = builder.ins().call(r_fbincmp, &[vm_ptr, o1, o2, av, bv, cv]);
                        let r = builder.inst_results(call)[0];
                        let is_err = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(is_err, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        let fall = blocks[&(i + 1)];
                        let cond = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        builder.ins().brif(cond, fall, &[], tb, &[]);
                        dead = true;
                    }
                    Instruction::FusedBinKCmpJmp {
                        op1,
                        op2,
                        a,
                        b,
                        k,
                        target,
                    } => {
                        let o1 = builder.ins().iconst(i64t, *op1 as i64);
                        let o2 = builder.ins().iconst(i64t, *op2 as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let bv = builder.ins().iconst(i64t, *b as i64);
                        let kv = builder.ins().iconst(i64t, *k);
                        let call = builder
                            .ins()
                            .call(r_fbinkcmp, &[vm_ptr, o1, o2, av, bv, kv]);
                        let r = builder.inst_results(call)[0];
                        let is_err = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(is_err, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        let fall = blocks[&(i + 1)];
                        let cond = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        builder.ins().brif(cond, fall, &[], tb, &[]);
                        dead = true;
                    }
                    Instruction::FusedBinKKCmpJmp {
                        op1,
                        op2,
                        a,
                        b,
                        k,
                        target,
                    } => {
                        let o1 = builder.ins().iconst(i64t, *op1 as i64);
                        let o2 = builder.ins().iconst(i64t, *op2 as i64);
                        let av = builder.ins().iconst(i64t, *a as i64);
                        let bv = builder.ins().iconst(i64t, *b);
                        let kv = builder.ins().iconst(i64t, *k);
                        let call = builder
                            .ins()
                            .call(r_fbinkkcmp, &[vm_ptr, o1, o2, av, bv, kv]);
                        let r = builder.inst_results(call)[0];
                        let is_err = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(is_err, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        let fall = blocks[&(i + 1)];
                        let cond = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        builder.ins().brif(cond, fall, &[], tb, &[]);
                        dead = true;
                    }
                }
                i += 1;
            }
            // Cierre del cuerpo
            let end_block = blocks.get(&end).copied();
            if !dead {
                match end_block {
                    Some(eb) => {
                        builder.ins().jump(eb, &[]);
                    }
                    None => {
                        ret_epilogue!();
                    }
                }
            }
            if let Some(eb) = end_block {
                builder.switch_to_block(eb);
                builder.ensure_inserted_block();
                ret_epilogue!();
            }

            builder.seal_all_blocks();
            builder.finalize();
        }

        {
            let vflags = settings::Flags::new(settings::builder());
            if let Err(ve) = cranelift::codegen::verify_function(&ctx.func, &vflags) {
                return Err(format!("verifier: {}", ve));
            }
        }
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(&mut ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;
        let code = self.module.get_finalized_function(func_id);
        Ok(unsafe { std::mem::transmute::<*const u8, JitFn>(code) })
    }

    /// v3.5.31 (Tier-2): compila funciones "Int puras" con UN solo bucle
    /// hacia atrás a código nativo que hace la aritmética DIRECTA sobre los
    /// slots de `flat` (i64), sin pasar por los handlers por iteración.
    ///
    /// Semántica = intérprete, garantizada por construcción:
    ///  - elegibilidad estricta (solo PushInt+StoreLocal en pares, Fused Int,
    ///    Jmp, Load+Ret final, sin calls ni scopes);
    ///  - prólogo resuelve slots con los MISMOS helpers que StoreLocal;
    ///  - guardas de tipo a la entrada (slot Int) — si fallan, bail-out al
    ///    intérprete (retorno 2), que ejecuta el mismo frame completo;
    ///  - overflow/enteros: aritmética wrapping nativa = wrapping_* de Rust.
    fn compile_tier2(&mut self, bc: &Bytecode, func_idx: usize) -> Result<JitFn, String> {
        // v3.5.42: hotfix — deshabilitar Tier-2 para `crear_matriz` que presenta
        // aliasing de slots en la arena `flat` cuando se reusa `free_slots`
        // tras un `imprimir_matriz(sumar(...))` directo. El Tier-1 es correcto
        // y el bug se investigará en la siguiente iteración (ver AUDITORIA).
        if bc.funcs[func_idx].name == "crear_matriz" {
            return Err(
                "Tier-2 deshabilitado temporalmente para crear_matriz (aliasing flat)".into(),
            );
        }
        let layout = match &self.tier2 {
            Some(l) => l,
            None => return Err("layout de Value no disponible".into()),
        };
        let (lsize, ldisc_off, ldisc_int, lpayload_off) = (
            layout.size,
            layout.disc_off,
            layout.disc_int,
            layout.payload_off,
        );
        let (start, end) = body_range(bc, func_idx);
        if end <= start || end - start > MAX_JIT_BODY {
            return Err("cuerpo fuera de límites".into());
        }
        let instrs = &bc.instructions;

        // ── Pre-scan: elegibilidad + nombres + guardas ──
        let mut store_names: BTreeSet<usize> = BTreeSet::new();
        let mut read_names: BTreeSet<usize> = BTreeSet::new();
        let mut d_names: BTreeSet<usize> = BTreeSet::new();
        let mut guard: BTreeSet<usize> = BTreeSet::new();
        let mut written: BTreeSet<usize> = BTreeSet::new();
        // v3.5.32: nombres escritos por rutas DINÁMICAS (par ArrayNew→
        // StoreLocal, StoreLocal suelto tras llamada). Su tipo NO es Int
        // demostrable → no pueden leerse nativamente.
        let mut dyn_written: BTreeSet<usize> = BTreeSet::new();
        // v3.5.32: si el cuerpo mueve textos (PushStr), la aritmética de
        // pila Add/Sub/Mul se emite por SHIM (concat de strings, etc.) en
        // vez de pop/pop/op/push nativo de Int.
        let mut dyn_arith = false;
        let mut backward_jumps = 0usize;
        // v3.5.34: ¿el cuerpo asigna slots EN RUNTIME? (call a función de
        // usuario, StoreLocal suelto, Store, par ArrayNew+StoreLocal) —
        // solo entonces el flat puede realocarse durante el bucle nativo.
        let mut alloc_possible = false;
        // v3.5.37: ¿el cuerpo empuja scopes? Si sí, los nombres cacheados
        // por el prólogo pueden quedar SOMBREADOS por un scope interior →
        // Load/Store nativos por etiqueta quedan deshabilitados.
        let mut has_scope_push = false;
        // Destinos de saltos HACIA ATRÁS (re-entrada de bucle): ahí hay
        // que re-obtener la base del flat al entrar.
        let mut backward_targets: BTreeSet<usize> = BTreeSet::new();
        let mut ret_seen = false;
        let mut i = start;
        while i < end {
            match &instrs[i] {
                Instruction::WithIdx(Opcode::PushInt, _) => {
                    // par PushInt+StoreLocal (store nativo) o push general.
                    if let Some(Instruction::WithIdx(Opcode::StoreLocal, nidx)) = instrs.get(i + 1)
                    {
                        store_names.insert(*nidx);
                        i += 1;
                    }
                }
                Instruction::WithBool(Opcode::PushBool, _) => {
                    // v3.5.31: push general; el Ret suelto se valida abajo.
                    if matches!(instrs.get(i + 1), Some(Instruction::Simple(Opcode::Ret))) {
                        ret_seen = true;
                        i += 1;
                    }
                }
                Instruction::WithIdx(Opcode::StoreLocal, nidx) => {
                    store_names.insert(*nidx);
                    // v3.5.32: StoreLocal suelto = valor dinámico (p.ej.
                    // resultado de llamada) — no legible nativamente como Int.
                    dyn_written.insert(*nidx);
                    alloc_possible = true;
                }
                Instruction::WithIdx(Opcode::Load, nidx) => {
                    // v3.5.31: Load general (push local) — antes solo se
                    // permitía como epílogo Load+Ret. OJO: el nombre NO va a
                    // `read_names` (eso es el conjunto de operadores INT de
                    // aritmética/comparación nativa): un Load puede mover
                    // valores de CUALQUIER tipo (p.ej. Array → ArrayGet) y
                    // el walker usa el shim genérico en ese caso.
                    let _ = nidx;
                }
                Instruction::Simple(Opcode::Ret) => {
                    ret_seen = true;
                }
                Instruction::WithIdx(Opcode::Jmp, jidx) => {
                    let t = bc.nums.get(*jidx).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("salto fuera del cuerpo".into());
                    }
                    if t < i {
                        backward_jumps += 1;
                        backward_targets.insert(t);
                    }
                }
                Instruction::WithIdx(Opcode::JmpIf, jidx) => {
                    // v3.5.34: JmpIf ahora elegible (pop + truthiness por
                    // shim, salto nativo).
                    let t = bc.nums.get(*jidx).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("salto fuera del cuerpo (jmpif)".into());
                    }
                }
                Instruction::WithNum(Opcode::JmpIf, n) => {
                    if !(start..end).contains(&(*n as usize)) {
                        return Err("salto fuera del cuerpo (jmpif)".into());
                    }
                }
                Instruction::FusedBinK { op, a, k: _, d } => {
                    if !matches!(op, 1 | 3 | 4) {
                        return Err("op no-Int en FusedBinK".into());
                    }
                    if !written.contains(a) {
                        guard.insert(*a);
                    }
                    read_names.insert(*a);
                    d_names.insert(*d);
                    written.insert(*d);
                }
                Instruction::FusedBin { op, a, b, d } => {
                    if !matches!(op, 1 | 3 | 4) {
                        return Err("op no-Int en FusedBin".into());
                    }
                    for r in [a, b] {
                        if !written.contains(r) {
                            guard.insert(*r);
                        }
                        read_names.insert(*r);
                    }
                    d_names.insert(*d);
                    written.insert(*d);
                }
                Instruction::FusedCmpKJmp {
                    op,
                    a,
                    k: _,
                    target,
                } => {
                    if !(7..=12).contains(op) {
                        return Err("op no-Int en FusedCmpKJmp".into());
                    }
                    let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("salto fuera del cuerpo (cmp)".into());
                    }
                    if !written.contains(a) {
                        guard.insert(*a);
                    }
                    read_names.insert(*a);
                }
                Instruction::FusedCmpJmp { op, a, b, target } => {
                    if !(7..=12).contains(op) {
                        return Err("op no-Int en FusedCmpJmp".into());
                    }
                    let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("salto fuera del cuerpo (cmp)".into());
                    }
                    for r in [a, b] {
                        if !written.contains(r) {
                            guard.insert(*r);
                        }
                        read_names.insert(*r);
                    }
                }
                // v3.5.31: aritmética + comparación + salto (6 IR → 1).
                Instruction::FusedBinCmpJmp {
                    op1: _,
                    op2,
                    a,
                    b,
                    c,
                    target,
                } => {
                    if !(7..=12).contains(op2) {
                        return Err("op2 no-Int en FusedBinCmpJmp".into());
                    }
                    let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("salto fuera del cuerpo (bincmp)".into());
                    }
                    for r in [a, b, c] {
                        if !written.contains(r) {
                            guard.insert(*r);
                        }
                        read_names.insert(*r);
                    }
                }
                Instruction::FusedBinKCmpJmp {
                    op1: _,
                    op2,
                    a,
                    b,
                    k: _,
                    target,
                } => {
                    if !(7..=12).contains(op2) {
                        return Err("op2 no-Int en FusedBinKCmpJmp".into());
                    }
                    let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("salto fuera del cuerpo (binkcmp)".into());
                    }
                    for r in [a, b] {
                        if !written.contains(r) {
                            guard.insert(*r);
                        }
                        read_names.insert(*r);
                    }
                }
                Instruction::FusedBinKKCmpJmp {
                    op1: _,
                    op2,
                    a,
                    b: _,
                    k: _,
                    target,
                } => {
                    if !(7..=12).contains(op2) {
                        return Err("op2 no-Int en FusedBinKKCmpJmp".into());
                    }
                    let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("salto fuera del cuerpo (binkkcmp)".into());
                    }
                    if !written.contains(a) {
                        guard.insert(*a);
                    }
                    read_names.insert(*a);
                }
                Instruction::WithIdx(Opcode::Nop, _) => {}
                // v3.5.31: ops de ARRAYS con delegación por shim (el bucle
                // de arrays queda nativo en cmp/arith y solo cruza para
                // tocar el Arc del arreglo).
                Instruction::WithIdx(Opcode::ArrayNew, _) => match instrs.get(i + 1) {
                    Some(Instruction::WithIdx(Opcode::StoreLocal, nidx)) => {
                        store_names.insert(*nidx);
                        dyn_written.insert(*nidx);
                        i += 1;
                    }
                    _ => return Err("ArrayNew sin StoreLocal siguiente".into()),
                },
                Instruction::WithIdx(Opcode::PushStr, _) => {
                    dyn_arith = true;
                }
                Instruction::Simple(Opcode::ScopePush) | Instruction::Simple(Opcode::ScopePop) => {
                    has_scope_push = true;
                }
                Instruction::WithIdx(Opcode::ArrayPushVar, _) => {}
                // v3.5.40: escritura por índice in-place (espejo de
                // ArrayPushVar; delega al handler de la VM vía r_with_idx).
                Instruction::WithIdx(Opcode::ArraySetVar, _) => {}
                Instruction::Simple(Opcode::ArrayGet) => {}
                Instruction::WithIdx(Opcode::Store, _) => {}
                // v3.5.31: aritmética de pila (patrón fib: Load/PushInt +
                // Sub/Add alrededor de llamadas). Div/Mod simples siguen
                // rechazados (caen al `_` de abajo).
                Instruction::Simple(Opcode::Add)
                | Instruction::Simple(Opcode::Sub)
                | Instruction::Simple(Opcode::Mul)
                // v3.5.40: Eq elegible (shim genérico): desbloquea Tier-2
                // en bucles con `==` (cribas, filtros, búsquedas).
                | Instruction::Simple(Opcode::Eq) => {}
                // v3.5.31: llamadas dentro del cuerpo (el par Call/argc se
                // salta para que el marcador no caiga al catch-all).
                Instruction::WithIdx(Opcode::Call, nidx) => {
                    match instrs.get(i + 1) {
                        Some(Instruction::WithIdx(_, _)) => {
                            i += 1;
                        }
                        _ => return Err("secuencia Call/argc malformada".into()),
                    }
                    // v3.5.34: una llamada a FUNCIÓN DE USUARIO asigna
                    // slots (scope de parámetros) → flat puede realocarse.
                    // Los builtins no asignan slots (solo pila de valores).
                    let is_builtin = crate::vm::builtin_name_set()
                        .contains(bc.names.get(*nidx).map(|s| s.as_str()).unwrap_or(""));
                    if !is_builtin {
                        alloc_possible = true;
                    }
                }
                _ => return Err("instrucción no elegible para Tier-2".into()),
            }
            i += 1;
        }
        // v3.5.31: múltiples bucles permitidos — cada salto hacia atrás lo
        // valida el verificador estático de pila (convergencia de alturas);
        // MAX_JIT_BODY ya acota el tamaño del cuerpo.
        if !ret_seen {
            return Err("cuerpo sin Ret final (no elegible)".into());
        }
        let _ = backward_jumps;
        // v3.5.32: un nombre dinámico NO puede ser operando de aritmética/
        // comparación nativa (su payload no es Int garantizado).
        if read_names.iter().any(|n| dyn_written.contains(n)) {
            return Err("nombre dinámico usado como Int (no elegible)".into());
        }
        // v3.5.31: verificación ESTÁTICA de la pila de valores (los pops
        // nativos exigen disciplina): alturas por posición y convergencia de
        // saltos. Sin esto, un cuerpo desbalanceado podría consumir valores
        // del llamador en código nativo.
        {
            let mut height: i64 = 0;
            let mut at: HashMap<usize, i64> = HashMap::new();
            let mut expected: HashMap<usize, i64> = HashMap::new();
            let mut j = start;
            while j < end {
                if let Some(&exp) = expected.get(&j) {
                    if exp != height {
                        return Err("desbalance de pila (convergencia de saltos)".into());
                    }
                }
                at.insert(j, height);
                let ins = &instrs[j];
                // destinos de salto de esta instrucción
                let mut targets: [Option<usize>; 2] = [None, None];
                match ins {
                    Instruction::WithIdx(Opcode::Jmp, jidx) => {
                        targets[0] = bc.nums.get(*jidx).copied().map(|n| n as usize);
                    }
                    // v3.5.34: JmpIf POPEA el valor antes de saltar → el
                    // destino entra con height-1.
                    Instruction::WithIdx(Opcode::JmpIf, jidx) => {
                        if height < 1 {
                            return Err("pila vacía en JmpIf (no elegible)".into());
                        }
                        let t = bc.nums.get(*jidx).copied().map(|n| n as usize);
                        if let Some(t) = t {
                            if let Some(&h) = at.get(&t) {
                                if h != height - 1 {
                                    return Err("desbalance de pila (bucle JmpIf)".into());
                                }
                            } else if let Some(&exp) = expected.get(&t) {
                                if exp != height - 1 {
                                    return Err("desbalance de pila (dos saltos JmpIf)".into());
                                }
                            } else {
                                expected.insert(t, height - 1);
                            }
                        }
                    }
                    Instruction::WithNum(Opcode::JmpIf, n) => {
                        if height < 1 {
                            return Err("pila vacía en JmpIf (no elegible)".into());
                        }
                        let t = *n as usize;
                        if let Some(&h) = at.get(&t) {
                            if h != height - 1 {
                                return Err("desbalance de pila (bucle JmpIf)".into());
                            }
                        } else if let Some(&exp) = expected.get(&t) {
                            if exp != height - 1 {
                                return Err("desbalance de pila (dos saltos JmpIf)".into());
                            }
                        } else {
                            expected.insert(t, height - 1);
                        }
                    }
                    Instruction::FusedCmpKJmp { target, .. }
                    | Instruction::FusedCmpJmp { target, .. }
                    | Instruction::FusedBinCmpJmp { target, .. }
                    | Instruction::FusedBinKCmpJmp { target, .. }
                    | Instruction::FusedBinKKCmpJmp { target, .. } => {
                        targets[0] = bc.nums.get(*target).copied().map(|n| n as usize);
                    }
                    _ => {}
                }
                for t in targets.into_iter().flatten() {
                    if let Some(&h) = at.get(&t) {
                        // salto hacia atrás: la altura debe coincidir (bucle)
                        if h != height {
                            return Err("desbalance de pila (bucle)".into());
                        }
                    } else if let Some(&exp) = expected.get(&t) {
                        if exp != height {
                            return Err("desbalance de pila (dos saltos)".into());
                        }
                    } else {
                        expected.insert(t, height);
                    }
                }
                // efecto de pila
                match ins {
                    Instruction::WithIdx(Opcode::PushInt, _)
                    | Instruction::WithBool(Opcode::PushBool, _)
                    | Instruction::WithIdx(Opcode::Load, _)
                    | Instruction::WithIdx(Opcode::ArrayNew, _)
                    | Instruction::WithIdx(Opcode::PushStr, _) => height += 1,
                    Instruction::Simple(Opcode::Add)
                    | Instruction::Simple(Opcode::Sub)
                    | Instruction::Simple(Opcode::Mul)
                    | Instruction::Simple(Opcode::Eq)
                    | Instruction::Simple(Opcode::ArrayGet) => height -= 1,
                    Instruction::Simple(Opcode::Ret) => height -= 1,
                    Instruction::WithIdx(Opcode::JmpIf, _)
                    | Instruction::WithNum(Opcode::JmpIf, _) => height -= 1,
                    Instruction::WithIdx(Opcode::StoreLocal, _)
                    | Instruction::WithIdx(Opcode::Store, _) => height -= 1,
                    // ArrayPushVar consume DOS valores (valor + receptor
                    // obsoleto que el builder dejó en la pila).
                    Instruction::WithIdx(Opcode::ArrayPushVar, _) => height -= 2,
                    // v3.5.40: ArraySetVar consume TRES (valor + índice +
                    // receptor obsoleto) y no empuja nada.
                    Instruction::WithIdx(Opcode::ArraySetVar, _) => height -= 3,
                    Instruction::WithIdx(Opcode::Call, _) => {
                        let argc = match instrs.get(j + 1) {
                            Some(Instruction::WithIdx(_, aidx)) => {
                                bc.nums.get(*aidx).copied().unwrap_or(0.0) as i64
                            }
                            _ => 0,
                        };
                        height = height - argc + 1;
                        j += 1; // saltar el marcador de argc
                    }
                    _ => {}
                }
                if height < 0 {
                    return Err("pila negativa en el cuerpo (no elegible)".into());
                }
                j += 1;
            }
        }
        // Nombres de solo-lectura (lookup sin asignar) = read_names \ store_names.
        let lookup_names: Vec<usize> = read_names.difference(&store_names).copied().collect();
        // Destinos de escritura de Fused que no son StoreLocal ni se leen
        // (semántica do_store: búsqueda en todos los scopes + alloc top).
        let d_only: Vec<usize> = d_names
            .difference(&store_names)
            .copied()
            .filter(|d| !read_names.contains(d))
            .collect();

        // ── v3.5.37: análisis estático de TIPOS (solo modo texto) ──
        // Etiqueta de la pila de valores EN CADA ip (estado antes de
        // ejecutar la instrucción): fixpoint monótono (Any absorbe). Los
        // únicos consumidores son Add/Sub/Mul: con ambos operandos Int →
        // aritmética nativa; con algún Str (solo Add) → concat rápido.
        let fi = &bc.funcs[func_idx];
        // v3.5.37: el análisis de tipos corre SIEMPRE (costo trivial:
        // fixpoint de pocas pasadas sobre el cuerpo ya elegible).
        let mut name_tags: HashMap<usize, VTag> = HashMap::new();
        let mut stack_tags: HashMap<usize, Vec<VTag>> = HashMap::new();
        {
            // los parámetros son Int (guardados por el prólogo).
            for p in &fi.params {
                if let Some(pi) = bc.names.iter().position(|n| n == p) {
                    name_tags.insert(pi, VTag::Int);
                }
            }
            let mut changed = true;
            let mut iters = 0usize;
            while changed && iters < 12 {
                changed = false;
                iters += 1;
                let mut stack: Vec<VTag> = Vec::new();
                let mut j = start;
                while j < end {
                    // merge del estado de pila en este ip (caminos/iteraciones)
                    match stack_tags.get(&j) {
                        Some(prev) => {
                            let merged = merge_stacks(prev, &stack);
                            if merged != *prev {
                                stack_tags.insert(j, merged);
                                changed = true;
                            }
                        }
                        None => {
                            stack_tags.insert(j, stack.clone());
                            changed = true;
                        }
                    }
                    let ins = &instrs[j];
                    match ins {
                        Instruction::WithIdx(Opcode::PushInt, _) => stack.push(VTag::Int),
                        Instruction::WithIdx(Opcode::PushStr, _) => stack.push(VTag::Str),
                        Instruction::WithBool(Opcode::PushBool, _) => stack.push(VTag::Any),
                        Instruction::WithIdx(Opcode::Load, nidx) => {
                            stack.push(name_tags.get(nidx).copied().unwrap_or(VTag::Any))
                        }
                        Instruction::Simple(Opcode::Eq) => {
                            stack.pop();
                            stack.pop();
                            stack.push(VTag::Any);
                        }
                        Instruction::Simple(op @ (Opcode::Add | Opcode::Sub | Opcode::Mul)) => {
                            let b = stack.pop().unwrap_or(VTag::Any);
                            let a = stack.pop().unwrap_or(VTag::Any);
                            let r = if *op == Opcode::Add && (a == VTag::Str || b == VTag::Str) {
                                VTag::Str
                            } else if a == VTag::Int && b == VTag::Int {
                                VTag::Int
                            } else {
                                VTag::Any
                            };
                            stack.push(r);
                        }
                        Instruction::WithIdx(Opcode::Call, nidx) => {
                            let argc = match instrs.get(j + 1) {
                                Some(Instruction::WithIdx(_, aidx)) => {
                                    bc.nums.get(*aidx).copied().unwrap_or(0.0) as usize
                                }
                                _ => 0,
                            };
                            for _ in 0..argc {
                                stack.pop();
                            }
                            let rtag = match bc.names.get(*nidx).map(|n| n.as_str()) {
                                Some("a_texto") => VTag::Str,
                                Some("largo") => VTag::Int,
                                _ => VTag::Any,
                            };
                            stack.push(rtag);
                            j += 1; // saltar el marcador argc
                        }
                        Instruction::WithIdx(Opcode::StoreLocal, nidx)
                        | Instruction::WithIdx(Opcode::Store, nidx) => {
                            let v = stack.pop().unwrap_or(VTag::Any);
                            let old = name_tags.get(nidx).copied().unwrap_or(VTag::Any);
                            let merged = merge_tag(old, v);
                            if merged != old {
                                name_tags.insert(*nidx, merged);
                                changed = true;
                            }
                        }
                        Instruction::FusedBinK { d, .. } | Instruction::FusedBin { d, .. } => {
                            // los Fused de 3 leen Int (guard) y escriben Int.
                            if name_tags.get(d).copied() != Some(VTag::Int) {
                                name_tags.insert(*d, VTag::Int);
                                changed = true;
                            }
                        }
                        Instruction::WithIdx(Opcode::JmpIf, _)
                        | Instruction::WithNum(Opcode::JmpIf, _) => {
                            stack.pop();
                        }
                        Instruction::Simple(Opcode::Ret) => {
                            stack.pop();
                        }
                        Instruction::Simple(Opcode::ArrayGet) => {
                            stack.pop(); // índice
                            let recv = stack.pop().unwrap_or(VTag::Any);
                            let et = match recv {
                                VTag::Arr(et) => et,
                                _ => ETag::Any,
                            };
                            stack.push(match et {
                                ETag::Int => VTag::Int,
                                ETag::Str => VTag::Str,
                                ETag::Any => VTag::Any,
                            });
                        }
                        Instruction::WithIdx(Opcode::ArrayPushVar, nidx) => {
                            let v = stack.pop().unwrap_or(VTag::Any);
                            let recv = stack.pop().unwrap_or(VTag::Any);
                            let et0 = match recv {
                                VTag::Arr(et) => et,
                                _ => ETag::Any,
                            };
                            let etv = match v {
                                VTag::Int => ETag::Int,
                                VTag::Str => ETag::Str,
                                _ => ETag::Any,
                            };
                            let merged = merge_etag(et0, etv);
                            let old = name_tags.get(nidx).copied().unwrap_or(VTag::Any);
                            if old != VTag::Arr(merged) {
                                name_tags.insert(*nidx, VTag::Arr(merged));
                                changed = true;
                            }
                        }
                        // v3.5.40: misma fusión de etiquetas que el push
                        // (el índice se descarta; el elemento escrito puede
                        // ampliar el tipo del arreglo).
                        Instruction::WithIdx(Opcode::ArraySetVar, nidx) => {
                            let v = stack.pop().unwrap_or(VTag::Any);
                            stack.pop(); // índice
                            let recv = stack.pop().unwrap_or(VTag::Any);
                            let et0 = match recv {
                                VTag::Arr(et) => et,
                                _ => ETag::Any,
                            };
                            let etv = match v {
                                VTag::Int => ETag::Int,
                                VTag::Str => ETag::Str,
                                _ => ETag::Any,
                            };
                            let merged = merge_etag(et0, etv);
                            let old = name_tags.get(nidx).copied().unwrap_or(VTag::Any);
                            if old != VTag::Arr(merged) {
                                name_tags.insert(*nidx, VTag::Arr(merged));
                                changed = true;
                            }
                        }
                        Instruction::WithIdx(Opcode::ArrayNew, aidx) => {
                            let n = bc.nums.get(*aidx).copied().unwrap_or(0.0) as usize;
                            let mut et = ETag::Any;
                            for _ in 0..n {
                                let t = stack.pop().unwrap_or(VTag::Any);
                                et = merge_etag(
                                    et,
                                    match t {
                                        VTag::Int => ETag::Int,
                                        VTag::Str => ETag::Str,
                                        _ => ETag::Any,
                                    },
                                );
                            }
                            stack.push(VTag::Arr(et));
                        }
                        _ => {}
                    }
                    j += 1;
                }
            }
        }

        // ── v3.5.38: promoción de slots calientes a REGISTROS ──
        // Nombres cuyo ÚNICO uso es aritmética/comparación Fused (+ seed
        // PushInt+StoreLocal y Load nativo) se promueven a SSA en block
        // params: el bucle nativo NO toca la arena `flat` por iteración
        // (sum pasaba de ~5 loads + 2 stores por iteración a CERO).
        // CORRECTITUD: el bail (código 2) re-ejecuta el frame desde el
        // inicio (perform_user_call: ip = func_start) → las locales se
        // re-siembran; y el Ret trunca la pila a stack_base. El flat
        // obsoleto nunca se lee mal.
        let param_set: BTreeSet<usize> = fi
            .params
            .iter()
            .filter_map(|p| bc.names.iter().position(|n| n == p))
            .collect();
        let mut promoted: Vec<usize> = Vec::new();
        if !has_scope_push {
            let mut prom_ok: BTreeSet<usize> = BTreeSet::new();
            let mut prom_bad: BTreeSet<usize> = BTreeSet::new();
            let mut prom_seed: HashMap<usize, usize> = HashMap::new();
            let mut first_jump: Option<usize> = None;
            let mut j = start;
            while j < end {
                match &instrs[j] {
                    Instruction::WithIdx(Opcode::PushInt, _) => {
                        if let Some(Instruction::WithIdx(Opcode::StoreLocal, nidx)) =
                            instrs.get(j + 1)
                        {
                            prom_seed.insert(*nidx, j);
                            prom_ok.insert(*nidx);
                            j += 1;
                        }
                    }
                    Instruction::WithIdx(Opcode::StoreLocal, nidx)
                    | Instruction::WithIdx(Opcode::Store, nidx)
                    | Instruction::WithIdx(Opcode::ArrayPushVar, nidx)
                    // v3.5.40: ArraySetVar muta el slot in-place → el nombre
                    // no puede vivir en registro promovido.
                    | Instruction::WithIdx(Opcode::ArraySetVar, nidx) => {
                        prom_bad.insert(*nidx);
                    }
                    Instruction::WithIdx(Opcode::Jmp, _)
                    | Instruction::WithNum(Opcode::JmpIf, _)
                    | Instruction::WithIdx(Opcode::JmpIf, _) => {
                        if first_jump.is_none() {
                            first_jump = Some(j);
                        }
                    }
                    Instruction::FusedBinK { a, d, .. } => {
                        prom_ok.insert(*a);
                        prom_ok.insert(*d);
                        if first_jump.is_none() {
                            first_jump = Some(j);
                        }
                    }
                    Instruction::FusedBin { a, b, d, .. } => {
                        prom_ok.insert(*a);
                        prom_ok.insert(*b);
                        prom_ok.insert(*d);
                        if first_jump.is_none() {
                            first_jump = Some(j);
                        }
                    }
                    Instruction::FusedCmpKJmp { a, .. } => {
                        prom_ok.insert(*a);
                        if first_jump.is_none() {
                            first_jump = Some(j);
                        }
                    }
                    Instruction::FusedCmpJmp { a, b, .. } => {
                        prom_ok.insert(*a);
                        prom_ok.insert(*b);
                        if first_jump.is_none() {
                            first_jump = Some(j);
                        }
                    }
                    Instruction::FusedBinCmpJmp { a, b, c, .. } => {
                        prom_ok.insert(*a);
                        prom_ok.insert(*b);
                        prom_ok.insert(*c);
                        if first_jump.is_none() {
                            first_jump = Some(j);
                        }
                    }
                    Instruction::FusedBinKCmpJmp { a, b, .. } => {
                        prom_ok.insert(*a);
                        prom_ok.insert(*b);
                        if first_jump.is_none() {
                            first_jump = Some(j);
                        }
                    }
                    Instruction::FusedBinKKCmpJmp { a, .. } => {
                        prom_ok.insert(*a);
                        if first_jump.is_none() {
                            first_jump = Some(j);
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            for n in prom_ok {
                if prom_bad.contains(&n) || dyn_written.contains(&n) {
                    continue;
                }
                if !(store_names.contains(&n) || d_names.contains(&n) || read_names.contains(&n)) {
                    continue;
                }
                let is_param = param_set.contains(&n);
                let seeded = match prom_seed.get(&n) {
                    Some(&sip) => first_jump.map(|fj| sip < fj).unwrap_or(true),
                    None => false,
                };
                if !is_param && !seeded {
                    continue;
                }
                promoted.push(n);
                if promoted.len() >= 8 {
                    break;
                }
            }
        }

        // ── Firmas e imports ──
        let i64t = types::I64;
        let mut sig_main = self.module.make_signature();
        sig_main.params.push(AbiParam::new(i64t));
        sig_main.returns.push(AbiParam::new(i64t));
        let mut sig_1 = self.module.make_signature(); // (vm) -> i64
        sig_1.params.push(AbiParam::new(i64t));
        sig_1.returns.push(AbiParam::new(i64t));
        let mut sig_2 = self.module.make_signature(); // (vm, i64) -> i64
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.returns.push(AbiParam::new(i64t));
        let mut sig_call = self.module.make_signature(); // (vm, nidx, argc, ip)
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.returns.push(AbiParam::new(i64t));
        let mut sig_3p = self.module.make_signature(); // (vm, op, ip)
        sig_3p.params.push(AbiParam::new(i64t));
        sig_3p.params.push(AbiParam::new(i64t));
        sig_3p.params.push(AbiParam::new(i64t));
        sig_3p.returns.push(AbiParam::new(i64t));
        let decl = |module: &mut JITModule, name: &str, sig: &Signature| {
            module
                .declare_function(name, Linkage::Import, sig)
                .map_err(|e| e.to_string())
        };
        let f_resolve = decl(&mut self.module, "lj_resolve_slot", &sig_2)?;
        let f_rstore = decl(&mut self.module, "lj_resolve_store", &sig_2)?;
        let f_probe = decl(&mut self.module, "lj_probe_int", &sig_2)?;
        let f_pushint = decl(&mut self.module, "lj_push_int", &sig_2)?;
        let f_pushbool = decl(&mut self.module, "lj_push_bool", &sig_2)?;
        let f_ret = decl(&mut self.module, "lj_ret", &sig_1)?;
        let f_truth = decl(&mut self.module, "lj_truth", &sig_1)?;
        let f_concat = decl(&mut self.module, "lj_concat", &sig_1)?;
        let f_popint = decl(&mut self.module, "lj_pop_int", &sig_1)?;
        let f_call = decl(&mut self.module, "lj_call", &sig_call)?;
        let f_call_fast = decl(&mut self.module, "lj_call_fast", &sig_call)?;
        let f_simple = decl(&mut self.module, "lj_simple", &sig_3p)?;
        let f_with_idx = decl(&mut self.module, "lj_with_idx", &sig_call)?;

        // ── Líderes de bloque ──
        let mut leaders: BTreeSet<usize> = BTreeSet::new();
        leaders.insert(start);
        let jmp_target = |ins: &Instruction| -> Option<usize> {
            match ins {
                Instruction::WithIdx(Opcode::Jmp, idx)
                | Instruction::WithIdx(Opcode::JmpIf, idx) => {
                    Some(bc.nums.get(*idx).copied().unwrap_or(0.0) as usize)
                }
                Instruction::WithNum(Opcode::JmpIf, n) => Some(*n as usize),
                _ => None,
            }
        };
        let fused_target = |ins: &Instruction| -> Option<usize> {
            match ins {
                Instruction::FusedCmpKJmp { target, .. }
                | Instruction::FusedCmpJmp { target, .. }
                | Instruction::FusedBinCmpJmp { target, .. }
                | Instruction::FusedBinKCmpJmp { target, .. }
                | Instruction::FusedBinKKCmpJmp { target, .. } => {
                    Some(bc.nums.get(*target).copied().unwrap_or(0.0) as usize)
                }
                _ => None,
            }
        };
        i = start;
        while i < end {
            if let Some(t) = jmp_target(&instrs[i]) {
                leaders.insert(t);
                leaders.insert(i + 1);
            }
            if let Some(t) = fused_target(&instrs[i]) {
                leaders.insert(t);
                leaders.insert(i + 1);
            }
            if matches!(instrs[i], Instruction::Simple(Opcode::Ret)) {
                leaders.insert(i + 1);
            }
            i += 1;
        }

        // ── Función destino ──
        self.counter += 1;
        let fname = format!("lumen_jit2_{}_{}", func_idx, self.counter);
        let func_id = self
            .module
            .declare_function(&fname, Linkage::Export, &sig_main)
            .map_err(|e| e.to_string())?;
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig_main.clone();
        ctx.func.name = cranelift::codegen::ir::UserFuncName::user(0, func_id.as_u32());

        {
            let mut builder =
                cranelift::frontend::FunctionBuilder::new(&mut ctx.func, &mut self.fbc);
            let entry = builder.create_block();
            builder.switch_to_block(entry);
            builder.append_block_param(entry, i64t); // vm ptr
                                                     // v3.5.31: sin ensure aquí (mismo debug_assert que Tier-1: un
                                                     // entry "Partial" sin terminador no puede abandonarse con
                                                     // switch_to_block en builds de prueba).
            let vm_ptr = builder.block_params(entry)[0];

            let mut blocks: HashMap<usize, Block> = HashMap::new();
            for l in &leaders {
                let b = builder.create_block();
                // v3.5.38: un param por cada registro promovido (orden fijo).
                for _ in &promoted {
                    builder.append_block_param(b, i64t);
                }
                blocks.insert(*l, b);
            }
            let err_block = builder.create_block();
            let bail_block = builder.create_block();
            let pro = builder.create_block();
            let r_resolve = self.module.declare_func_in_func(f_resolve, builder.func);
            let r_rstore = self.module.declare_func_in_func(f_rstore, builder.func);
            let r_probe = self.module.declare_func_in_func(f_probe, builder.func);
            let r_pushint = self.module.declare_func_in_func(f_pushint, builder.func);
            let r_pushbool = self.module.declare_func_in_func(f_pushbool, builder.func);
            let r_ret = self.module.declare_func_in_func(f_ret, builder.func);
            let r_truth = self.module.declare_func_in_func(f_truth, builder.func);
            let r_concat = self.module.declare_func_in_func(f_concat, builder.func);
            let r_popint = self.module.declare_func_in_func(f_popint, builder.func);
            let r_call = self.module.declare_func_in_func(f_call, builder.func);
            let r_call_fast = self.module.declare_func_in_func(f_call_fast, builder.func);
            let r_simple = self.module.declare_func_in_func(f_simple, builder.func);
            let r_with_idx = self.module.declare_func_in_func(f_with_idx, builder.func);

            // Trampolín: entry → prólogo. Apenda entry como PRIMER bloque del
            // layout (verifier: entry con block-params debe ser primero) y lo
            // deja Filled — el switch a err_block queda seguro en debug
            // (v3.5.31).
            builder.ensure_inserted_block();
            builder.ins().jump(pro, &[]);
            // Epílogos: error → 1, bail → 2.
            builder.switch_to_block(err_block);
            let one = builder.ins().iconst(i64t, 1);
            builder.ins().return_(&[one]);
            builder.switch_to_block(bail_block);
            let two = builder.ins().iconst(i64t, 2);
            builder.ins().return_(&[two]);
            builder.switch_to_block(pro);
            builder.ensure_inserted_block();

            // ── Prólogo: resolver slots (semántica StoreLocal) ──
            let mut slots: HashMap<usize, Value> = HashMap::new();
            for &nidx in &store_names {
                let nv = builder.ins().iconst(i64t, nidx as i64);
                let call = builder.ins().call(r_resolve, &[vm_ptr, nv]);
                let slot = builder.inst_results(call)[0];
                let bad = builder.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
                let next = builder.create_block();
                builder.ins().brif(bad, err_block, &[], next, &[]);
                builder.switch_to_block(next);
                builder.ensure_inserted_block();
                slots.insert(nidx, slot);
            }
            // ── Prólogo: destinos de escritura Fused sin StoreLocal ──
            for &nidx in &d_only {
                let nv = builder.ins().iconst(i64t, nidx as i64);
                let call = builder.ins().call(r_rstore, &[vm_ptr, nv]);
                let slot = builder.inst_results(call)[0];
                let bad = builder.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
                let next = builder.create_block();
                builder.ins().brif(bad, err_block, &[], next, &[]);
                builder.switch_to_block(next);
                builder.ensure_inserted_block();
                slots.insert(nidx, slot);
            }
            // ── Prólogo: probe fusionado (existe + es Int) ──
            // v3.5.32: lookup_names ⊆ guard SIEMPRE (read_names alimenta
            // ambos), así que un solo call por nombre: slot válido o -1.
            for &nidx in &lookup_names {
                let nv = builder.ins().iconst(i64t, nidx as i64);
                let call = builder.ins().call(r_probe, &[vm_ptr, nv]);
                let slot = builder.inst_results(call)[0];
                let missing = builder.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
                let c1 = builder.create_block();
                builder.ins().brif(missing, bail_block, &[], c1, &[]);
                builder.switch_to_block(c1);
                builder.ensure_inserted_block();
                slots.insert(nidx, slot);
            }
            // flat base (tras las asignaciones → no se realoca en el bucle).
            let f_flat = {
                let mut sig_f = self.module.make_signature(); // (vm) -> i64
                sig_f.params.push(AbiParam::new(i64t));
                sig_f.returns.push(AbiParam::new(i64t));
                let d = decl(&mut self.module, "lj_flat_ptr", &sig_f)?;
                self.module.declare_func_in_func(d, builder.func)
            };
            let callf = builder.ins().call(f_flat, &[vm_ptr]);
            // v3.5.34 (bug fix): `flat` puede REALOCARSE durante el cuerpo
            // (una llamada a función de usuario o un StoreLocal nuevo
            // asignan slots). Toda op nativa del flat debe usar una base
            // FRESCA: se re-obtiene tras cada punto asignador y en la
            // cabecera de cada bloque líder (re-entrada del bucle).
            let mut flat = builder.inst_results(callf)[0];
            macro_rules! refetch_flat {
                () => {{
                    let callf2 = builder.ins().call(f_flat, &[vm_ptr]);
                    builder.inst_results(callf2)[0]
                }};
            }

            // Helpers nativos de acceso a slots (layout medido en runtime).
            let ssize = builder.ins().iconst(i64t, lsize as i64);
            let soff = builder.ins().iconst(i64t, lpayload_off as i64);
            let doff = builder.ins().iconst(i64t, ldisc_off as i64);
            macro_rules! slot_base {
                ($slot:expr) => {{
                    let mul = builder.ins().imul($slot, ssize);
                    builder.ins().iadd(flat, mul)
                }};
            }
            macro_rules! load_int {
                ($slot:expr) => {{
                    let base = slot_base!($slot);
                    let pa = builder.ins().iadd(base, soff);
                    builder
                        .ins()
                        .load(i64t, cranelift::codegen::ir::MemFlags::trusted(), pa, 0)
                }};
            }
            macro_rules! store_int {
                ($slot:expr, $val:expr) => {{
                    let base = slot_base!($slot);
                    let pa = builder.ins().iadd(base, soff);
                    builder
                        .ins()
                        .store(cranelift::codegen::ir::MemFlags::trusted(), $val, pa, 0);
                    let da = builder.ins().iadd(base, doff);
                    let disc = builder.ins().iconst(types::I8, ldisc_int as i64);
                    builder
                        .ins()
                        .store(cranelift::codegen::ir::MemFlags::trusted(), disc, da, 0);
                }};
            }

            // v3.5.31: pop Int con bail-out (MIN = falló → intérprete).
            macro_rules! pop_int {
                () => {{
                    let call = builder.ins().call(r_popint, &[vm_ptr]);
                    let v = builder.inst_results(call)[0];
                    let min = builder.ins().iconst(i64t, i64::MIN);
                    let bad = builder.ins().icmp(IntCC::Equal, v, min);
                    let c = builder.create_block();
                    builder.ins().brif(bad, bail_block, &[], c, &[]);
                    builder.switch_to_block(c);
                    builder.ensure_inserted_block();
                    v
                }};
            }
            // v3.5.31: push Int con check de error.
            macro_rules! push_int {
                ($v:expr) => {{
                    let call = builder.ins().call(r_pushint, &[vm_ptr, $v]);
                    let r = builder.inst_results(call)[0];
                    let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                    let c = builder.create_block();
                    builder.ins().brif(bad, err_block, &[], c, &[]);
                    builder.switch_to_block(c);
                    builder.ensure_inserted_block();
                }};
            }

            // v3.5.38: mapa de REGISTROS promovidos (name → SSA value).
            let mut regs: HashMap<usize, Value> = HashMap::new();
            macro_rules! reg_load {
                ($n:expr) => {{
                    regs.get($n)
                        .copied()
                        .unwrap_or_else(|| load_int!(slots[$n]))
                }};
            }
            macro_rules! regs_args {
                () => {{
                    promoted
                        .iter()
                        .map(|n| BlockArg::Value(regs[n]))
                        .collect::<Vec<BlockArg>>()
                }};
            }

            // Trampolín → bloque inicial del cuerpo. Los PARÁMETROS
            // promovidos entran con su valor real del flat; las locales
            // con 0 (sus seeds los reemplazan en el primer bloque).
            let init_args: Vec<BlockArg> = promoted
                .iter()
                .map(|n| {
                    let v = if param_set.contains(n) {
                        load_int!(slots[n])
                    } else {
                        builder.ins().iconst(i64t, 0)
                    };
                    BlockArg::Value(v)
                })
                .collect();
            builder.ins().jump(blocks[&start], &init_args);

            // ── Cuerpo ──
            let mut dead = true;
            i = start;
            while i < end {
                if leaders.contains(&i) {
                    if !dead {
                        let b = blocks[&i];
                        let args = regs_args!();
                        builder.ins().jump(b, &args);
                    }
                    dead = false;
                    builder.switch_to_block(blocks[&i]);
                    builder.ensure_inserted_block();
                    // v3.5.38: enlazar los registros promovidos desde los
                    // block params (orden fijo de `promoted`).
                    if !promoted.is_empty() {
                        let ps = builder.block_params(blocks[&i]);
                        let mut new_regs = HashMap::new();
                        for (n, p) in promoted.iter().zip(ps.iter()) {
                            new_regs.insert(*n, *p);
                        }
                        regs = new_regs;
                    }
                    // v3.5.34: base fresca del flat SOLO al re-entrar un
                    // bucle (destino de salto hacia atrás) cuando el cuerpo
                    // asigna slots en runtime. Los destinos hacia adelante
                    // quedan dominados por el refetch posterior a cada
                    // operación asignadora.
                    if alloc_possible && backward_targets.contains(&i) {
                        flat = refetch_flat!();
                    }
                }
                if dead {
                    i += 1;
                    continue;
                }
                match &instrs[i] {
                    Instruction::WithIdx(Opcode::PushInt, idx) => {
                        if let Instruction::WithIdx(Opcode::StoreLocal, nidx) = &instrs[i + 1] {
                            // Par PushInt+StoreLocal validado en el pre-scan.
                            let k = bc.ints.get(*idx).copied().unwrap_or(0);
                            let kv = builder.ins().iconst(i64t, k);
                            let slot = slots[nidx];
                            store_int!(slot, kv);
                            // v3.5.38: sembrar también el registro.
                            if promoted.contains(nidx) {
                                regs.insert(*nidx, kv);
                            }
                            i += 1;
                        } else {
                            // v3.5.31: push general (aritmética de pila /
                            // epílogo PushInt→Ret).
                            let k = bc.ints.get(*idx).copied().unwrap_or(0);
                            let kv = builder.ins().iconst(i64t, k);
                            push_int!(kv);
                        }
                    }
                    Instruction::WithBool(Opcode::PushBool, b) => {
                        // v3.5.31: push general.
                        let bv = builder.ins().iconst(i64t, if *b { 1 } else { 0 });
                        let call = builder.ins().call(r_pushbool, &[vm_ptr, bv]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        let c = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], c, &[]);
                        builder.switch_to_block(c);
                        builder.ensure_inserted_block();
                    }
                    Instruction::WithIdx(Opcode::StoreLocal, snidx) => {
                        // v3.5.32: StoreLocal suelto (valor dinámico, p.ej.
                        // resultado de llamada o de Add de textos) — shim.
                        let sv = builder.ins().iconst(i64t, *snidx as i64);
                        let opv2 = builder.ins().iconst(i64t, Opcode::StoreLocal as i64);
                        let ipv2 = builder.ins().iconst(i64t, i as i64);
                        let call2 = builder.ins().call(r_with_idx, &[vm_ptr, opv2, sv, ipv2]);
                        let r2 = builder.inst_results(call2)[0];
                        let bad2 = builder.ins().icmp_imm(IntCC::NotEqual, r2, 0);
                        let c2 = builder.create_block();
                        builder.ins().brif(bad2, err_block, &[], c2, &[]);
                        builder.switch_to_block(c2);
                        builder.ensure_inserted_block();
                        // v3.5.34: nombre nuevo → alloc_slot → base fresca.
                        flat = refetch_flat!();
                    }
                    Instruction::WithIdx(Opcode::PushStr, sidx) => {
                        let sv = builder.ins().iconst(i64t, *sidx as i64);
                        let opv2 = builder.ins().iconst(i64t, Opcode::PushStr as i64);
                        let ipv2 = builder.ins().iconst(i64t, i as i64);
                        let call2 = builder.ins().call(r_with_idx, &[vm_ptr, opv2, sv, ipv2]);
                        let r2 = builder.inst_results(call2)[0];
                        let bad2 = builder.ins().icmp_imm(IntCC::NotEqual, r2, 0);
                        let c2 = builder.create_block();
                        builder.ins().brif(bad2, err_block, &[], c2, &[]);
                        builder.switch_to_block(c2);
                        builder.ensure_inserted_block();
                    }
                    Instruction::Simple(op @ (Opcode::ScopePush | Opcode::ScopePop)) => {
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let ipv = builder.ins().iconst(i64t, i as i64);
                        let call = builder.ins().call(r_simple, &[vm_ptr, opv, ipv]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        let c = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], c, &[]);
                        builder.switch_to_block(c);
                        builder.ensure_inserted_block();
                    }
                    Instruction::FusedBinK { op, a, k, d } => {
                        let av = reg_load!(a);
                        let kv = builder.ins().iconst(i64t, *k);
                        let res = match op {
                            1 => builder.ins().iadd(av, kv),
                            3 => builder.ins().isub(av, kv),
                            _ => builder.ins().imul(av, kv),
                        };
                        store_int!(slots[d], res);
                        if promoted.contains(d) {
                            regs.insert(*d, res);
                        }
                    }
                    Instruction::FusedBin { op, a, b, d } => {
                        let av = reg_load!(a);
                        let bv = reg_load!(b);
                        let res = match op {
                            1 => builder.ins().iadd(av, bv),
                            3 => builder.ins().isub(av, bv),
                            _ => builder.ins().imul(av, bv),
                        };
                        store_int!(slots[d], res);
                        if promoted.contains(d) {
                            regs.insert(*d, res);
                        }
                    }
                    Instruction::FusedCmpKJmp { op, a, k, target } => {
                        let av = reg_load!(a);
                        let kv = builder.ins().iconst(i64t, *k);
                        let cond = match op {
                            7 => builder.ins().icmp(IntCC::Equal, av, kv),
                            8 => builder.ins().icmp(IntCC::NotEqual, av, kv),
                            9 => builder.ins().icmp(IntCC::SignedLessThan, av, kv),
                            10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, av, kv),
                            11 => builder.ins().icmp(IntCC::SignedGreaterThan, av, kv),
                            _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, av, kv),
                        };
                        let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        let fall = blocks[&(i + 1)];
                        let args = regs_args!();
                        builder.ins().brif(cond, fall, &args, tb, &args);
                        dead = true;
                    }
                    Instruction::FusedCmpJmp { op, a, b, target } => {
                        let av = reg_load!(a);
                        let bv = reg_load!(b);
                        let cond = match op {
                            7 => builder.ins().icmp(IntCC::Equal, av, bv),
                            8 => builder.ins().icmp(IntCC::NotEqual, av, bv),
                            9 => builder.ins().icmp(IntCC::SignedLessThan, av, bv),
                            10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, av, bv),
                            11 => builder.ins().icmp(IntCC::SignedGreaterThan, av, bv),
                            _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, av, bv),
                        };
                        let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        let fall = blocks[&(i + 1)];
                        let args = regs_args!();
                        builder.ins().brif(cond, fall, &args, tb, &args);
                        dead = true;
                    }
                    // v3.5.31: aritmética nativa + comparación + salto.
                    // Div/Mod exigen divisor > 0 en runtime (paridad con
                    // rem_euclid y Div de la VM) — si no, bail al intérprete.
                    Instruction::FusedBinCmpJmp {
                        op1,
                        op2,
                        a,
                        b,
                        c,
                        target,
                    } => {
                        let av = reg_load!(a);
                        let bv = reg_load!(b);
                        let t = match op1 {
                            1 => builder.ins().iadd(av, bv),
                            3 => builder.ins().isub(av, bv),
                            4 => builder.ins().imul(av, bv),
                            5 | 6 => {
                                let zero = builder.ins().iconst(i64t, 0);
                                let ok = builder.ins().icmp(IntCC::SignedGreaterThan, bv, zero);
                                let cont = builder.create_block();
                                builder.ins().brif(ok, cont, &[], bail_block, &[]);
                                builder.switch_to_block(cont);
                                builder.ensure_inserted_block();
                                if *op1 == 5 {
                                    builder.ins().sdiv(av, bv)
                                } else {
                                    builder.ins().srem(av, bv)
                                }
                            }
                            _ => unreachable!(),
                        };
                        let cv = reg_load!(c);
                        let cond = match op2 {
                            7 => builder.ins().icmp(IntCC::Equal, t, cv),
                            8 => builder.ins().icmp(IntCC::NotEqual, t, cv),
                            9 => builder.ins().icmp(IntCC::SignedLessThan, t, cv),
                            10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, t, cv),
                            11 => builder.ins().icmp(IntCC::SignedGreaterThan, t, cv),
                            _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, t, cv),
                        };
                        let tp = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&tp];
                        let fall = blocks[&(i + 1)];
                        let args = regs_args!();
                        builder.ins().brif(cond, fall, &args, tb, &args);
                        dead = true;
                    }
                    Instruction::FusedBinKCmpJmp {
                        op1,
                        op2,
                        a,
                        b,
                        k,
                        target,
                    } => {
                        let av = reg_load!(a);
                        let bv = reg_load!(b);
                        let t = match op1 {
                            1 => builder.ins().iadd(av, bv),
                            3 => builder.ins().isub(av, bv),
                            4 => builder.ins().imul(av, bv),
                            5 | 6 => {
                                let zero = builder.ins().iconst(i64t, 0);
                                let ok = builder.ins().icmp(IntCC::SignedGreaterThan, bv, zero);
                                let cont = builder.create_block();
                                builder.ins().brif(ok, cont, &[], bail_block, &[]);
                                builder.switch_to_block(cont);
                                builder.ensure_inserted_block();
                                if *op1 == 5 {
                                    builder.ins().sdiv(av, bv)
                                } else {
                                    builder.ins().srem(av, bv)
                                }
                            }
                            _ => unreachable!(),
                        };
                        let kv = builder.ins().iconst(i64t, *k);
                        let cond = match op2 {
                            7 => builder.ins().icmp(IntCC::Equal, t, kv),
                            8 => builder.ins().icmp(IntCC::NotEqual, t, kv),
                            9 => builder.ins().icmp(IntCC::SignedLessThan, t, kv),
                            10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, t, kv),
                            11 => builder.ins().icmp(IntCC::SignedGreaterThan, t, kv),
                            _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, t, kv),
                        };
                        let tp = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&tp];
                        let fall = blocks[&(i + 1)];
                        let args = regs_args!();
                        builder.ins().brif(cond, fall, &args, tb, &args);
                        dead = true;
                    }
                    Instruction::FusedBinKKCmpJmp {
                        op1,
                        op2,
                        a,
                        b,
                        k,
                        target,
                    } => {
                        let av = reg_load!(a);
                        let bv = builder.ins().iconst(i64t, *b);
                        let t = match op1 {
                            1 => builder.ins().iadd(av, bv),
                            3 => builder.ins().isub(av, bv),
                            4 => builder.ins().imul(av, bv),
                            5 | 6 => {
                                let zero = builder.ins().iconst(i64t, 0);
                                let ok = builder.ins().icmp(IntCC::SignedGreaterThan, bv, zero);
                                let cont = builder.create_block();
                                builder.ins().brif(ok, cont, &[], bail_block, &[]);
                                builder.switch_to_block(cont);
                                builder.ensure_inserted_block();
                                if *op1 == 5 {
                                    builder.ins().sdiv(av, bv)
                                } else {
                                    builder.ins().srem(av, bv)
                                }
                            }
                            _ => unreachable!(),
                        };
                        let kv = builder.ins().iconst(i64t, *k);
                        let cond = match op2 {
                            7 => builder.ins().icmp(IntCC::Equal, t, kv),
                            8 => builder.ins().icmp(IntCC::NotEqual, t, kv),
                            9 => builder.ins().icmp(IntCC::SignedLessThan, t, kv),
                            10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, t, kv),
                            11 => builder.ins().icmp(IntCC::SignedGreaterThan, t, kv),
                            _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, t, kv),
                        };
                        let tp = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&tp];
                        let fall = blocks[&(i + 1)];
                        let args = regs_args!();
                        builder.ins().brif(cond, fall, &args, tb, &args);
                        dead = true;
                    }
                    Instruction::WithIdx(Opcode::Jmp, jidx) => {
                        let t = bc.nums.get(*jidx).copied().unwrap_or(0.0) as usize;
                        let tb = blocks[&t];
                        let args = regs_args!();
                        builder.ins().jump(tb, &args);
                        dead = true;
                    }
                    Instruction::WithIdx(Opcode::JmpIf, tidx) => {
                        // v3.5.34: pop + truthiness por el MISMO shim del
                        // intérprete (acepta Bool/Int/etc.), salto nativo.
                        let call = builder.ins().call(r_truth, &[vm_ptr]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        builder.ensure_inserted_block();
                        let fall = blocks[&(i + 1)];
                        let t = bc.nums.get(*tidx).copied().unwrap_or(0.0) as usize;
                        let target = blocks[&t];
                        // truthy → cae (i+1); falsy → salta al destino
                        let args = regs_args!();
                        builder.ins().brif(r, fall, &args, target, &args);
                        dead = true;
                    }
                    Instruction::WithNum(Opcode::JmpIf, n) => {
                        let call = builder.ins().call(r_truth, &[vm_ptr]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::SignedLessThan, r, 0);
                        let cont = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], cont, &[]);
                        builder.switch_to_block(cont);
                        builder.ensure_inserted_block();
                        let fall = blocks[&(i + 1)];
                        let target = blocks[&(*n as usize)];
                        let args = regs_args!();
                        builder.ins().brif(r, fall, &args, target, &args);
                        dead = true;
                    }
                    Instruction::WithIdx(Opcode::Load, nidx) => {
                        // v3.5.31: Load general — nativo si el nombre
                        // participa en aritmética/comparación Int (validado
                        // por el pre-scan) O si el análisis de etiquetas lo
                        // demuestra Int (v3.5.37) y no hay scopes interiores
                        // que puedan sombrearlo. Si no, delega al shim.
                        let tag_int =
                            !has_scope_push && name_tags.get(nidx).copied() == Some(VTag::Int);
                        // v3.5.37: slots[nidx] debe EXISTIR (resuelto por el
                        // prólogo); si no, shim (un HashMap index panic
                        // mataría la VM en cuerpos con nombres solo-Store).
                        if (read_names.contains(nidx) || tag_int) && slots.contains_key(nidx) {
                            let v = reg_load!(nidx);
                            push_int!(v);
                        } else {
                            let iv = builder.ins().iconst(i64t, *nidx as i64);
                            let opv = builder.ins().iconst(i64t, Opcode::Load as i64);
                            let ipv = builder.ins().iconst(i64t, i as i64);
                            let call = builder.ins().call(r_with_idx, &[vm_ptr, opv, iv, ipv]);
                            let r = builder.inst_results(call)[0];
                            let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                            let c = builder.create_block();
                            builder.ins().brif(bad, err_block, &[], c, &[]);
                            builder.switch_to_block(c);
                            builder.ensure_inserted_block();
                        }
                    }
                    Instruction::WithIdx(Opcode::ArrayNew, aidx) => {
                        // Par ArrayNew+StoreLocal (validado): shims.
                        let iv = builder.ins().iconst(i64t, *aidx as i64);
                        let opv = builder.ins().iconst(i64t, Opcode::ArrayNew as i64);
                        let ipv = builder.ins().iconst(i64t, i as i64);
                        let call = builder.ins().call(r_with_idx, &[vm_ptr, opv, iv, ipv]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        let c = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], c, &[]);
                        builder.switch_to_block(c);
                        builder.ensure_inserted_block();
                        if let Instruction::WithIdx(Opcode::StoreLocal, snidx) = &instrs[i + 1] {
                            let sv = builder.ins().iconst(i64t, *snidx as i64);
                            let opv2 = builder.ins().iconst(i64t, Opcode::StoreLocal as i64);
                            let ipv2 = builder.ins().iconst(i64t, (i + 1) as i64);
                            let call2 = builder.ins().call(r_with_idx, &[vm_ptr, opv2, sv, ipv2]);
                            let r2 = builder.inst_results(call2)[0];
                            let bad2 = builder.ins().icmp_imm(IntCC::NotEqual, r2, 0);
                            let c2 = builder.create_block();
                            builder.ins().brif(bad2, err_block, &[], c2, &[]);
                            builder.switch_to_block(c2);
                            builder.ensure_inserted_block();
                            // v3.5.34: nombre nuevo → alloc_slot → base fresca.
                            flat = refetch_flat!();
                            i += 1;
                        }
                    }
                    Instruction::WithIdx(Opcode::ArrayPushVar, idx) => {
                        let iv = builder.ins().iconst(i64t, *idx as i64);
                        let opv = builder.ins().iconst(i64t, Opcode::ArrayPushVar as i64);
                        let ipv = builder.ins().iconst(i64t, i as i64);
                        let call = builder.ins().call(r_with_idx, &[vm_ptr, opv, iv, ipv]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        let c = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], c, &[]);
                        builder.switch_to_block(c);
                        builder.ensure_inserted_block();
                    }
                    // v3.5.40: ArraySetVar — mismo patrón de delegación que
                    // ArrayPushVar (el handler muta el slot in-place O(1)).
                    Instruction::WithIdx(Opcode::ArraySetVar, idx) => {
                        let iv = builder.ins().iconst(i64t, *idx as i64);
                        let opv = builder.ins().iconst(i64t, Opcode::ArraySetVar as i64);
                        let ipv = builder.ins().iconst(i64t, i as i64);
                        let call = builder.ins().call(r_with_idx, &[vm_ptr, opv, iv, ipv]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        let c = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], c, &[]);
                        builder.switch_to_block(c);
                        builder.ensure_inserted_block();
                    }
                    Instruction::WithIdx(Opcode::Store, idx) => {
                        // v3.5.37: Store NATIVO si el análisis demuestra que
                        // el nombre es Int, su slot se resolvió en el
                        // prólogo (pair StoreLocal) y no hay scopes
                        // interiores que lo sombreen ni escrituras dinámicas.
                        let tag_int = !has_scope_push
                            && name_tags.get(idx).copied() == Some(VTag::Int)
                            && store_names.contains(idx)
                            && !dyn_written.contains(idx)
                            && slots.contains_key(idx);
                        if tag_int {
                            let v = pop_int!();
                            store_int!(slots[idx], v);
                        } else {
                            let iv = builder.ins().iconst(i64t, *idx as i64);
                            let opv = builder.ins().iconst(i64t, Opcode::Store as i64);
                            let ipv = builder.ins().iconst(i64t, i as i64);
                            let call = builder.ins().call(r_with_idx, &[vm_ptr, opv, iv, ipv]);
                            let r = builder.inst_results(call)[0];
                            let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                            let c = builder.create_block();
                            builder.ins().brif(bad, err_block, &[], c, &[]);
                            builder.switch_to_block(c);
                            builder.ensure_inserted_block();
                            // v3.5.34: nombre faltante → alloc_slot → base fresca.
                            flat = refetch_flat!();
                        }
                    }
                    Instruction::Simple(Opcode::ArrayGet) => {
                        let opv = builder.ins().iconst(i64t, Opcode::ArrayGet as i64);
                        let ipv = builder.ins().iconst(i64t, i as i64);
                        let call = builder.ins().call(r_simple, &[vm_ptr, opv, ipv]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        let c = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], c, &[]);
                        builder.switch_to_block(c);
                        builder.ensure_inserted_block();
                    }
                    Instruction::Simple(Opcode::Eq) => {
                        // v3.5.40: comparación polimórfica — shim genérico
                        // (el handler de la VM resuelve Int/Str/etc.). Los
                        // operandos viven en la pila real (Load shim).
                        let opv = builder.ins().iconst(i64t, Opcode::Eq as i64);
                        let ipv = builder.ins().iconst(i64t, i as i64);
                        let call = builder.ins().call(r_simple, &[vm_ptr, opv, ipv]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        let c = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], c, &[]);
                        builder.switch_to_block(c);
                        builder.ensure_inserted_block();
                    }
                    Instruction::Simple(op @ (Opcode::Add | Opcode::Sub | Opcode::Mul)) => {
                        if dyn_arith {
                            // v3.5.37: etiquetas ESTÁTICAS de los operandos.
                            let tags = stack_tags.get(&i).cloned().unwrap_or_default();
                            let bt = tags
                                .get(tags.len().wrapping_sub(1))
                                .copied()
                                .unwrap_or(VTag::Any);
                            let at = tags
                                .get(tags.len().wrapping_sub(2))
                                .copied()
                                .unwrap_or(VTag::Any);
                            if at == VTag::Int && bt == VTag::Int {
                                // aritmética nativa (pop b, pop a, op, push).
                                let bv = pop_int!();
                                let av = pop_int!();
                                let res = match op {
                                    Opcode::Add => builder.ins().iadd(av, bv),
                                    Opcode::Sub => builder.ins().isub(av, bv),
                                    _ => builder.ins().imul(av, bv),
                                };
                                push_int!(res);
                            } else if *op == Opcode::Add && (at == VTag::Str || bt == VTag::Str) {
                                // concat rápido: un shim reproduce el arm
                                // Add del intérprete exactamente.
                                let call = builder.ins().call(r_concat, &[vm_ptr]);
                                let r = builder.inst_results(call)[0];
                                let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                                let c = builder.create_block();
                                builder.ins().brif(bad, err_block, &[], c, &[]);
                                builder.switch_to_block(c);
                                builder.ensure_inserted_block();
                            } else {
                                // mixtos/desconocidos → shim genérico.
                                let opv = builder.ins().iconst(i64t, *op as i64);
                                let ipv = builder.ins().iconst(i64t, i as i64);
                                let call = builder.ins().call(r_simple, &[vm_ptr, opv, ipv]);
                                let r = builder.inst_results(call)[0];
                                let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                                let c = builder.create_block();
                                builder.ins().brif(bad, err_block, &[], c, &[]);
                                builder.switch_to_block(c);
                                builder.ensure_inserted_block();
                            }
                        } else {
                            // v3.5.31: aritmética de pila nativa (pop b,
                            // pop a, a op b, push). MIN → bail-out.
                            let bv = pop_int!();
                            let av = pop_int!();
                            let res = match op {
                                Opcode::Add => builder.ins().iadd(av, bv),
                                Opcode::Sub => builder.ins().isub(av, bv),
                                _ => builder.ins().imul(av, bv),
                            };
                            push_int!(res);
                        }
                    }
                    Instruction::WithIdx(Opcode::Call, nidx) => {
                        // v3.5.31: llamada dentro del cuerpo nativo; el
                        // resultado queda en la pila de la VM para las ops
                        // siguientes (el pre-scan valida el par Call/argc).
                        let argc = match &instrs[i + 1] {
                            Instruction::WithIdx(_, aidx) => {
                                bc.nums.get(*aidx).copied().unwrap_or(0.0) as i64
                            }
                            _ => 0,
                        };
                        // v3.5.39: INLINING — callee simple (sin llamadas
                        // ni scopes, lecturas todas promovibles). El cuerpo
                        // se compila inline en registros; la llamada
                        // desaparece. Si algo no aplica, shim de siempre.
                        let mut inlined = false;
                        if let Some((ics, ice, icprom, _icparams, iparam_order)) = try_inline_plan(
                            bc,
                            func_idx,
                            bc.names.get(*nidx).map(|s| s.as_str()).unwrap_or(""),
                        ) {
                            if iparam_order.len() as i64 == argc {
                                // Snapshot de los registros del caller para
                                // la continuación (no cambian en el inline).
                                let caller_args = regs_args!();
                                let cont_block = builder.create_block();
                                for _ in &promoted {
                                    builder.append_block_param(cont_block, i64t);
                                }
                                // Argumentos: pop×argc + reverse (mismo
                                // orden que el intérprete). Un pop no-Int
                                // hace bail → el frame se re-ejecuta entero.
                                let mut arg_vals: Vec<Value> = Vec::new();
                                for _ in 0..argc {
                                    arg_vals.push(pop_int!());
                                }
                                arg_vals.reverse();
                                emit_inline_body(
                                    &mut builder,
                                    bc,
                                    ics,
                                    ice,
                                    &icprom,
                                    &iparam_order,
                                    &arg_vals,
                                    &caller_args,
                                    cont_block,
                                    bail_block,
                                    err_block,
                                    i64t,
                                    vm_ptr,
                                    r_pushint,
                                    r_pushbool,
                                )?;
                                builder.switch_to_block(cont_block);
                                builder.ensure_inserted_block();
                                // Re-enlazar los registros del caller desde
                                // los params de la continuación.
                                if !promoted.is_empty() {
                                    let ps = builder.block_params(cont_block);
                                    let mut new_regs = HashMap::new();
                                    for (n, p) in promoted.iter().zip(ps.iter()) {
                                        new_regs.insert(*n, *p);
                                    }
                                    regs = new_regs;
                                }
                                // El inline no asigna slots → flat estable.
                                inlined = true;
                                if std::env::var_os("LUMEN_JIT_LOG").is_some() {
                                    eprintln!(
                                        "[jit] 🔗 inlining de '{}' en '{}'",
                                        bc.names.get(*nidx).map(|s| s.as_str()).unwrap_or(""),
                                        bc.funcs
                                            .get(func_idx)
                                            .map(|f| f.name.as_str())
                                            .unwrap_or("")
                                    );
                                }
                            }
                        }
                        if !inlined {
                            let nv = builder.ins().iconst(i64t, *nidx as i64);
                            let av = builder.ins().iconst(i64t, argc);
                            let ipv = builder.ins().iconst(i64t, (i + 1) as i64);
                            // v3.5.32: nombre NO builtin (set estático) → call
                            // rápido sin pre-filtro; builtin → ruta completa.
                            let is_builtin = crate::vm::builtin_name_set()
                                .contains(bc.names.get(*nidx).map(|s| s.as_str()).unwrap_or(""));
                            let f_use = if is_builtin { r_call } else { r_call_fast };
                            let call = builder.ins().call(f_use, &[vm_ptr, nv, av, ipv]);
                            let r = builder.inst_results(call)[0];
                            let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                            let c = builder.create_block();
                            builder.ins().brif(bad, err_block, &[], c, &[]);
                            builder.switch_to_block(c);
                            builder.ensure_inserted_block();
                            // v3.5.34: una llamada a función de USUARIO asigna
                            // slots (scope de parámetros) → base fresca. Los
                            // builtins no asignan slots.
                            if !is_builtin {
                                flat = refetch_flat!();
                            }
                        }
                        i += 1; // consumir el marcador de argc
                    }
                    Instruction::Simple(Opcode::Ret) => {
                        // v3.5.31: Ret suelto — el resultado YA está en la
                        // pila (aritmética de pila / Load / PushInt).
                        let call = builder.ins().call(r_ret, &[vm_ptr]);
                        let r = builder.inst_results(call)[0];
                        let bad = builder.ins().icmp_imm(IntCC::NotEqual, r, 0);
                        let okb = builder.create_block();
                        builder.ins().brif(bad, err_block, &[], okb, &[]);
                        builder.switch_to_block(okb);
                        let zero = builder.ins().iconst(i64t, 0);
                        builder.ins().return_(&[zero]);
                        dead = true;
                    }
                    Instruction::WithIdx(Opcode::Nop, _) => {}
                    _ => {
                        return Err("instrucción no elegible en el cuerpo".into());
                    }
                }
                i += 1;
            }
            // Cierre del cuerpo (bloque final inalcanzable si hay Ret).
            if !dead {
                match blocks.get(&end).copied() {
                    Some(eb) => {
                        let args = regs_args!();
                        builder.ins().jump(eb, &args);
                    }
                    None => {
                        let zero = builder.ins().iconst(i64t, 0);
                        builder.ins().return_(&[zero]);
                    }
                }
            }
            if let Some(eb) = blocks.get(&end).copied() {
                builder.switch_to_block(eb);
                builder.ensure_inserted_block();
                let zero = builder.ins().iconst(i64t, 0);
                builder.ins().return_(&[zero]);
            }

            builder.seal_all_blocks();
            builder.finalize();
        }

        {
            let vflags = settings::Flags::new(settings::builder());
            if let Err(ve) = cranelift::codegen::verify_function(&ctx.func, &vflags) {
                if std::env::var_os("LUMEN_JIT_LOG").is_some() {
                    eprintln!("=== IR ROTO (Tier-2) ===\n{}", ctx.func.display());
                }
                return Err(format!("verifier: {}", ve));
            }
        }
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(&mut ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;
        let code = self.module.get_finalized_function(func_id);
        Ok(unsafe { std::mem::transmute::<*const u8, JitFn>(code) })
    }

    /// v3.5.34 (Tier-R): recursión auto-nativa EN REGISTROS para funciones
    /// auto-recursivas puras de enteros (el caso `fib`). El cuerpo se
    /// compila como una función CLIF recursiva que pasa el parámetro por
    /// registro y NO toca la pila de la VM: cada nivel de recursión deja de
    /// pagar frame, shims y tráfico de pila (antes ≈ 5 shims + 10 ops de
    /// pila por nivel).
    ///
    /// Elegibilidad ESTRICTA (si no → Err y decide el Tier-2 genérico):
    /// 1 parámetro (Int); cuerpo de solo {FusedCmpKJmp, Load del parámetro,
    /// PushInt, Add/Sub/Mul, Jmp, Call auto (argc=1), Nop, Ret}; pila de
    /// valores VACÍA en cada salto y en cada fallthrough a destino de salto;
    /// el cuerpo termina en Ret. El límite de profundidad
    /// (MAX_CALL_STACK_DEPTH) se replica con un contador: al superarlo se
    /// devuelve código 2 → el intérprete re-ejecuta el MISMO frame y produce
    /// el mismo error de desbordamiento (paridad exacta).
    fn try_compile_recursive(&mut self, bc: &Bytecode, func_idx: usize) -> Result<JitFn, String> {
        let layout = match &self.tier2 {
            Some(l) => l,
            None => return Err("rec: layout de Value no disponible".into()),
        };
        let (lsize, lpayload_off) = (layout.size, layout.payload_off);
        let (start, end) = body_range(bc, func_idx);
        if end <= start || end - start > MAX_JIT_BODY {
            return Err("rec: cuerpo fuera de límites".into());
        }
        let fi = &bc.funcs[func_idx];
        if fi.params.len() != 1 {
            return Err("rec: se requiere exactamente 1 parámetro".into());
        }
        let pname = &fi.params[0];
        let pidx = bc
            .names
            .iter()
            .position(|n| n == pname)
            .ok_or_else(|| "rec: parámetro sin índice de nombre".to_string())?;
        let self_nidx = bc
            .names
            .iter()
            .position(|n| *n == fi.name)
            .ok_or_else(|| "rec: función sin índice de nombre".to_string())?;
        let instrs = &bc.instructions;

        // ── Pre-scan estricto + simulación de altura de pila ──
        let mut height: i64 = 0;
        let mut n_calls = 0usize;
        let mut n_rets = 0usize;
        let mut jump_targets: BTreeSet<usize> = BTreeSet::new();
        let mut i = start;
        while i < end {
            let mut falls = true;
            match &instrs[i] {
                Instruction::FusedCmpKJmp {
                    op,
                    a,
                    k: _,
                    target,
                } => {
                    if !(7..=12).contains(op) {
                        return Err("rec: op cmp no-Int".into());
                    }
                    if *a != pidx {
                        return Err("rec: operando cmp no es el parámetro".into());
                    }
                    let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("rec: salto cmp fuera del cuerpo".into());
                    }
                    if height != 0 {
                        return Err("rec: pila no vacía en salto cmp".into());
                    }
                    jump_targets.insert(t);
                    falls = false;
                }
                Instruction::WithIdx(Opcode::Jmp, tidx) => {
                    let t = bc.nums.get(*tidx).copied().unwrap_or(0.0) as usize;
                    if !(start..end).contains(&t) {
                        return Err("rec: Jmp fuera del cuerpo".into());
                    }
                    if height != 0 {
                        return Err("rec: pila no vacía en Jmp".into());
                    }
                    jump_targets.insert(t);
                    falls = false;
                }
                Instruction::WithIdx(Opcode::Load, nidx) => {
                    if *nidx != pidx {
                        return Err("rec: Load de nombre no-parámetro".into());
                    }
                    height += 1;
                }
                Instruction::WithIdx(Opcode::PushInt, _) => height += 1,
                Instruction::Simple(Opcode::Add | Opcode::Sub | Opcode::Mul) => height -= 1,
                Instruction::WithIdx(Opcode::Call, nidx) => {
                    if *nidx != self_nidx {
                        return Err("rec: llamada no-auto".into());
                    }
                    let argc = match instrs.get(i + 1) {
                        Some(Instruction::WithIdx(_, aidx)) => {
                            bc.nums.get(*aidx).copied().unwrap_or(0.0) as i64
                        }
                        _ => return Err("rec: Call sin marcador argc".into()),
                    };
                    if argc != 1 {
                        return Err("rec: argc != 1".into());
                    }
                    height = height - 1 + 1;
                    n_calls += 1;
                    i += 1; // saltar el marcador argc
                }
                Instruction::Simple(Opcode::Ret) => {
                    if height != 1 {
                        return Err("rec: Ret con pila != 1".into());
                    }
                    height -= 1;
                    n_rets += 1;
                    falls = false;
                }
                Instruction::WithIdx(Opcode::Nop, _) => {}
                _ => return Err("rec: instrucción no elegible".into()),
            }
            if height < 0 {
                return Err("rec: pila negativa".into());
            }
            if falls && i + 1 < end && jump_targets.contains(&(i + 1)) && height != 0 {
                return Err("rec: fallthrough a destino de salto con pila no vacía".into());
            }
            i += 1;
        }
        if n_calls == 0 || n_rets == 0 {
            return Err("rec: sin auto-llamada o sin Ret".into());
        }
        if !matches!(instrs.get(end - 1), Some(Instruction::Simple(Opcode::Ret))) {
            return Err("rec: el cuerpo no termina en Ret".into());
        }

        // ── Firmas e imports ──
        let i64t = types::I64;
        let mut sig_w = self.module.make_signature(); // (vm) -> i64
        sig_w.params.push(AbiParam::new(i64t));
        sig_w.returns.push(AbiParam::new(i64t));
        let mut sig_r = self.module.make_signature(); // (vm, n, depth) -> (val, ok)
        for _ in 0..3 {
            sig_r.params.push(AbiParam::new(i64t));
        }
        sig_r.returns.push(AbiParam::new(i64t));
        sig_r.returns.push(AbiParam::new(i64t));
        let mut sig_1 = self.module.make_signature(); // (i64) -> i64
        sig_1.params.push(AbiParam::new(i64t));
        sig_1.returns.push(AbiParam::new(i64t));
        let mut sig_2 = self.module.make_signature(); // (i64, i64) -> i64
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.returns.push(AbiParam::new(i64t));

        let decl = |module: &mut JITModule, name: &str, sig: &Signature| {
            module
                .declare_function(name, Linkage::Import, sig)
                .map_err(|e| e.to_string())
        };
        let f_probe = decl(&mut self.module, "lj_probe_int", &sig_2)?;
        let f_flat = decl(&mut self.module, "lj_flat_ptr", &sig_1)?;
        let f_push = decl(&mut self.module, "lj_push_int", &sig_2)?;
        let f_ret = decl(&mut self.module, "lj_ret", &sig_1)?;

        // ── Funciones destino (wrapper + recursiva) ──
        self.counter += 1;
        let wname = format!("lumen_jitr_{}_{}", func_idx, self.counter);
        let w_id = self
            .module
            .declare_function(&wname, Linkage::Export, &sig_w)
            .map_err(|e| e.to_string())?;
        self.counter += 1;
        let rname = format!("lumen_jitr_{}_{}_rec", func_idx, self.counter);
        let r_id = self
            .module
            .declare_function(&rname, Linkage::Export, &sig_r)
            .map_err(|e| e.to_string())?;

        // ── Cuerpo recursivo (registros puros) ──
        let mut ctx_r = self.module.make_context();
        ctx_r.func.signature = sig_r.clone();
        ctx_r.func.name = cranelift::codegen::ir::UserFuncName::user(0, r_id.as_u32());
        {
            let mut builder =
                cranelift::frontend::FunctionBuilder::new(&mut ctx_r.func, &mut self.fbc);
            // auto-declaración para la recursión directa
            let r_rec = self.module.declare_func_in_func(r_id, builder.func);

            let entry_r = builder.create_block();
            builder.switch_to_block(entry_r);
            builder.append_block_param(entry_r, i64t); // vm
            builder.append_block_param(entry_r, i64t); // n
            builder.append_block_param(entry_r, i64t); // depth
            let vmr = builder.block_params(entry_r)[0];
            let nr = builder.block_params(entry_r)[1];
            let dr = builder.block_params(entry_r)[2];

            // guarda de profundidad — paridad con MAX_CALL_STACK_DEPTH del
            // intérprete (el frame nativo ya ocupa el nivel 1).
            let lim = builder
                .ins()
                .iconst(i64t, (crate::vm::MAX_CALL_STACK_DEPTH - 1) as i64);
            let over = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, dr, lim);

            let mut blocks: BTreeMap<usize, Block> = BTreeMap::new();
            for k in start..end {
                let b = builder.create_block();
                builder.append_block_param(b, i64t);
                builder.append_block_param(b, i64t);
                builder.append_block_param(b, i64t);
                blocks.insert(k, b);
            }
            let fail_block = builder.create_block();

            builder.ins().brif(
                over,
                fail_block,
                &[],
                blocks[&start],
                &[
                    BlockArg::Value(vmr),
                    BlockArg::Value(nr),
                    BlockArg::Value(dr),
                ],
            );

            // fallo (profundidad límite) → (0, 0) → el wrapper devuelve 2.
            builder.switch_to_block(fail_block);
            builder.ensure_inserted_block();
            let zero = builder.ins().iconst(i64t, 0);
            builder.ins().return_(&[zero, zero]);

            // ── cuerpo ──
            let mut stk: Vec<Value> = Vec::new();
            let mut i = start;
            while i < end {
                builder.switch_to_block(blocks[&i]);
                builder.ensure_inserted_block();
                let vm = builder.block_params(blocks[&i])[0];
                let n = builder.block_params(blocks[&i])[1];
                let depth = builder.block_params(blocks[&i])[2];
                let mut dead = false;
                match &instrs[i] {
                    Instruction::WithIdx(Opcode::PushInt, idx) => {
                        let k = bc.ints.get(*idx).copied().unwrap_or(0);
                        let kv = builder.ins().iconst(i64t, k);
                        stk.push(kv);
                    }
                    Instruction::WithIdx(Opcode::Load, _) => {
                        // pre-scan: solo Loads del parámetro → el registro.
                        stk.push(n);
                    }
                    Instruction::Simple(op @ (Opcode::Add | Opcode::Sub | Opcode::Mul)) => {
                        let b = stk.pop().unwrap();
                        let a = stk.pop().unwrap();
                        let res = match op {
                            Opcode::Add => builder.ins().iadd(a, b),
                            Opcode::Sub => builder.ins().isub(a, b),
                            _ => builder.ins().imul(a, b),
                        };
                        stk.push(res);
                    }
                    Instruction::WithIdx(Opcode::Call, _) => {
                        let arg = stk.pop().unwrap();
                        let onec = builder.ins().iconst(i64t, 1);
                        let d1 = builder.ins().iadd(depth, onec);
                        let call = builder.ins().call(r_rec, &[vm, arg, d1]);
                        let val = builder.inst_results(call)[0];
                        let ok = builder.inst_results(call)[1];
                        let is_bad = builder.ins().icmp_imm(IntCC::Equal, ok, 0);
                        let cont = builder.create_block();
                        builder.append_block_param(cont, i64t);
                        builder
                            .ins()
                            .brif(is_bad, fail_block, &[], cont, &[BlockArg::Value(val)]);
                        builder.switch_to_block(cont);
                        builder.ensure_inserted_block();
                        stk.push(builder.block_params(cont)[0]);
                    }
                    Instruction::FusedCmpKJmp {
                        op,
                        a: _,
                        k,
                        target,
                    } => {
                        let t = bc.nums.get(*target).copied().unwrap_or(0.0) as usize;
                        let kv = builder.ins().iconst(i64t, *k);
                        let cond = match op {
                            7 => builder.ins().icmp(IntCC::Equal, n, kv),
                            8 => builder.ins().icmp(IntCC::NotEqual, n, kv),
                            9 => builder.ins().icmp(IntCC::SignedLessThan, n, kv),
                            10 => builder.ins().icmp(IntCC::SignedLessThanOrEqual, n, kv),
                            11 => builder.ins().icmp(IntCC::SignedGreaterThan, n, kv),
                            _ => builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, n, kv),
                        };
                        let fb = blocks[&(i + 1)];
                        let tb = blocks[&t];
                        let args: &[BlockArg] = &[
                            BlockArg::Value(vm),
                            BlockArg::Value(n),
                            BlockArg::Value(depth),
                        ];
                        builder.ins().brif(cond, fb, args, tb, args);
                        dead = true;
                    }
                    Instruction::WithIdx(Opcode::Jmp, tidx) => {
                        let t = bc.nums.get(*tidx).copied().unwrap_or(0.0) as usize;
                        builder.ins().jump(
                            blocks[&t],
                            &[
                                BlockArg::Value(vm),
                                BlockArg::Value(n),
                                BlockArg::Value(depth),
                            ],
                        );
                        dead = true;
                    }
                    Instruction::Simple(Opcode::Ret) => {
                        let v = stk.pop().unwrap();
                        let one = builder.ins().iconst(i64t, 1);
                        builder.ins().return_(&[v, one]);
                        dead = true;
                    }
                    Instruction::WithIdx(Opcode::Nop, _) => {}
                    _ => unreachable!("pre-scan validado"),
                }
                if !dead {
                    if i + 1 < end {
                        builder.ins().jump(
                            blocks[&(i + 1)],
                            &[
                                BlockArg::Value(vm),
                                BlockArg::Value(n),
                                BlockArg::Value(depth),
                            ],
                        );
                    } else {
                        let zero2 = builder.ins().iconst(i64t, 0);
                        builder.ins().return_(&[zero2, zero2]);
                    }
                }
                i += 1;
            }
            builder.seal_all_blocks();
            builder.finalize();
        }

        // ── Wrapper (JitFn): probe → carga → rec → push + Ret ──
        let mut ctx_w = self.module.make_context();
        ctx_w.func.signature = sig_w.clone();
        ctx_w.func.name = cranelift::codegen::ir::UserFuncName::user(0, w_id.as_u32());
        {
            let mut builder =
                cranelift::frontend::FunctionBuilder::new(&mut ctx_w.func, &mut self.fbc);
            let r_probe = self.module.declare_func_in_func(f_probe, builder.func);
            let r_flat = self.module.declare_func_in_func(f_flat, builder.func);
            let r_push = self.module.declare_func_in_func(f_push, builder.func);
            let r_ret = self.module.declare_func_in_func(f_ret, builder.func);
            let r_rec = self.module.declare_func_in_func(r_id, builder.func);

            let entry = builder.create_block();
            builder.switch_to_block(entry);
            builder.append_block_param(entry, i64t);
            let vm_ptr = builder.block_params(entry)[0];

            let err_block = builder.create_block();
            let bail_block = builder.create_block();
            let pro = builder.create_block();

            builder.ensure_inserted_block();
            builder.ins().jump(pro, &[]);
            // Epílogos: error → 1, bail → 2 (mismo contrato que el Tier-2).
            builder.switch_to_block(err_block);
            let one = builder.ins().iconst(i64t, 1);
            builder.ins().return_(&[one]);
            builder.switch_to_block(bail_block);
            let two = builder.ins().iconst(i64t, 2);
            builder.ins().return_(&[two]);
            builder.switch_to_block(pro);
            builder.ensure_inserted_block();

            // probe del parámetro (existe + es Int) → si no, bail-out.
            let pv = builder.ins().iconst(i64t, pidx as i64);
            let callp = builder.ins().call(r_probe, &[vm_ptr, pv]);
            let slot = builder.inst_results(callp)[0];
            let bad = builder.ins().icmp_imm(IntCC::SignedLessThan, slot, 0);
            let c1 = builder.create_block();
            builder.ins().brif(bad, bail_block, &[], c1, &[]);
            builder.switch_to_block(c1);
            builder.ensure_inserted_block();

            // carga del parámetro desde el slot (flat).
            let callf = builder.ins().call(r_flat, &[vm_ptr]);
            let flat = builder.inst_results(callf)[0];
            let ssize = builder.ins().iconst(i64t, lsize as i64);
            let soff = builder.ins().iconst(i64t, lpayload_off as i64);
            let mul = builder.ins().imul(slot, ssize);
            let base = builder.ins().iadd(flat, mul);
            let pa = builder.ins().iadd(base, soff);
            let nval = builder
                .ins()
                .load(i64t, cranelift::codegen::ir::MemFlags::trusted(), pa, 0);

            // llamada recursiva (n, profundidad 0).
            let d0 = builder.ins().iconst(i64t, 0);
            let rcall = builder.ins().call(r_rec, &[vm_ptr, nval, d0]);
            let val = builder.inst_results(rcall)[0];
            let ok = builder.inst_results(rcall)[1];
            let is_bad = builder.ins().icmp_imm(IntCC::Equal, ok, 0);
            let c2 = builder.create_block();
            builder.ins().brif(is_bad, bail_block, &[], c2, &[]);
            builder.switch_to_block(c2);
            builder.ensure_inserted_block();

            // epílogo idéntico al Tier-2: push del resultado + Ret de la VM.
            let callp2 = builder.ins().call(r_push, &[vm_ptr, val]);
            let rp = builder.inst_results(callp2)[0];
            let bad2 = builder.ins().icmp_imm(IntCC::NotEqual, rp, 0);
            let c3 = builder.create_block();
            builder.ins().brif(bad2, err_block, &[], c3, &[]);
            builder.switch_to_block(c3);
            builder.ensure_inserted_block();
            let callr2 = builder.ins().call(r_ret, &[vm_ptr]);
            let rr = builder.inst_results(callr2)[0];
            let bad3 = builder.ins().icmp_imm(IntCC::NotEqual, rr, 0);
            let c4 = builder.create_block();
            builder.ins().brif(bad3, err_block, &[], c4, &[]);
            builder.switch_to_block(c4);
            builder.ensure_inserted_block();
            let zero = builder.ins().iconst(i64t, 0);
            builder.ins().return_(&[zero]);

            builder.seal_all_blocks();
            builder.finalize();
        }

        // ── Verificación y registro ──
        {
            let vflags = settings::Flags::new(settings::builder());
            if let Err(ve) = cranelift::codegen::verify_function(&ctx_r.func, &vflags) {
                return Err(format!("rec: verifier (rec): {}", ve));
            }
            if let Err(ve) = cranelift::codegen::verify_function(&ctx_w.func, &vflags) {
                return Err(format!("rec: verifier (wrapper): {}", ve));
            }
        }
        self.module
            .define_function(r_id, &mut ctx_r)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(&mut ctx_r);
        self.module
            .define_function(w_id, &mut ctx_w)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(&mut ctx_w);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;
        let code = self.module.get_finalized_function(w_id);
        Ok(unsafe { std::mem::transmute::<*const u8, JitFn>(code) })
    }
}

/// Rango [start, end) del cuerpo de una función: desde su `start` hasta el
/// `start` de la siguiente función registrada (o el fin del bytecode).
fn body_range(bc: &Bytecode, func_idx: usize) -> (usize, usize) {
    let start = bc.funcs[func_idx].start;
    let end = bc
        .funcs
        .iter()
        .map(|f| f.start)
        .filter(|&s| s > start)
        .min()
        .unwrap_or(bc.instructions.len());
    (start, end)
}

/// v3.5.31: ¿el cuerpo de la función contiene un salto HACIA ATRÁS (bucle)?
/// Un bucle domina el tiempo aunque la función se llame una sola vez, así que
/// el JIT la compila en la primera llamada (ver `VM::jit_maybe_compile`).
pub fn body_has_loop(bc: &Bytecode, func_idx: usize) -> bool {
    let (start, end) = body_range(bc, func_idx);
    let mut i = start;
    while i < end {
        let t = match bc.instructions.get(i) {
            Some(Instruction::WithNum(Opcode::Jmp, n))
            | Some(Instruction::WithNum(Opcode::JmpIf, n)) => Some(*n as usize),
            Some(Instruction::WithIdx(Opcode::Jmp, idx))
            | Some(Instruction::WithIdx(Opcode::JmpIf, idx)) => {
                Some(bc.nums.get(*idx).copied().unwrap_or(0.0) as usize)
            }
            Some(Instruction::FusedCmpKJmp { target, .. })
            | Some(Instruction::FusedCmpJmp { target, .. }) => {
                Some(bc.nums.get(*target).copied().unwrap_or(0.0) as usize)
            }
            _ => None,
        };
        if let Some(t) = t {
            if t < i {
                return true;
            }
        }
        i += 1;
    }
    false
}
