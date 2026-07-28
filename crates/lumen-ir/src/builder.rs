use crate::ir::*;
use lumen_parser::ast::{BinOp, Decl, DeclOrStmt, Expr, Param, Stmt, Type, UnOp};
use std::collections::{HashMap, HashSet};

struct LoopLabels {
    break_label: usize,
    continue_label: usize,
    loop_name: Option<String>,
}

pub struct IRBuilder {
    program: crate::ir::Program,
    current_func: Option<String>,
    current_instrs: Vec<Instr>,
    temp_counter: usize,
    label_counter: usize,
    lambda_counter: usize,
    loop_labels: Vec<LoopLabels>,
    default_params: HashMap<String, Vec<Option<Expr>>>,
    fn_names: HashSet<String>,
    impl_method_map: HashMap<String, String>,
    capture_map: HashMap<String, String>,
    is_in_lambda: bool,
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            program: crate::ir::Program::new(),
            current_func: None,
            current_instrs: Vec::new(),
            temp_counter: 0,
            label_counter: 0,
            lambda_counter: 0,
            loop_labels: Vec::new(),
            default_params: HashMap::new(),
            fn_names: HashSet::new(),
            impl_method_map: HashMap::new(),
            capture_map: HashMap::new(),
            is_in_lambda: false,
        }
    }

    pub fn build(mut self, program: &[DeclOrStmt]) -> crate::ir::Program {
        let has_toplevel_code = program.iter().any(|node| {
            !matches!(
                node,
                DeclOrStmt::Decl(Decl::Function { .. })
                    | DeclOrStmt::Decl(Decl::Struct { .. })
                    | DeclOrStmt::Decl(Decl::Enum { .. })
                    | DeclOrStmt::Decl(Decl::Rasgo { .. })
                    | DeclOrStmt::Decl(Decl::ImplRasgo { .. })
            )
        });

        for node in program {
            if let DeclOrStmt::Decl(Decl::Function { name, params, .. }) = node {
                let func = Func {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    entry: 0,
                    instrs: Vec::new(),
                };
                self.program.funcs.insert(name.clone(), func);
                self.fn_names.insert(name.clone());
            }
        }

        // Collect impl methods
        for node in program {
            if let DeclOrStmt::Decl(Decl::ImplRasgo {
                trait_name,
                target_type,
                methods,
                ..
            }) = node
            {
                let type_name = match type_to_impl_name(target_type) {
                    Some(n) => n,
                    None => continue,
                };
                for method_decl in methods {
                    if let Decl::Function {
                        name, ref params, ..
                    } = method_decl
                    {
                        let mangled = format!("{}_{}_{}", type_name, trait_name, name);
                        let mut param_names: Vec<String> =
                            params.iter().map(|p| p.name.clone()).collect();
                        // Trait methods always have an implicit receiver (self)
                        if !param_names
                            .iter()
                            .any(|n| n == "self" || n == "yo" || n == "este")
                        {
                            param_names.insert(0, "self".to_string());
                        }
                        let func = Func {
                            name: mangled.clone(),
                            params: param_names,
                            entry: 0,
                            instrs: Vec::new(),
                        };
                        self.program.funcs.insert(mangled.clone(), func);
                        self.fn_names.insert(mangled.clone());
                        self.impl_method_map.insert(name.clone(), mangled);
                    }
                }
            }
        }

        for node in program {
            if let DeclOrStmt::Decl(Decl::Function { name, params, .. }) = node {
                let defaults: Vec<Option<Expr>> = params
                    .iter()
                    .map(|p| p.default.clone().map(|boxed| *boxed))
                    .collect();
                self.default_params.insert(name.clone(), defaults);
            }
        }

        if has_toplevel_code {
            let main_func = Func {
                name: "__main__".to_string(),
                params: Vec::new(),
                entry: 0,
                instrs: Vec::new(),
            };
            self.program.funcs.insert("__main__".to_string(), main_func);
            self.fn_names.insert("__main__".to_string());
            self.finalize_func(); // Guardar el estado inicial (vacío) de __main__
            self.current_func = Some("__main__".to_string()); // Volver a main
            self.current_instrs = self.program.funcs.get("__main__").unwrap().instrs.clone();
            // Cargar sus instrucciones
        }

        for node in program {
            self.gen_decl_or_stmt(node);
        }

        if self
            .current_instrs
            .last()
            .is_none_or(|i| !matches!(i, Instr::Halt))
        {
            self.emit(Instr::Halt);
        }

        if self.program.entry.is_empty() {
            self.program.entry = "__main__".to_string();
        }

        self.finalize_func();
        self.program.clone()
    }

    fn gen_decl_or_stmt(&mut self, node: &DeclOrStmt) {
        match node {
            DeclOrStmt::Decl(d) => self.gen_decl(d),
            DeclOrStmt::Stmt(s) => self.gen_stmt(s),
        }
    }

    fn gen_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Variable { name, init, .. } => {
                if let Some(init_expr) = init {
                    self.gen_expr(init_expr);
                    let resolved = self
                        .capture_map
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.clone());
                    self.emit(Instr::Store(resolved));
                }
            }
            Decl::Destructure { targets, init, .. } => {
                let temp = format!("__dt_{}", self.temp_counter);
                self.temp_counter += 1;
                self.gen_expr(init);
                self.emit(Instr::Store(temp.clone()));
                for (i, target) in targets.iter().enumerate() {
                    if target.name == "_" {
                        continue;
                    }
                    self.emit(Instr::Load(temp.clone()));
                    self.emit(Instr::TupleAccess(i));
                    self.emit(Instr::Store(target.name.clone()));
                }
            }
            Decl::Function { name, body, .. } => {
                self.finalize_func(); // Guarda las instrucciones de la función actual (que podría ser __main__) antes de cambiar de contexto!
                let prev_func_name = self.current_func.take();

                self.current_func = Some(name.clone());
                // Cargar instrucciones si esta función ya existía (por ejemplo, en un paso previo)
                self.current_instrs = self
                    .program
                    .funcs
                    .get(name)
                    .map(|f| f.instrs.clone())
                    .unwrap_or_default();
                self.temp_counter = 0;
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                if !self
                    .current_instrs
                    .iter()
                    .any(|i| matches!(i, Instr::Return))
                {
                    self.emit(Instr::Return);
                }
                self.finalize_func(); // Guardar las instrucciones de esta función

                // Restaurar el contexto anterior
                if let Some(prev_name) = prev_func_name {
                    self.current_func = Some(prev_name.clone());
                    // Cargar las instrucciones actualizadas de la función anterior
                    self.current_instrs = self
                        .program
                        .funcs
                        .get(&prev_name)
                        .map(|f| f.instrs.clone())
                        .unwrap_or_default();
                } else {
                    self.current_func = None;
                    self.current_instrs = Vec::new(); // Estado inicial si no había función anterior
                }
            }
            Decl::Struct { .. } => {
                // Struct declarations are collected during IR build setup
                // No code generation needed for the declaration itself
            }
            Decl::Enum { .. } => {
                // Enum declarations are collected during IR build setup
                // No code generation needed for the declaration itself
            }
            Decl::Rasgo { .. } => {
                // Trait declaration — no code generation
            }
            Decl::ImplRasgo {
                trait_name,
                target_type,
                methods,
                ..
            } => {
                let type_name = match type_to_impl_name(target_type) {
                    Some(n) => n,
                    None => return,
                };

                self.finalize_func(); // Guarda las instrucciones de la función actual (que podría ser __main__) antes de cambiar de contexto!
                let prev_func_name = self.current_func.take();

                for method_decl in methods {
                    if let Decl::Function { name, ref body, .. } = method_decl {
                        let mangled = format!("{}_{}_{}", type_name, trait_name, name);
                        self.current_func = Some(mangled.clone());
                        // Cargar instrucciones si este método ya existía
                        self.current_instrs = self
                            .program
                            .funcs
                            .get(&mangled)
                            .map(|f| f.instrs.clone())
                            .unwrap_or_default();
                        self.temp_counter = 0;
                        for node in body {
                            self.gen_decl_or_stmt(node);
                        }
                        if !self
                            .current_instrs
                            .iter()
                            .any(|i| matches!(i, Instr::Return))
                        {
                            self.emit(Instr::Return);
                        }
                        self.finalize_func(); // Guardar las instrucciones de este método
                    }
                }

                // Restaurar el contexto anterior
                if let Some(prev_name) = prev_func_name {
                    self.current_func = Some(prev_name.clone());
                    // Cargar las instrucciones actualizadas de la función anterior
                    self.current_instrs = self
                        .program
                        .funcs
                        .get(&prev_name)
                        .map(|f| f.instrs.clone())
                        .unwrap_or_default();
                } else {
                    self.current_func = None;
                    self.current_instrs = Vec::new(); // Estado inicial si no había función anterior
                }
            }
            Decl::Const { name, value, .. } => {
                self.gen_expr(value);
                let resolved = self
                    .capture_map
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone());
                self.emit(Instr::Store(resolved));
            }
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assignment { name, value, .. } => {
                self.gen_expr(value);
                let resolved = self
                    .capture_map
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone());
                self.emit(Instr::Store(resolved));
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let else_label = self.new_label();
                let end_label = self.new_label();
                self.gen_expr(condition);
                self.emit(Instr::JmpIf(else_label));
                for node in then_body {
                    self.gen_decl_or_stmt(node);
                }
                self.emit(Instr::Jmp(end_label));
                self.emit(Instr::Label(else_label));
                if let Some(else_body) = else_body {
                    for node in else_body {
                        self.gen_decl_or_stmt(node);
                    }
                }
                self.emit(Instr::Label(end_label));
            }
            Stmt::While {
                condition, body, ..
            } => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                self.emit(Instr::Label(start_label));
                self.gen_expr(condition);
                self.emit(Instr::JmpIf(end_label));
                self.loop_labels.push(LoopLabels {
                    break_label: end_label,
                    continue_label: start_label,
                    loop_name: None,
                });
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                self.loop_labels.pop();
                self.emit(Instr::Jmp(start_label));
                self.emit(Instr::Label(end_label));
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
                ..
            } => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                let continue_label = self.new_label();
                self.gen_decl(init);
                self.emit(Instr::Label(start_label));
                self.gen_expr(condition);
                self.emit(Instr::JmpIf(end_label));
                self.loop_labels.push(LoopLabels {
                    break_label: end_label,
                    continue_label,
                    loop_name: None,
                });
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                self.loop_labels.pop();
                self.emit(Instr::Label(continue_label));
                self.gen_stmt(update);
                self.emit(Instr::Jmp(start_label));
                self.emit(Instr::Label(end_label));
            }
            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    self.gen_expr(val);
                }
                self.emit(Instr::Return);
            }
            Stmt::FieldAssign {
                expr, field, value, ..
            } => {
                let var_name = match expr.as_ref() {
                    Expr::Ident { name, .. } => Some(name.clone()),
                    _ => None,
                };
                self.gen_expr(expr);
                self.emit(Instr::ConstStr(field.clone()));
                self.gen_expr(value);
                self.emit(Instr::StructSet);
                if let Some(name) = var_name {
                    self.emit(Instr::Store(name));
                }
            }
            Stmt::Expr { expr, .. } => {
                self.gen_expr(expr);
            }
            Stmt::Break { label, .. } => {
                let target = if let Some(ref lbl) = label {
                    self.loop_labels
                        .iter()
                        .rev()
                        .find(|ll| ll.loop_name.as_deref() == Some(lbl))
                        .map(|ll| ll.break_label)
                } else {
                    self.loop_labels.last().map(|ll| ll.break_label)
                };
                if let Some(t) = target {
                    self.emit(Instr::Jmp(t));
                }
            }
            Stmt::Continue { label, .. } => {
                let target = if let Some(ref lbl) = label {
                    self.loop_labels
                        .iter()
                        .rev()
                        .find(|ll| ll.loop_name.as_deref() == Some(lbl))
                        .map(|ll| ll.continue_label)
                } else {
                    self.loop_labels.last().map(|ll| ll.continue_label)
                };
                if let Some(t) = target {
                    self.emit(Instr::Jmp(t));
                }
            }
            Stmt::Match {
                expr,
                arms,
                default,
                ..
            } => {
                let end_label = self.new_label();
                let mut next_label = self.new_label();
                for arm in arms {
                    self.emit(Instr::Label(next_label));
                    self.gen_expr(expr);
                    self.gen_expr(&arm.value);
                    self.emit(Instr::Binary(Op::Equal));
                    next_label = self.new_label();
                    self.emit(Instr::JmpIf(next_label));
                    // Check OR pattern alternatives
                    for alt in &arm.alt_values {
                        self.gen_expr(expr);
                        self.gen_expr(alt);
                        self.emit(Instr::Binary(Op::Equal));
                        let match_label = self.new_label();
                        self.emit(Instr::JmpIf(match_label));
                        self.emit(Instr::Jmp(next_label));
                        self.emit(Instr::Label(match_label));
                    }
                    if let Some(ref guard_expr) = arm.guard {
                        self.gen_expr(guard_expr);
                        self.emit(Instr::JmpIf(next_label));
                    }
                    for node in &arm.body {
                        self.gen_decl_or_stmt(node);
                    }
                    self.emit(Instr::Jmp(end_label));
                }
                self.emit(Instr::Label(next_label));
                if let Some(default_body) = default {
                    for node in default_body {
                        self.gen_decl_or_stmt(node);
                    }
                }
                self.emit(Instr::Label(end_label));
            }
            Stmt::Block { stmts, .. } => {
                for node in stmts {
                    self.gen_decl_or_stmt(node);
                }
            }
            Stmt::ForEach {
                var_name,
                expr,
                body,
                ..
            } => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                let arr_temp = format!("__for_arr_{}", self.temp_counter);
                self.temp_counter += 1;
                let idx_temp = format!("__for_i_{}", self.temp_counter);
                self.temp_counter += 1;
                let len_temp = format!("__for_len_{}", self.temp_counter);
                self.temp_counter += 1;

                self.gen_expr(expr);
                self.emit(Instr::Store(arr_temp.clone()));
                self.emit(Instr::ConstInt(0));
                self.emit(Instr::Store(idx_temp.clone()));
                self.emit(Instr::Load(arr_temp.clone()));
                self.emit(Instr::ArrayLen);
                self.emit(Instr::Store(len_temp.clone()));
                self.emit(Instr::Label(start_label));
                self.emit(Instr::Load(idx_temp.clone()));
                self.emit(Instr::Load(len_temp.clone()));
                self.emit(Instr::Binary(Op::Less));
                self.emit(Instr::JmpIf(end_label));
                self.emit(Instr::Load(arr_temp.clone()));
                self.emit(Instr::Load(idx_temp.clone()));
                self.emit(Instr::ArrayGet);
                self.emit(Instr::Store(var_name.clone()));
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                self.emit(Instr::Load(idx_temp.clone()));
                self.emit(Instr::ConstInt(1));
                self.emit(Instr::Binary(Op::Add));
                self.emit(Instr::Store(idx_temp.clone()));
                self.emit(Instr::Jmp(start_label));
                self.emit(Instr::Label(end_label));
            }
            Stmt::Import { .. } => {}
            Stmt::IfLet {
                pattern,
                value,
                then_body,
                else_body,
                ..
            } => {
                self.gen_expr(value);
                self.gen_expr(pattern);
                self.emit(Instr::Binary(Op::Equal));
                let el = self.new_label();
                let end_l = self.new_label();
                self.emit(Instr::JmpIf(el));
                for n in then_body {
                    self.gen_decl_or_stmt(n);
                }
                self.emit(Instr::Jmp(end_l));
                self.emit(Instr::Label(el));
                if let Some(eb) = else_body {
                    for n in eb {
                        self.gen_decl_or_stmt(n);
                    }
                }
                self.emit(Instr::Label(end_l));
            }
            Stmt::GuardLet {
                pattern,
                value,
                else_body,
                ..
            } => {
                self.gen_expr(value);
                self.gen_expr(pattern);
                self.emit(Instr::Binary(Op::Equal));
                let ok_l = self.new_label();
                let else_l = self.new_label();
                self.emit(Instr::JmpIf(else_l));
                self.emit(Instr::Jmp(ok_l));
                self.emit(Instr::Label(else_l));
                for n in else_body {
                    self.gen_decl_or_stmt(n);
                }
                self.emit(Instr::Label(ok_l));
            }
            Stmt::Destructure { targets, value, .. } => {
                let temp = format!("__dt_{}", self.temp_counter);
                self.temp_counter += 1;
                self.gen_expr(value);
                self.emit(Instr::Store(temp.clone()));
                for (i, target) in targets.iter().enumerate() {
                    if target.name == "_" {
                        continue;
                    }
                    self.emit(Instr::Load(temp.clone()));
                    self.emit(Instr::TupleAccess(i));
                    self.emit(Instr::Store(target.name.clone()));
                }
            }
        }
    }

    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int { value, .. } => {
                self.emit(Instr::ConstInt(*value));
            }
            Expr::Float { value, .. } => {
                self.emit(Instr::ConstFloat(*value));
            }
            Expr::Str { value, .. } => {
                self.emit(Instr::ConstStr(value.clone()));
            }
            Expr::Bool { value, .. } => {
                self.emit(Instr::ConstBool(*value));
            }
            Expr::Ident { name, .. } => {
                let resolved = self
                    .capture_map
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone());
                self.emit(Instr::Load(resolved));
            }
            Expr::Binary {
                op,
                left,
                right,
                resolved_method,
                ..
            } => {
                if let Some(ref fname) = resolved_method {
                    self.gen_expr(left);
                    self.gen_expr(right);
                    self.emit(Instr::Call(fname.clone(), 2));
                } else {
                    self.gen_expr(left);
                    self.gen_expr(right);
                    self.emit(Instr::Binary(match op {
                        BinOp::Add => Op::Add,
                        BinOp::Sub => Op::Sub,
                        BinOp::Mul => Op::Mul,
                        BinOp::Div => Op::Div,
                        BinOp::Mod => Op::Mod,
                        BinOp::Equal => Op::Equal,
                        BinOp::NotEqual => Op::NotEqual,
                        BinOp::Less => Op::Less,
                        BinOp::LessEqual => Op::LessEqual,
                        BinOp::Greater => Op::Greater,
                        BinOp::GreaterEqual => Op::GreaterEqual,
                        BinOp::And => Op::And,
                        BinOp::Or => Op::Or,
                    }));
                }
            }
            Expr::Unary { op, operand, .. } => {
                self.gen_expr(operand);
                self.emit(Instr::Unary(match op {
                    UnOp::Negate => Op::Negate,
                    UnOp::Not => Op::Not,
                }));
            }
            Expr::Call { callee, args, .. } => {
                let callee_inner = match callee.as_ref() {
                    Expr::Grouping { expr, .. } => expr.as_ref(),
                    other => other,
                };
                match callee_inner {
                    Expr::Ident { name, .. } => {
                        if self.fn_names.contains(name)
                            || matches!(
                                name.as_str(),
                                "imprimir"
                                    | "print"
                                    | "leer"
                                    | "read"
                                    | "a_texto"
                                    | "to_texto"
                                    | "__str_from"
                                    | "largo"
                                    | "len"
                                    | "agregar"
                                    | "push"
                                    | "__str_len"
                                    | "__str_longitud"
                                    | "__str_upper"
                                    | "__str_mayusculas"
                                    | "__str_lower"
                                    | "__str_minusculas"
                                    | "__str_trim"
                                    | "__str_recortar"
                                    | "__str_contains"
                                    | "__str_contiene"
                                    | "__str_split"
                                    | "__str_dividir"
                                    | "__str_ord"
                                    | "__str_codigo"
                                    | "__file_read"
                                    | "__leer_archivo"
                                    | "__file_write"
                                    | "__escribir_archivo"
                                    | "__file_exists"
                                    | "__existe_archivo"
                                    | "__time_now"
                                    | "__tiempo_ahora"
                                    | "__list_reverse"
                                    | "__lista_invertir"
                                    | "__list_sort"
                                    | "__lista_ordenar"
| "__json_parse"
| "__json_parsear"
| "__json_stringify"
| "__json_texto"
| "__js_call"
| "__js_llamar"
| "__js_eval"
| "__js_evaluar"
                                    | "__map_new"
                                    | "__map_nuevo"
                                    | "__map_set"
                                    | "__map_poner"
                                    | "__map_get"
                                    | "__map_obtener"
                                    | "__map_len"
                                    | "__map_longitud"
                                    | "__map_keys"
                                    | "__map_claves"
                                    | "__map_contains"
                                    | "__map_contiene"
                                    | "__set_new"
                                    | "__conjunto_nuevo"
                                    | "__set_add"
                                    | "__conjunto_agregar"
                                    | "__set_has"
                                    | "__conjunto_tiene"
                                    | "__set_union"
                                    | "__conjunto_unir"
                                    | "__set_inter"
                                    | "__conjunto_interseccion"
                                    | "__set_diff"
                                    | "__conjunto_diferencia"
                                    | "__deque_new"
                                    | "__deque_nuevo"
                                    | "__deque_push_front"
                                    | "__deque_agregar_frente"
                                    | "__deque_push_back"
                                    | "__deque_agregar_final"
                                    | "__deque_pop_front"
                                    | "__deque_quitar_frente"
                                    | "__deque_pop_back"
                                    | "__deque_quitar_final"
                                    | "__deque_len"
                                    | "__deque_longitud"
                                    | "__heap_new"
                                    | "__monticulo_nuevo"
                                    | "__heap_push"
                                    | "__monticulo_agregar"
                                    | "__heap_pop"
                                    | "__monticulo_quitar"
                                    | "__heap_peek"
                                    | "__monticulo_ver"
                                    | "__heap_len"
                                    | "__monticulo_longitud"
                                    | "__linked_new"
                                    | "__enlazada_nuevo"
                                    | "__linked_push_front"
                                    | "__enlazada_agregar_frente"
                                    | "__linked_push_back"
                                    | "__enlazada_agregar_final"
                                    | "__linked_pop_front"
                                    | "__enlazada_quitar_frente"
                                    | "__linked_pop_back"
                                    | "__enlazada_quitar_final"
                                    | "__linked_len"
                                    | "__enlazada_longitud"
                                    | "__regex_new"
                                    | "__regex_nuevo"
                                    | "__regex_is_match"
                                    | "__regex_coincide"
                                    | "__regex_captures"
                                    | "__regex_capturar"
                                    | "__regex_replace"
                                    | "__regex_reemplazar"
                                    | "__unicode_normalize"
                                    | "__unicode_normalizar"
                                    | "__str_pad_start"
                                    | "__str_padding_inicio"
                                    | "__str_pad_end"
                                    | "__str_padding_fin"
                                    | "__encoding_utf8"
                                    | "__codificacion_utf8"
                                    | "__encoding_from_utf8"
                                    | "__desde_utf8"
                                    | "__buf_reader"
                                    | "__lector_buffer"
                                    | "__buf_writer"
                                    | "__escritor_buffer"
                                    | "__stream_chunks"
                                    | "__stream_trozos"
                                    | "__tcp_connect"
                                    | "__tcp_conectar"
                                    | "__tcp_listen"
                                    | "__tcp_escuchar"
                                    | "__tcp_accept"
                                    | "__tcp_aceptar"
                                    | "__http_get"
                                    | "__http_obtener"
                                    | "__http_post"
                                    | "__http_enviar"
                                    | "__http_server"
                                    | "__http_servidor"
                                     | "__serial_open"
                                     | "__serial_abrir"
                                     | "__actor_enviar"
                                     | "__actor_new"
                                     | "__actor_nuevo"
                                     | "__actor_recibir"
                                     | "__actor_recv"
                                     | "__actor_send"
                                     | "__aes_decrypt"
                                     | "__aes_desencriptar"
                                     | "__aes_encriptar"
                                     | "__aes_encrypt"
                                     | "__arc_asignar"
                                     | "__arc_get"
                                     | "__arc_new"
                                     | "__arc_nuevo"
                                     | "__arc_obtener"
                                     | "__arc_set"
                                     | "__canal_enviar"
                                     | "__canal_nuevo"
                                     | "__canal_recibir"
                                     | "__channel_new"
                                     | "__channel_recv"
                                     | "__channel_send"
                                     | "__cluster_conectar"
                                     | "__cluster_connect"
                                     | "__cluster_enviar"
                                     | "__cluster_send"
                                     | "__coro_ceder"
                                     | "__coro_crear"
                                     | "__coro_create"
                                     | "__coro_reanudar"
                                     | "__coro_resume"
                                     | "__coro_yield"
                                     | "__dormir"
                                     | "__env_list"
                                     | "__env_listar"
                                     | "__ffi_alloc"
                                     | "__ffi_asignar"
                                     | "__ffi_call"
                                     | "__ffi_cargar"
                                     | "__ffi_escribir"
                                     | "__ffi_free"
                                     | "__ffi_leer"
                                     | "__ffi_liberar"
                                     | "__ffi_llamar"
                                     | "__ffi_load"
                                     | "__ffi_peek"
                                     | "__ffi_poke"
                                     | "__ffi_read"
                                     | "__ffi_write"
                                     | "__fs_listar"
                                     | "__fs_listdir"
                                     | "__generador_nuevo"
                                     | "__generador_siguiente"
                                     | "__generator_new"
                                     | "__generator_next"
                                     | "__gui_cerrar"
                                     | "__gui_close"
                                     | "__gui_esperar"
                                     | "__gui_hwnd"
                                     | "__gui_id"
                                     | "__gui_mostrar"
                                     | "__gui_poll"
                                     | "__gui_show"
                                     | "__gui_ventana"
                                     | "__gui_window"
                                     | "__hash_sha256"
                                     | "__hash_sha512"
                                     | "__hilo_esperar"
                                     | "__hilo_lanzar"
                                     | "__jwt_codificar"
                                     | "__jwt_decode"
                                     | "__jwt_decodificar"
                                     | "__jwt_encode"
                                     | "__mutex_bloquear"
                                     | "__mutex_lock"
                                     | "__mutex_new"
                                     | "__mutex_nuevo"
                                     | "__par_join"
                                     | "__par_map"
                                     | "__par_mapear"
                                     | "__par_unir"
                                     | "__rwlock_escribir"
                                     | "__rwlock_leer"
                                     | "__rwlock_new"
                                     | "__rwlock_nuevo"
                                     | "__rwlock_read"
                                     | "__rwlock_write"
                                     | "__scope_cancel"
                                     | "__scope_cancelar"
                                     | "__scope_lanzar"
                                     | "__scope_new"
                                     | "__scope_nuevo"
                                     | "__scope_spawn"
                                     | "__seleccionar"
                                     | "__select"
                                     | "__sleep"
                                     | "__stream_colectar"
                                     | "__stream_collect"
                                     | "__stream_desde"
                                     | "__stream_filter"
                                     | "__stream_filtrar"
                                     | "__stream_from"
                                     | "__stream_map"
                                     | "__stream_mapear"
                                     | "__supervisor_add"
                                     | "__supervisor_agregar"
                                     | "__supervisor_iniciar"
                                     | "__supervisor_new"
                                     | "__supervisor_nuevo"
                                     | "__supervisor_start"
                                     | "__tarea_esperar"
                                     | "__tarea_lanzar"
                                     | "__task_await"
                                     | "__task_spawn"
                                     | "__thread_join"
                                     | "__thread_spawn"
                                     | "__tiempo_diferencia"
                                     | "__tiempo_formatear"
                                     | "__tiempo_parsear"
                                     | "__time_diff"
                                     | "__time_format"
                                     | "__time_parse"
                                      | "__tipo_de"
                                      | "__typeof"
                                      | "__timezone_info"
                                      | "__zona_info"
                                      | "__duration_new"
                                      | "__duracion_nueva"
                                      | "__duration_secs"
                                      | "__duracion_segundos"
                                      | "__calendar_hijri"
                                      | "__calendario_hijri"
                                      | "__calendar_persian"
                                      | "__calendario_persa"
                                      | "__leer_archivo_async"
                                      | "__file_read_async"
                                      | "__escribir_archivo_async"
                                      | "__file_write_async"
                                      | "__timer_delay"
                                      | "__temporizador_esperar"
                                      | "__tcp_connect_async"
                                      | "__tcp_conectar_async"
                              )
                        {
                            for arg in args {
                                self.gen_expr(arg);
                            }
                            let defaults = self.default_params.get(name).cloned();
                            let argc = if let Some(defaults) = defaults {
                                let mut count = args.len();
                                for default_expr in defaults.iter().skip(args.len()).flatten() {
                                    self.gen_expr(default_expr);
                                    count += 1;
                                }
                                count
                            } else {
                                args.len()
                            };
                            self.emit(Instr::Call(name.clone(), argc));
                        } else {
                            self.emit(Instr::Load(name.clone()));
                            for arg in args {
                                self.gen_expr(arg);
                            }
                            self.emit(Instr::CallValue(args.len()));
                        }
                    }
                    Expr::Lambda { params, body, .. } => {
                        let lambda_name = self.compile_lambda(params, body);
                        for arg in args {
                            self.gen_expr(arg);
                        }
                        self.emit(Instr::Call(lambda_name, args.len()));
                    }
                    _ => {
                        self.gen_expr(callee);
                        for arg in args {
                            self.gen_expr(arg);
                        }
                        self.emit(Instr::CallValue(args.len()));
                    }
                }
            }
            Expr::List { items, .. } => {
                for item in items {
                    self.gen_expr(item);
                }
                self.emit(Instr::ArrayNew(items.len()));
            }
            Expr::Index { expr, index, .. } => {
                self.gen_expr(expr);
                self.gen_expr(index);
                self.emit(Instr::ArrayGet);
            }
            Expr::MethodCall {
                expr,
                method,
                args,
                resolved_func,
                ..
            } => {
                let var_name = match expr.as_ref() {
                    Expr::Ident { name, .. } => Some(name.clone()),
                    _ => None,
                };
                // Check if this is a trait method call
                let func_name = resolved_func
                    .clone()
                    .or_else(|| self.impl_method_map.get(method.as_str()).cloned());
                if let Some(fname) = func_name {
                    // Trait method: receiver pushed as first arg
                    self.gen_expr(expr);
                    for arg in args {
                        self.gen_expr(arg);
                    }
                    self.emit(Instr::Call(fname, args.len() + 1));
                } else {
                    self.gen_expr(expr);
                    match method.as_str() {
                        "agregar" | "push" => {
                            for arg in args {
                                self.gen_expr(arg);
                            }
                            self.emit(Instr::ArrayPush);
                            if let Some(name) = var_name {
                                self.emit(Instr::Store(name));
                            }
                        }
                        "largo" | "len" | "length" => {
                            self.emit(Instr::ArrayLen);
                        }
                        _ => {}
                    }
                }
            }
            Expr::Grouping { expr, .. } => {
                self.gen_expr(expr);
            }
            Expr::StructInit {
                struct_name,
                fields,
                ..
            } => {
                for (_, val) in fields {
                    self.gen_expr(val);
                }
                for (name, _) in fields {
                    self.emit(Instr::ConstStr(name.clone()));
                }
                self.emit(Instr::StructNew(struct_name.clone(), fields.len()));
            }
            Expr::FieldAccess { expr, field, .. } => {
                self.gen_expr(expr);
                self.emit(Instr::ConstStr(field.clone()));
                self.emit(Instr::StructGet);
            }
            Expr::Lambda { params, body, .. } => {
                let lambda_name = self.compile_lambda(params, body);
                self.emit(Instr::FuncRef(lambda_name));
            }
            Expr::Exito { expr, .. } => {
                self.gen_expr(expr);
                self.emit(Instr::ResultOk);
            }
            Expr::Error { expr, .. } => {
                self.gen_expr(expr);
                self.emit(Instr::ResultErr);
            }
            Expr::Intentar { expr, .. } => {
                self.gen_expr(expr);
                self.emit(Instr::TryUnwrap);
            }
            Expr::Algun { expr, .. } => {
                self.gen_expr(expr);
                self.emit(Instr::OptionSome);
            }
            Expr::Ninguno { .. } => {
                self.emit(Instr::OptionNone);
            }
            Expr::Tuple { items, .. } => {
                for item in items {
                    self.gen_expr(item);
                }
                self.emit(Instr::TupleNew(items.len()));
            }
            Expr::TupleAccess { expr, index, .. } => {
                self.gen_expr(expr);
                self.emit(Instr::TupleAccess(*index));
            }
            Expr::EnumCtor {
                enum_name,
                variant,
                args,
                ..
            } => {
                for arg in args {
                    self.gen_expr(arg);
                }
                self.emit(Instr::EnumCtor {
                    enum_name: enum_name.clone(),
                    variant: variant.clone(),
                    argc: args.len(),
                });
            }
            Expr::Ternary {
                condition,
                true_branch,
                false_branch,
                ..
            } => {
                let else_label = self.new_label();
                let end_label = self.new_label();
                self.gen_expr(condition);
                self.emit(Instr::JmpIf(else_label));
                self.gen_expr(true_branch);
                self.emit(Instr::Jmp(end_label));
                self.emit(Instr::Label(else_label));
                self.gen_expr(false_branch);
                self.emit(Instr::Label(end_label));
            }
            Expr::Esperar { expr, .. } => {
                self.gen_expr(expr);
                self.emit(Instr::Call("__tarea_esperar".to_string(), 1));
            }
        }
    }

    fn compile_lambda(&mut self, params: &[Param], body: &[DeclOrStmt]) -> String {
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        let mut captured = Vec::new();
        self.collect_variable_refs(body, &param_names, &mut captured);
        // Only create capture slots when NOT already inside a lambda (nested lambdas
        // capture via parent's params which are already in the parent's scope)
        if !self.is_in_lambda {
            for var_name in &captured {
                if !param_names.contains(var_name) && !self.capture_map.contains_key(var_name) {
                    let cap_name = format!("__cap_{}_{}", self.lambda_counter, var_name);
                    self.capture_map.insert(var_name.clone(), cap_name);
                }
            }
        }

        let func = Func {
            name: lambda_name.clone(),
            params: param_names,
            entry: 0,
            instrs: Vec::new(),
        };
        self.program.funcs.insert(lambda_name.clone(), func);
        let saved_instrs = std::mem::take(&mut self.current_instrs);
        let saved_func = self.current_func.clone();
        let saved_temp = self.temp_counter;
        let saved_label = self.label_counter;
        let saved_loop = std::mem::take(&mut self.loop_labels);
        let saved_is_lambda = self.is_in_lambda;
        self.current_func = Some(lambda_name.clone());
        self.current_instrs = Vec::new();
        self.temp_counter = 0;
        self.label_counter = 0;
        self.is_in_lambda = true;
        for node in body {
            self.gen_decl_or_stmt(node);
        }
        if !self
            .current_instrs
            .iter()
            .any(|i| matches!(i, Instr::Return))
        {
            self.emit(Instr::Return);
        }
        self.finalize_func();
        self.current_func = saved_func;
        self.current_instrs = saved_instrs;
        self.temp_counter = saved_temp;
        self.label_counter = saved_label;
        self.loop_labels = saved_loop;
        // Keep capture_map — don't restore, so outer scope shares captured vars
        self.is_in_lambda = saved_is_lambda;
        lambda_name
    }

    fn collect_variable_refs(&self, body: &[DeclOrStmt], params: &[String], out: &mut Vec<String>) {
        for node in body {
            match node {
                DeclOrStmt::Stmt(stmt) => self.collect_stmt_refs(stmt, params, out),
                DeclOrStmt::Decl(decl) => self.collect_decl_refs(decl, params, out),
            }
        }
    }

    fn collect_stmt_refs(&self, stmt: &Stmt, params: &[String], out: &mut Vec<String>) {
        match stmt {
            Stmt::Expr { expr, .. } => self.collect_expr_refs(expr, params, out),
            Stmt::Assignment { name, value, .. } => {
                if !params.contains(name) && !out.contains(name) {
                    out.push(name.clone());
                }
                self.collect_expr_refs(value, params, out);
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.collect_expr_refs(condition, params, out);
                self.collect_variable_refs(then_body, params, out);
                if let Some(eb) = else_body {
                    self.collect_variable_refs(eb, params, out);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.collect_expr_refs(condition, params, out);
                self.collect_variable_refs(body, params, out);
            }
            Stmt::ForEach { expr, body, .. } => {
                self.collect_expr_refs(expr, params, out);
                self.collect_variable_refs(body, params, out);
            }
            Stmt::Return { value: Some(v), .. } => self.collect_expr_refs(v, params, out),
            Stmt::Return { value: None, .. } => {}
            Stmt::FieldAssign { expr, value, .. } => {
                self.collect_expr_refs(expr, params, out);
                self.collect_expr_refs(value, params, out);
            }
            Stmt::Match { expr, arms, .. } => {
                self.collect_expr_refs(expr, params, out);
                for arm in arms {
                    self.collect_variable_refs(&arm.body, params, out);
                }
            }
            Stmt::Block { stmts, .. } => self.collect_variable_refs(stmts, params, out),
            _ => {}
        }
    }

    fn collect_decl_refs(&self, decl: &Decl, params: &[String], out: &mut Vec<String>) {
        match decl {
            Decl::Variable { init: Some(v), .. } => self.collect_expr_refs(v, params, out),
            Decl::Variable { init: None, .. } => {}
            Decl::Destructure { init, .. } => {
                self.collect_expr_refs(init, params, out);
            }
            _ => {}
        }
    }

    fn collect_expr_refs(&self, expr: &Expr, params: &[String], out: &mut Vec<String>) {
        match expr {
            Expr::Ident { name, .. } => {
                if !params.contains(name) && !out.contains(name) {
                    out.push(name.clone());
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_expr_refs(left, params, out);
                self.collect_expr_refs(right, params, out);
            }
            Expr::Unary { operand, .. } => self.collect_expr_refs(operand, params, out),
            Expr::Call { callee, args, .. } => {
                self.collect_expr_refs(callee, params, out);
                for a in args {
                    self.collect_expr_refs(a, params, out);
                }
            }
            Expr::Index { expr, index, .. } => {
                self.collect_expr_refs(expr, params, out);
                self.collect_expr_refs(index, params, out);
            }
            Expr::MethodCall { expr, args, .. } => {
                self.collect_expr_refs(expr, params, out);
                for a in args {
                    self.collect_expr_refs(a, params, out);
                }
            }
            Expr::Ternary {
                condition,
                true_branch,
                false_branch,
                ..
            } => {
                self.collect_expr_refs(condition, params, out);
                self.collect_expr_refs(true_branch, params, out);
                self.collect_expr_refs(false_branch, params, out);
            }
            Expr::List { items, .. } => {
                for i in items {
                    self.collect_expr_refs(i, params, out);
                }
            }
            Expr::Grouping { expr, .. } => self.collect_expr_refs(expr, params, out),
            Expr::StructInit { fields, .. } => {
                for (_, v) in fields {
                    self.collect_expr_refs(v, params, out);
                }
            }
            Expr::FieldAccess { expr, .. } => self.collect_expr_refs(expr, params, out),
            Expr::Tuple { items, .. } => {
                for i in items {
                    self.collect_expr_refs(i, params, out);
                }
            }
            _ => {}
        }
    }

    fn emit(&mut self, instr: Instr) {
        self.current_instrs.push(instr);
    }

    fn new_label(&mut self) -> usize {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }

    fn finalize_func(&mut self) {
        if let Some(ref name) = self.current_func {
            if let Some(func) = self.program.funcs.get_mut(name) {
                func.instrs = std::mem::take(&mut self.current_instrs);
                Self::optimize_func(func);
            }
        }
    }

    pub fn fold_constants_pass(instrs: &[Instr]) -> Vec<Instr> {
        let mut result = Vec::with_capacity(instrs.len());
        let mut i = 0;
        while i < instrs.len() {
            // Binary folding: ConstX(a), ConstY(b), Binary(op)
            if i + 2 < instrs.len() {
                if let Some(folded) =
                    Self::try_fold_binary(&instrs[i], &instrs[i + 1], &instrs[i + 2])
                {
                    result.push(folded);
                    i += 3;
                    continue;
                }
            }
            // Unary folding: ConstX(a), Unary(op)
            if i + 1 < instrs.len() {
                if let Some(folded) = Self::try_fold_unary(&instrs[i], &instrs[i + 1]) {
                    result.push(folded);
                    i += 2;
                    continue;
                }
            }
            result.push(instrs[i].clone());
            i += 1;
        }
        result
    }

    fn try_fold_binary(a: &Instr, b: &Instr, op: &Instr) -> Option<Instr> {
        match (a, b, op) {
            // Int +-*/ Int
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Add)) => {
                Some(Instr::ConstInt(a.overflowing_add(*b).0))
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Sub)) => {
                Some(Instr::ConstInt(a.overflowing_sub(*b).0))
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Mul)) => {
                Some(Instr::ConstInt(a.overflowing_mul(*b).0))
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Div)) => {
                if *b != 0 {
                    Some(Instr::ConstInt(a / b))
                } else {
                    None
                }
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Mod)) => {
                if *b != 0 {
                    Some(Instr::ConstInt(a % b))
                } else {
                    None
                }
            }
            // Float +-*/ Float
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::Add)) => {
                Some(Instr::ConstFloat(a + b))
            }
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::Sub)) => {
                Some(Instr::ConstFloat(a - b))
            }
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::Mul)) => {
                Some(Instr::ConstFloat(a * b))
            }
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::Div)) => {
                if *b != 0.0 {
                    Some(Instr::ConstFloat(a / b))
                } else {
                    None
                }
            }
            // Mixed Int/Float arithmetic
            (Instr::ConstInt(a), Instr::ConstFloat(b), Instr::Binary(Op::Add)) => {
                Some(Instr::ConstFloat(*a as f64 + b))
            }
            (Instr::ConstInt(a), Instr::ConstFloat(b), Instr::Binary(Op::Sub)) => {
                Some(Instr::ConstFloat(*a as f64 - b))
            }
            (Instr::ConstInt(a), Instr::ConstFloat(b), Instr::Binary(Op::Mul)) => {
                Some(Instr::ConstFloat(*a as f64 * b))
            }
            (Instr::ConstInt(a), Instr::ConstFloat(b), Instr::Binary(Op::Div)) => {
                if *b != 0.0 {
                    Some(Instr::ConstFloat(*a as f64 / b))
                } else {
                    None
                }
            }
            (Instr::ConstFloat(a), Instr::ConstInt(b), Instr::Binary(Op::Add)) => {
                Some(Instr::ConstFloat(a + *b as f64))
            }
            (Instr::ConstFloat(a), Instr::ConstInt(b), Instr::Binary(Op::Sub)) => {
                Some(Instr::ConstFloat(a - *b as f64))
            }
            (Instr::ConstFloat(a), Instr::ConstInt(b), Instr::Binary(Op::Mul)) => {
                Some(Instr::ConstFloat(a * *b as f64))
            }
            (Instr::ConstFloat(a), Instr::ConstInt(b), Instr::Binary(Op::Div)) => {
                if *b != 0 {
                    Some(Instr::ConstFloat(a / *b as f64))
                } else {
                    None
                }
            }
            // Int comparisons
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Equal)) => {
                Some(Instr::ConstBool(a == b))
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::NotEqual)) => {
                Some(Instr::ConstBool(a != b))
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Less)) => {
                Some(Instr::ConstBool(a < b))
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::LessEqual)) => {
                Some(Instr::ConstBool(a <= b))
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Greater)) => {
                Some(Instr::ConstBool(a > b))
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::GreaterEqual)) => {
                Some(Instr::ConstBool(a >= b))
            }
            // Float comparisons
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::Equal)) => {
                Some(Instr::ConstBool((a - b).abs() < f64::EPSILON))
            }
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::NotEqual)) => {
                Some(Instr::ConstBool((a - b).abs() >= f64::EPSILON))
            }
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::Less)) => {
                Some(Instr::ConstBool(a < b))
            }
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::LessEqual)) => {
                Some(Instr::ConstBool(a <= b))
            }
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::Greater)) => {
                Some(Instr::ConstBool(a > b))
            }
            (Instr::ConstFloat(a), Instr::ConstFloat(b), Instr::Binary(Op::GreaterEqual)) => {
                Some(Instr::ConstBool(a >= b))
            }
            // Mixed Int/Float comparisons
            (Instr::ConstInt(a), Instr::ConstFloat(b), Instr::Binary(Op::Equal)) => {
                Some(Instr::ConstBool((*a as f64 - b).abs() < f64::EPSILON))
            }
            (Instr::ConstInt(a), Instr::ConstFloat(b), Instr::Binary(Op::NotEqual)) => {
                Some(Instr::ConstBool((*a as f64 - b).abs() >= f64::EPSILON))
            }
            (Instr::ConstInt(a), Instr::ConstFloat(b), Instr::Binary(Op::Less)) => {
                Some(Instr::ConstBool((*a as f64) < *b))
            }
            (Instr::ConstInt(a), Instr::ConstFloat(b), Instr::Binary(Op::Greater)) => {
                Some(Instr::ConstBool((*a as f64) > *b))
            }
            (Instr::ConstFloat(a), Instr::ConstInt(b), Instr::Binary(Op::Equal)) => {
                Some(Instr::ConstBool((a - *b as f64).abs() < f64::EPSILON))
            }
            (Instr::ConstFloat(a), Instr::ConstInt(b), Instr::Binary(Op::NotEqual)) => {
                Some(Instr::ConstBool((a - *b as f64).abs() >= f64::EPSILON))
            }
            (Instr::ConstFloat(a), Instr::ConstInt(b), Instr::Binary(Op::Less)) => {
                Some(Instr::ConstBool(*a < *b as f64))
            }
            (Instr::ConstFloat(a), Instr::ConstInt(b), Instr::Binary(Op::Greater)) => {
                Some(Instr::ConstBool(*a > *b as f64))
            }
            // Bool logical ops
            (Instr::ConstBool(a), Instr::ConstBool(b), Instr::Binary(Op::And)) => {
                Some(Instr::ConstBool(*a && *b))
            }
            (Instr::ConstBool(a), Instr::ConstBool(b), Instr::Binary(Op::Or)) => {
                Some(Instr::ConstBool(*a || *b))
            }
            (Instr::ConstBool(a), Instr::ConstBool(b), Instr::Binary(Op::Equal)) => {
                Some(Instr::ConstBool(a == b))
            }
            (Instr::ConstBool(a), Instr::ConstBool(b), Instr::Binary(Op::NotEqual)) => {
                Some(Instr::ConstBool(a != b))
            }
            // String concatenation
            (Instr::ConstStr(a), Instr::ConstStr(b), Instr::Binary(Op::Add)) => {
                Some(Instr::ConstStr(format!("{}{}", a, b)))
            }
            _ => None,
        }
    }

    fn try_fold_unary(a: &Instr, op: &Instr) -> Option<Instr> {
        match (a, op) {
            (Instr::ConstInt(n), Instr::Unary(Op::Negate)) => {
                Some(Instr::ConstInt(n.overflowing_neg().0))
            }
            (Instr::ConstFloat(n), Instr::Unary(Op::Negate)) => Some(Instr::ConstFloat(-n)),
            (Instr::ConstBool(b), Instr::Unary(Op::Not)) => Some(Instr::ConstBool(!b)),
            (Instr::ConstInt(n), Instr::Unary(Op::Not)) => Some(Instr::ConstBool(*n == 0)),
            (Instr::ConstFloat(n), Instr::Unary(Op::Not)) => Some(Instr::ConstBool(*n == 0.0)),
            _ => None,
        }
    }

    fn optimize_func(func: &mut Func) {
        let mut current = func.instrs.clone();

        // Run constant folding multiple passes for chained operations
        for _ in 0..3 {
            let new = Self::fold_constants_pass(&current);
            if new == current {
                break;
            }
            current = new;
        }

        // Remove consecutive Nops (keep at most one)
        let mut optimized = Vec::with_capacity(current.len());
        let mut prev_nop = false;
        for instr in &current {
            match instr {
                Instr::Nop => {
                    if !prev_nop {
                        optimized.push(instr.clone());
                        prev_nop = true;
                    }
                }
                _ => {
                    optimized.push(instr.clone());
                    prev_nop = false;
                }
            }
        }
        func.instrs = optimized;
    }
}

fn type_to_impl_name(t: &Type) -> Option<String> {
    match t {
        Type::Numero => Some("numero".to_string()),
        Type::Entero => Some("entero".to_string()),
        Type::Decimal => Some("decimal".to_string()),
        Type::Texto => Some("texto".to_string()),
        Type::Booleano => Some("booleano".to_string()),
        Type::Struct(name) => Some(name.clone()),
        Type::GenericStruct { name, .. } => Some(name.clone()),
        Type::Lista(_) => Some("lista".to_string()),
        Type::Resultado { .. } => Some("resultado".to_string()),
        Type::Opcion(_) => Some("opcion".to_string()),
        Type::Tuple(_) => Some("tupla".to_string()),
        Type::Func { .. } | Type::ImplTrait(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_lexer::Lexer;
    use lumen_parser::Parser;

    fn build_ir(source: &str) -> crate::ir::Program {
        let lexer = Lexer::new(source);
        let (tokens, lex_errors) = lexer.tokenize();
        assert!(lex_errors.is_empty());
        let parser = Parser::new(tokens);
        let (program, parse_errors) = parser.parse();
        assert!(parse_errors.is_empty());
        let builder = IRBuilder::new();
        builder.build(&program)
    }

    #[test]
    fn test_variable_assignment() {
        let program = build_ir("numero x = 42;");
        assert!(!program.funcs.is_empty());
    }

    #[test]
    fn test_simple_function() {
        let source = "funcion numero suma(numero a, numero b) { retornar a + b; }";
        let program = build_ir(source);
        assert!(program.funcs.contains_key("suma"));
    }

    #[test]
    fn test_if_else() {
        let source =
            "booleano flag = verdadero; si (flag) { numero x = 1; } sino { numero y = 2; }";
        let program = build_ir(source);
        assert!(!program.funcs.is_empty());
    }

    #[test]
    fn test_while_loop() {
        let source = "numero i = 0; mientras (i < 10) { i = i + 1; }";
        let program = build_ir(source);
        assert!(!program.funcs.is_empty());
    }

    #[test]
    fn test_complex_program() {
        let source = "funcion numero suma(numero a, numero b) { retornar a + b; }
numero x = suma(3, 4);
imprimir(x);";
        let program = build_ir(source);
        assert!(program.funcs.contains_key("suma"));
    }

    #[test]
    fn test_constant_folding_int_add() {
        let instrs = vec![
            Instr::ConstInt(2),
            Instr::ConstInt(3),
            Instr::Binary(Op::Add),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstInt(5)));
    }

    #[test]
    fn test_constant_folding_int_sub() {
        let instrs = vec![
            Instr::ConstInt(10),
            Instr::ConstInt(3),
            Instr::Binary(Op::Sub),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstInt(7)));
    }

    #[test]
    fn test_constant_folding_int_mul() {
        let instrs = vec![
            Instr::ConstInt(6),
            Instr::ConstInt(7),
            Instr::Binary(Op::Mul),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstInt(42)));
    }

    #[test]
    fn test_constant_folding_float_add() {
        let instrs = vec![
            Instr::ConstFloat(1.5),
            Instr::ConstFloat(2.5),
            Instr::Binary(Op::Add),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstFloat(v) if (v - 4.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_constant_folding_comparison_lt() {
        let instrs = vec![
            Instr::ConstInt(3),
            Instr::ConstInt(5),
            Instr::Binary(Op::Less),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstBool(true)));
    }

    #[test]
    fn test_constant_folding_comparison_gt() {
        let instrs = vec![
            Instr::ConstInt(5),
            Instr::ConstInt(3),
            Instr::Binary(Op::Greater),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstBool(true)));
    }

    #[test]
    fn test_constant_folding_bool_and() {
        let instrs = vec![
            Instr::ConstBool(true),
            Instr::ConstBool(false),
            Instr::Binary(Op::And),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstBool(false)));
    }

    #[test]
    fn test_constant_folding_bool_or() {
        let instrs = vec![
            Instr::ConstBool(false),
            Instr::ConstBool(true),
            Instr::Binary(Op::Or),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstBool(true)));
    }

    #[test]
    fn test_constant_folding_string_concat() {
        let instrs = vec![
            Instr::ConstStr("Hola ".to_string()),
            Instr::ConstStr("Mundo".to_string()),
            Instr::Binary(Op::Add),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(&folded[0], Instr::ConstStr(s) if s == "Hola Mundo"));
    }

    #[test]
    fn test_constant_folding_unary_negate_int() {
        let instrs = vec![Instr::ConstInt(5), Instr::Unary(Op::Negate)];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstInt(-5)));
    }

    #[test]
    fn test_constant_folding_unary_negate_float() {
        let instrs = vec![Instr::ConstFloat(1.5), Instr::Unary(Op::Negate)];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstFloat(v) if (v - (-1.5)).abs() < f64::EPSILON));
    }

    #[test]
    fn test_constant_folding_unary_not_bool() {
        let instrs = vec![Instr::ConstBool(true), Instr::Unary(Op::Not)];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstBool(false)));
    }

    #[test]
    fn test_constant_folding_chained() {
        let instrs = vec![
            Instr::ConstInt(1),
            Instr::ConstInt(2),
            Instr::Binary(Op::Add),
            Instr::ConstInt(3),
            Instr::Binary(Op::Add),
        ];
        let mut current = instrs;
        for _ in 0..3 {
            current = IRBuilder::fold_constants_pass(&current);
        }
        assert_eq!(current.len(), 1);
        assert!(matches!(current[0], Instr::ConstInt(6)));
    }

    #[test]
    fn test_constant_folding_mixed_int_float() {
        let instrs = vec![
            Instr::ConstInt(3),
            Instr::ConstFloat(2.5),
            Instr::Binary(Op::Add),
        ];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstFloat(v) if (v - 5.5).abs() < f64::EPSILON));
    }

    #[test]
    fn test_dce_removes_consecutive_nops() {
        let mut func = Func {
            name: "test".to_string(),
            params: vec![],
            entry: 0,
            instrs: vec![
                Instr::Nop,
                Instr::Nop,
                Instr::ConstInt(42),
                Instr::Nop,
                Instr::Nop,
                Instr::Nop,
                Instr::Store("x".to_string()),
            ],
        };
        IRBuilder::optimize_func(&mut func);
        assert_eq!(func.instrs.len(), 4);
        assert!(matches!(func.instrs[0], Instr::Nop));
        assert!(matches!(func.instrs[1], Instr::ConstInt(42)));
        assert!(matches!(func.instrs[2], Instr::Nop));
        assert!(matches!(func.instrs[3], Instr::Store(_)));
    }
}
