//! JIT Tier-1 de LÚMEN (v3.5.9) — bytecode caliente → código nativo Cranelift.
//!
//! Diseño "correcto por construcción": el código nativo NO reimplementa ninguna
//! semántica; cada opcode se delega a los MISMOS handlers del intérprete a
//! través de helpers `extern "C"` (lj_*). El JIT solo elimina el costo de
//! dispatch/decodificación y ejecuta los saltos (Jmp/JmpIf/Ret) de forma nativa.
//!
//! Gateado por la variable de entorno `LUMEN_JIT=1` (por defecto APAGADO para
//! que el fixpoint self-hosting siga corriendo en el intérprete puro).

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use lumen_codegen::bytecode::{Bytecode, Instruction, Opcode};
use std::collections::{BTreeSet, HashMap};

use crate::vm::{VmError, VM};

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
pub extern "C" fn lj_simple(p: *mut std::ffi::c_void, op: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
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
pub extern "C" fn lj_with_num(p: *mut std::ffi::c_void, op: i64, n: f64) -> i64 {
    let vm = unsafe { vm_ref(p) };
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
pub extern "C" fn lj_with_bool(p: *mut std::ffi::c_void, op: i64, b: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
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
pub extern "C" fn lj_with_idx(p: *mut std::ffi::c_void, op: i64, idx: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
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
pub extern "C" fn lj_with_str(p: *mut std::ffi::c_void, op: i64, ptr: i64, len: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
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
pub extern "C" fn lj_call(p: *mut std::ffi::c_void, nptr: i64, nlen: i64, argc: i64) -> i64 {
    let vm = unsafe { vm_ref(p) };
    let bytes = unsafe { std::slice::from_raw_parts(nptr as *const u8, nlen as usize) };
    let name = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => {
            vm.set_jit_error(VmError::Runtime("JIT: nombre inválido".into()));
            return 1;
        }
    };
    match vm.perform_call(name, argc as usize) {
        Ok(()) => 0,
        Err(e) => {
            vm.set_jit_error(e);
            1
        }
    }
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
// Motor JIT
// ──────────────────────────────────────────────────────────────────────

const MAX_JIT_BODY: usize = 60_000;

pub struct VmJit {
    fbc: cranelift::frontend::FunctionBuilderContext,
    module: JITModule,
    /// Signaturas + FuncRefs de los helpers (importados por función compilada).
    cache: HashMap<usize, JitFn>,
    failed: std::collections::HashSet<usize>,
    counter: usize,
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
        builder.symbol("lj_truth", lj_truth as *const u8);
        builder.symbol("lj_ret", lj_ret as *const u8);
        let module = JITModule::new(builder);
        Ok(Self {
            fbc: cranelift::frontend::FunctionBuilderContext::new(),
            module,
            cache: HashMap::new(),
            failed: std::collections::HashSet::new(),
            counter: 0,
        })
    }

    pub fn get(&self, func_idx: usize) -> Option<JitFn> {
        self.cache.get(&func_idx).copied()
    }

    pub fn should_try(&self, func_idx: usize) -> bool {
        !self.failed.contains(&func_idx) && !self.cache.contains_key(&func_idx)
    }

    /// Intenta compilar la función `func_idx`. Si no pertenece al subconjunto
    /// seguro, la marca como fallida y no se reintenta.
    pub fn try_compile(&mut self, bc: &Bytecode, func_idx: usize) {
        if !self.should_try(func_idx) {
            return;
        }
        match self.compile(bc, func_idx) {
            Ok(f) => {
                self.cache.insert(func_idx, f);
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
                self.failed.insert(func_idx);
            }
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
                // v3.5.20: los super-opcodes los ejecuta la VM con sus
                // brazos rápidos (más veloces que delegar instr por instr).
                Instruction::FusedBinK { .. }
                | Instruction::FusedBin { .. }
                | Instruction::FusedCmpKJmp { .. }
                | Instruction::FusedCmpJmp { .. } => {
                    return Err("super-opcode: lo ejecuta la VM".into())
                }
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
        // WithNum directo o WithIdx → pool de nums).
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

        let mut sig_2 = self.module.make_signature(); // (vm, op)
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.params.push(AbiParam::new(i64t));
        sig_2.returns.push(AbiParam::new(i64t));

        let mut sig_wn = self.module.make_signature(); // (vm, op, f64)
        sig_wn.params.push(AbiParam::new(i64t));
        sig_wn.params.push(AbiParam::new(i64t));
        sig_wn.params.push(AbiParam::new(f64t));
        sig_wn.returns.push(AbiParam::new(i64t));

        let mut sig_ws = self.module.make_signature(); // (vm, op, ptr, len)
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.params.push(AbiParam::new(i64t));
        sig_ws.returns.push(AbiParam::new(i64t));

        let mut sig_call = self.module.make_signature(); // (vm, ptr, len, argc)
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.params.push(AbiParam::new(i64t));
        sig_call.returns.push(AbiParam::new(i64t));

        let mut sig_1 = self.module.make_signature(); // (vm)
        sig_1.params.push(AbiParam::new(i64t));
        sig_1.returns.push(AbiParam::new(i64t));

        let mut sig_3 = self.module.make_signature(); // (vm, op, idx)
        sig_3.params.push(AbiParam::new(i64t));
        sig_3.params.push(AbiParam::new(i64t));
        sig_3.params.push(AbiParam::new(i64t));
        sig_3.returns.push(AbiParam::new(i64t));

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
        let f_truth = decl(&mut self.module, "lj_truth", &sig_1)?;
        let f_ret = decl(&mut self.module, "lj_ret", &sig_1)?;

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
            let entry = builder.create_block();
            builder.switch_to_block(entry);
            builder.append_block_param(entry, i64t); // vm ptr
            builder.ensure_inserted_block();
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
            let r_truth = self.module.declare_func_in_func(f_truth, builder.func);
            let r_ret = self.module.declare_func_in_func(f_ret, builder.func);

            // Epílogo de error: return 1
            builder.switch_to_block(err_block);
            let one = builder.ins().iconst(i64t, 1);
            builder.ins().return_(&[one]);
            builder.switch_to_block(entry);
            // El trampolín salta al inicio del cuerpo
            builder.ins().jump(blocks[&start], &[]);

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
                        let call = builder.ins().call(r_simple, &[vm_ptr, opv]);
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
                        let call = builder.ins().call(r_wn, &[vm_ptr, opv, nv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::WithBool(op, b) => {
                        let bv = builder.ins().iconst(i64t, if *b { 1 } else { 0 });
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let call = builder.ins().call(r_wb, &[vm_ptr, opv, bv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    Instruction::WithStr(op, s) => {
                        let pv = builder.ins().iconst(i64t, s.as_ptr() as i64);
                        let lv = builder.ins().iconst(i64t, s.len() as i64);
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let call = builder.ins().call(r_ws, &[vm_ptr, opv, pv, lv]);
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
                        let name = bc.names.get(*nidx).map(|s| s.as_str()).unwrap_or("");
                        let argc = match &instrs[i + 1] {
                            Instruction::WithIdx(_, aidx) => {
                                bc.nums.get(*aidx).copied().unwrap_or(0.0) as i64
                            }
                            _ => 0,
                        };
                        let pv = builder.ins().iconst(i64t, name.as_ptr() as i64);
                        let lv = builder.ins().iconst(i64t, name.len() as i64);
                        let av = builder.ins().iconst(i64t, argc);
                        let call = builder.ins().call(r_call, &[vm_ptr, pv, lv, av]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                        i += 1; // consumir la instrucción argc
                    }
                    Instruction::WithIdx(op, idx) => {
                        let iv = builder.ins().iconst(i64t, *idx as i64);
                        let opv = builder.ins().iconst(i64t, *op as i64);
                        let call = builder.ins().call(r_wi, &[vm_ptr, opv, iv]);
                        let r = builder.inst_results(call)[0];
                        check!(r);
                    }
                    // v3.5.20: inalcanzable — el pre-scan rechaza funciones
                    // con super-opcodes (las ejecuta la VM directamente).
                    Instruction::FusedBinK { .. }
                    | Instruction::FusedBin { .. }
                    | Instruction::FusedCmpKJmp { .. }
                    | Instruction::FusedCmpJmp { .. } => {}
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
