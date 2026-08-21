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
    /// BUG-063: pila de ámbitos de bloque. Cada entrada mapea el nombre escrito
    /// por el usuario al slot real donde vive. Las variables del runtime son
    /// planas por marco (una tabla por nombre), así que un `entero x` dentro de
    /// un `si` machacaba la `x` de fuera en vez de sombrearla. Sólo se renombra
    /// cuando el nombre YA es visible en un ámbito exterior, para no tocar el
    /// código que ya funcionaba.
    block_scopes: Vec<HashMap<String, String>>,
    shadow_counter: usize,
    /// BUG-028: cuerpos de los bloques `posponer` de la función que se está
    /// generando. Se emiten al salir (final de la función y en cada
    /// `retornar`), en orden inverso al de declaración, como manda un `defer`.
    deferred: Vec<Vec<crate::ir::Instr>>,
    is_in_lambda: bool,
    /// BUG-060: nombre de la variable que se está inicializando ahora mismo con
    /// una lambda. Su cuerpo puede referirse a ella (recursión), y esa
    /// referencia NO es una captura del entorno: en el momento de crear la
    /// closure la variable todavía no tiene valor, así que capturarla por valor
    /// guardaba un hueco y la llamada recursiva moría con «Variable no
    /// definida». Se resuelve por nombre al llamar, cuando ya está asignada.
    self_binding: Option<String>,
    /// BUG-008: índices de los parámetros declarados `prestado mut` en cada
    /// función. Tras la llamada se copia el valor final del parámetro de vuelta
    /// a la variable del llamador (paso por referencia observable).
    mut_borrow_params: HashMap<String, Vec<usize>>,
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
            block_scopes: Vec::new(),
            shadow_counter: 0,
            deferred: Vec::new(),
            is_in_lambda: false,
            self_binding: None,
            mut_borrow_params: HashMap::new(),
        }
    }

    pub fn build(mut self, program: &[DeclOrStmt]) -> crate::ir::Program {
        // BUG-063: ámbito base del programa (`__main__`). Sin él, las variables
        // de nivel superior no quedan registradas y un bloque interior no sabe
        // que está sombreando algo.
        self.push_block_scope();
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
                    captures: Vec::new(),
                };
                self.program.funcs.insert(name.clone(), func);
                self.fn_names.insert(name.clone());
                // BUG-008: registra qué parámetros son `prestado mut` para
                // emitir la copia de vuelta en cada llamada.
                let mut_idx: Vec<usize> = params
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| matches!(&p.param_type, Type::Prestado { mutable: true, .. }))
                    .map(|(i, _)| i)
                    .collect();
                if !mut_idx.is_empty() {
                    self.mut_borrow_params.insert(name.clone(), mut_idx);
                }
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
                        let mangled = if trait_name.is_empty() {
                            format!("{}_{}", type_name, name)
                        } else {
                            format!("{}_{}_{}", type_name, trait_name, name)
                        };
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
                            captures: Vec::new(),
                        };
                        self.program.funcs.insert(mangled.clone(), func);
                        self.fn_names.insert(mangled.clone());
                        // BUG-020: registrar también los `prestado mut` de los
                        // métodos. Los índices son sobre la lista de parámetros
                        // ya normalizada (con `self` en la posición 0).
                        let has_explicit_self = params
                            .iter()
                            .any(|p| p.name == "self" || p.name == "yo" || p.name == "este");
                        let offset = if has_explicit_self { 0 } else { 1 };
                        let mut_idx: Vec<usize> = params
                            .iter()
                            .enumerate()
                            .filter(|(_, p)| {
                                matches!(&p.param_type, Type::Prestado { mutable: true, .. })
                            })
                            .map(|(i, _)| i + offset)
                            .collect();
                        if !mut_idx.is_empty() {
                            self.mut_borrow_params.insert(mangled.clone(), mut_idx);
                        }
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
                captures: Vec::new(),
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

        // BUG-028: el código de nivel superior termina en `Halt`, no en
        // `Return`, así que un `posponer` global se quedaba sin volcar y su
        // bloque no se ejecutaba nunca. Se emite justo antes del `Halt`.
        self.emit_deferred();
        self.deferred.clear();

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

        if has_toplevel_code && self.program.funcs.contains_key("main") {
            if let Some(main_func) = self.program.funcs.get_mut("__main__") {
                // BUG-014: sólo auto-invocar `main` si el código de nivel
                // superior no la llamó ya; en caso contrario se ejecutaba dos
                // veces (una por la llamada del usuario y otra por ésta).
                let ya_llamada = main_func
                    .instrs
                    .iter()
                    .any(|i| matches!(i, Instr::Call(name, _) if name == "main"));
                if !ya_llamada {
                    if matches!(main_func.instrs.last(), Some(Instr::Halt)) {
                        main_func.instrs.pop();
                    }
                    main_func.instrs.push(Instr::Call("main".to_string(), 0));
                    main_func.instrs.push(Instr::Halt);
                }
            }
        }

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
                    // BUG-060: mientras se compila el cuerpo de una lambda que
                    // se asigna a `name`, las referencias a `name` son
                    // recursión, no capturas del entorno.
                    let previo = if matches!(**init_expr, Expr::Lambda { .. }) {
                        self.self_binding.replace(name.clone())
                    } else {
                        self.self_binding.clone()
                    };
                    self.gen_expr(init_expr);
                    self.self_binding = previo;
                    // BUG-063: la declaracion liga en el ambito de bloque actual.
                    let slot = self.declare_in_block(name);
                    let resolved = if slot != *name {
                        slot
                    } else {
                        self.capture_map
                            .get(name)
                            .cloned()
                            .unwrap_or_else(|| name.clone())
                    };
                    // BUG-023: una declaración liga en el marco actual. Si el
                    // nombre está remapeado por una captura conservamos `Store`,
                    // porque entonces el destino es el slot de la captura y no
                    // una variable nueva.
                    if resolved == *name {
                        self.emit(Instr::StoreLocal(resolved));
                    } else {
                        self.emit(Instr::Store(resolved));
                    }
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
                // BUG-063: una función es otro marco: ámbito de bloque limpio.
                let saved_scopes = std::mem::take(&mut self.block_scopes);
                self.push_block_scope();
                // BUG-028: los `posponer` son por función; no deben filtrarse
                // a la que se estuviera generando por fuera.
                let saved_deferred = std::mem::take(&mut self.deferred);
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                self.block_scopes = saved_scopes;
                // BUG-028: caída natural por el final — volcar los diferidos.
                if !matches!(self.current_instrs.last(), Some(Instr::Return)) {
                    self.emit_deferred();
                }
                self.deferred = saved_deferred;
                // BUG-010: sólo se puede omitir el `Return` final si la ÚLTIMA
                // instrucción ya es un retorno. Comprobar `any(...)` hacía que
                // un `retornar` temprano dentro de un `si` dejara la función
                // sin terminador: la ejecución continuaba sobre las
                // instrucciones de la función siguiente / fin del bytecode.
                if !matches!(self.current_instrs.last(), Some(Instr::Return)) {
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
                        let mangled = if trait_name.is_empty() {
                            format!("{}_{}", type_name, name)
                        } else {
                            format!("{}_{}_{}", type_name, trait_name, name)
                        };
                        self.current_func = Some(mangled.clone());
                        // Cargar instrucciones si este método ya existía
                        self.current_instrs = self
                            .program
                            .funcs
                            .get(&mangled)
                            .map(|f| f.instrs.clone())
                            .unwrap_or_default();
                        self.temp_counter = 0;
                        // BUG-063: ámbito de bloque limpio también en métodos.
                        let saved_scopes = std::mem::take(&mut self.block_scopes);
                        self.push_block_scope();
                        let saved_deferred = std::mem::take(&mut self.deferred);
                        for node in body {
                            self.gen_decl_or_stmt(node);
                        }
                        self.block_scopes = saved_scopes;
                        // BUG-028: volcar los `posponer` en la salida natural.
                        if !matches!(self.current_instrs.last(), Some(Instr::Return)) {
                            self.emit_deferred();
                        }
                        self.deferred = saved_deferred;
                        // BUG-010: ver arriba — el terminador depende de la
                        // última instrucción, no de que exista algún `Return`.
                        if !matches!(self.current_instrs.last(), Some(Instr::Return)) {
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

    /// BUG-026: una variable capturada por una lambda pasa a vivir en el slot
    /// `__cap_N_x`, pero el código YA emitido antes de crear la lambda (por
    /// ejemplo la condición de un `mientras`) sigue leyendo `x`. Si sólo
    /// escribimos en el slot, esa condición nunca ve los cambios y el bucle
    /// se vuelve infinito. Mantenemos los dos nombres sincronizados: tras
    /// guardar en el slot copiamos el valor de vuelta al nombre original.
    /// BUG-063: entra en un ámbito de bloque (`si`, `mientras`, `para`, ...).
    fn push_block_scope(&mut self) {
        self.block_scopes.push(HashMap::new());
    }

    fn pop_block_scope(&mut self) {
        self.block_scopes.pop();
    }

    /// BUG-063: registra una declaración en el ámbito actual y devuelve el slot
    /// donde debe vivir. Si el nombre ya es visible fuera, se le da un slot
    /// propio para que la variable exterior sobreviva intacta.
    fn declare_in_block(&mut self, name: &str) -> String {
        let ya_visible = self
            .block_scopes
            .iter()
            .any(|scope| scope.contains_key(name));
        let slot = if ya_visible {
            let s = format!("__sh_{}_{}", self.shadow_counter, name);
            self.shadow_counter += 1;
            s
        } else {
            name.to_string()
        };
        if let Some(actual) = self.block_scopes.last_mut() {
            actual.insert(name.to_string(), slot.clone());
        }
        slot
    }

    /// BUG-063: resuelve un nombre al slot real. El ámbito de bloque manda sólo
    /// cuando de verdad renombró algo; si no, se respeta el mapa de capturas
    /// tal y como estaba antes.
    fn resolve_var(&self, name: &str) -> String {
        for scope in self.block_scopes.iter().rev() {
            if let Some(slot) = scope.get(name) {
                if slot != name {
                    return slot.clone();
                }
                break;
            }
        }
        self.capture_map
            .get(name)
            .cloned()
            .unwrap_or_else(|| name.to_string())
    }

    fn emit_store_syncing_capture(&mut self, name: &str) {
        // BUG-063: si el nombre está sombreado en un bloque, la asignación va a
        // su slot y no toca ni la variable exterior ni ninguna captura.
        let resolved = self.resolve_var(name);
        if resolved != name && !resolved.starts_with("__cap_") {
            self.emit(Instr::Store(resolved));
            return;
        }
        match self.capture_map.get(name).cloned() {
            Some(slot) => {
                self.emit(Instr::Store(slot.clone()));
                self.emit(Instr::Load(slot));
                self.emit(Instr::Store(name.to_string()));
            }
            None => self.emit(Instr::Store(name.to_string())),
        }
    }

    /// BUG-028: vuelca los bloques `posponer` pendientes en el punto de salida
    /// actual. En orden inverso al de declaración (LIFO), como un `defer`.
    fn emit_deferred(&mut self) {
        if self.deferred.is_empty() {
            return;
        }
        let blocks = self.deferred.clone();
        for block in blocks.iter().rev() {
            self.current_instrs.extend(block.iter().cloned());
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assignment { name, value, .. } => {
                self.gen_expr(value);
                // BUG-026: mantener sincronizados `x` y su slot de captura.
                self.emit_store_syncing_capture(name);
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
                self.push_block_scope();
                for node in then_body {
                    self.gen_decl_or_stmt(node);
                }
                self.pop_block_scope();
                self.emit(Instr::Jmp(end_label));
                self.emit(Instr::Label(else_label));
                if let Some(else_body) = else_body {
                    self.push_block_scope();
                    for node in else_body {
                        self.gen_decl_or_stmt(node);
                    }
                    self.pop_block_scope();
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
                self.push_block_scope();
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                self.pop_block_scope();
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
                self.push_block_scope();
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
                self.pop_block_scope();
            }
            Stmt::Return { value, .. } => {
                // BUG-028: los bloques `posponer` corren antes de retornar. Se
                // emiten ANTES de evaluar el valor de retorno para no dejar
                // basura por encima de él en la pila.
                self.emit_deferred();
                if let Some(val) = value {
                    self.gen_expr(val);
                }
                self.emit(Instr::Return);
            }
            Stmt::FieldAssign {
                expr, field, value, ..
            } => {
                // `arr[i].campo = v` — array element field mutation with write-back
                if let Expr::Index {
                    expr: base, index, ..
                } = expr.as_ref()
                {
                    // BUG-011: `lista[i].campo = v`. `ArraySet` espera la pila
                    // en orden [array, índice, valor]; emitir el struct
                    // modificado ANTES de la lista dejaba [elem, array, índice]
                    // y corrompía la asignación (error "StructGet requires
                    // struct value"). Se guarda el elemento actualizado en un
                    // temporal y se recompone la pila en el orden correcto.
                    let elem_tmp = format!("__fa_elem_{}", self.temp_counter);
                    self.temp_counter += 1;
                    self.gen_expr(base);
                    self.gen_expr(index);
                    self.emit(Instr::ArrayGet);
                    self.emit(Instr::ConstStr(field.clone()));
                    self.gen_expr(value);
                    self.emit(Instr::StructSet);
                    self.emit(Instr::Store(elem_tmp.clone()));

                    self.gen_expr(base);
                    self.gen_expr(index);
                    self.emit(Instr::Load(elem_tmp));
                    self.emit(Instr::ArraySet);
                    // BUG-094: idem para `a.b[i].campo = v`.
                    self.finish_writeback(base);
                } else if let Expr::FieldAccess {
                    expr: base,
                    field: outer_field,
                    ..
                } = expr.as_ref()
                {
                    // `r.origen.x = v` — nested struct field mutation with write-back
                    self.gen_expr(base);
                    self.emit(Instr::ConstStr(outer_field.clone()));
                    self.gen_expr(base);
                    self.emit(Instr::ConstStr(outer_field.clone()));
                    self.emit(Instr::StructGet);
                    self.emit(Instr::ConstStr(field.clone()));
                    self.gen_expr(value);
                    self.emit(Instr::StructSet);
                    self.emit(Instr::StructSet);
                    // BUG-094: idem para `a.b.c.d = v`.
                    self.finish_writeback(base);
                } else {
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
            }
            Stmt::ArraySet {
                arr, index, value, ..
            } => {
                // `m[i][j] = v`, `x.campo[i] = v` o `a[i] = v`
                if let Expr::Index {
                    expr: base,
                    index: outer_idx,
                    ..
                } = arr.as_ref()
                {
                    // `m[i][j] = v` — 2D array mutation with write-back
                    self.gen_expr(base);
                    self.gen_expr(outer_idx);
                    self.gen_expr(base);
                    self.gen_expr(outer_idx);
                    self.emit(Instr::ArrayGet);
                    self.gen_expr(index);
                    self.gen_expr(value);
                    self.emit(Instr::ArraySet);
                    self.emit(Instr::ArraySet);
                    // BUG-094: el write-back sólo se emitía cuando la base era
                    // una variable suelta. Con `m.g[i][j] = v` la base es un
                    // campo (`FieldAccess`), así que la lista modificada se
                    // quedaba en la pila y se descartaba: la asignación no
                    // hacía NADA, en silencio y sin error. `finish_writeback`
                    // sube por la cadena (`a.b[i][j]`, `a[i].b[j]`, ...).
                    self.finish_writeback(base);
                } else if let Expr::FieldAccess {
                    expr: base,
                    field: struct_field,
                    ..
                } = arr.as_ref()
                {
                    // `x.campo[i] = v` — struct field array element mutation with write-back
                    self.gen_expr(base);
                    self.emit(Instr::ConstStr(struct_field.clone()));
                    self.gen_expr(base);
                    self.emit(Instr::ConstStr(struct_field.clone()));
                    self.emit(Instr::StructGet);
                    self.gen_expr(index);
                    self.gen_expr(value);
                    self.emit(Instr::ArraySet);
                    self.emit(Instr::StructSet);
                    // BUG-094: idem para `a.b.campo[i] = v`.
                    self.finish_writeback(base);
                } else {
                    self.gen_expr(arr);
                    self.gen_expr(index);
                    self.gen_expr(value);
                    self.emit(Instr::ArraySet);
                    if let Expr::Ident { name, .. } = arr.as_ref() {
                        self.emit(Instr::Store(name.clone()));
                    }
                }
            }
            Stmt::Expr { expr, .. } => {
                // BUG-064: `agregar(l, x)` en forma de FUNCIÓN es puramente
                // funcional: apila la lista nueva y nadie la guarda, así que el
                // elemento se perdía SIN ERROR (`lumen check` daba el programa
                // por válido). La forma método `l.agregar(x)` sí escribía de
                // vuelta desde BUG-033, con lo que dos sintaxis equivalentes
                // hacían cosas distintas. Aquí, como sentencia, el valor iba a
                // descartarse igualmente: lo guardamos en el receptor en vez de
                // tirarlo.
                if let Some(receptor) = self.agregar_como_sentencia(expr.as_ref()) {
                    self.gen_expr(expr);
                    match &receptor {
                        // Variable suelta: se guarda igual que hace la forma
                        // método cuando el receptor es un `Ident`.
                        Expr::Ident { name, .. } => {
                            let destino = self.resolve_var(name);
                            self.emit(Instr::Store(destino));
                        }
                        // `c.items` / `m[i]`: reutiliza el write-back de BUG-033.
                        otro => self.emit_container_writeback(otro),
                    }
                    return;
                }
                self.gen_expr(expr);
                // BUG-027: una sentencia-expresión evalúa por su efecto
                // secundario; el valor resultante (p. ej. el `void` de
                // `imprimir(...)`) no lo consume nadie y hay que descartarlo,
                // o se queda en la pila y se mezcla con los operandos que el
                // llamador esté montando.
                self.emit(Instr::Drop);
            }
            Stmt::Posponer { body, .. } => {
                // BUG-028: `posponer` es un `defer`: su cuerpo debe ejecutarse
                // al SALIR de la función, no donde está escrito. Antes se
                // emitía en línea, así que la "limpieza" corría antes que el
                // código que usaba el recurso. Lo generamos aparte y lo
                // guardamos para volcarlo en cada punto de salida.
                let saved = std::mem::take(&mut self.current_instrs);
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                let block = std::mem::replace(&mut self.current_instrs, saved);
                self.deferred.push(block);
            }
            Stmt::TryCatch {
                try_body,
                err_var,
                catch_body,
                ..
            } => {
                // BUG-022: antes se emitía la etiqueta del `atrapar` pero NADIE
                // saltaba a ella y `err_var` se ignoraba, así que el bloque era
                // código muerto y cualquier error abortaba el programa. Ahora
                // se instala un manejador de verdad: `PushHandler` registra el
                // destino, y la VM salta ahí desenrollando la pila cuando algo
                // falla.
                let catch_label = self.new_label();
                let end_label = self.new_label();

                self.emit(Instr::PushHandler(catch_label));
                for node in try_body {
                    self.gen_decl_or_stmt(node);
                }
                // El `intentar` terminó bien: el manejador ya no aplica.
                self.emit(Instr::PopHandler);
                self.emit(Instr::Jmp(end_label));

                self.emit(Instr::Label(catch_label));
                // La VM deja el error en la cima de la pila; se liga a la
                // variable del `atrapar (e)`. `StoreLocal` para que no pise una
                // global homónima (BUG-023).
                self.emit(Instr::StoreLocal(err_var.clone()));
                for node in catch_body {
                    self.gen_decl_or_stmt(node);
                }
                self.emit(Instr::Label(end_label));
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
                let mut fail_label = self.new_label();
                for arm in arms {
                    self.emit(Instr::Label(fail_label));
                    fail_label = self.new_label();
                    let body_label = self.new_label();
                    // Patterns: cada uno salta al body si matchea; si ninguno
                    // matchea, el fallthrough llega al Jmp(fail) del final.
                    self.emit_match_pattern(expr, &arm.value, fail_label, body_label);
                    for alt in &arm.alt_values {
                        self.emit_match_pattern(expr, alt, fail_label, body_label);
                    }
                    self.emit(Instr::Jmp(fail_label));
                    self.emit(Instr::Label(body_label));
                    if let Some(ref guard_expr) = arm.guard {
                        self.gen_expr(guard_expr);
                        self.emit(Instr::JmpIf(fail_label));
                    }
                    for node in &arm.body {
                        self.gen_decl_or_stmt(node);
                    }
                    self.emit(Instr::Jmp(end_label));
                }
                self.emit(Instr::Label(fail_label));
                if let Some(default_body) = default {
                    for node in default_body {
                        self.gen_decl_or_stmt(node);
                    }
                }
                self.emit(Instr::Label(end_label));
            }
            Stmt::Block { stmts, .. } => {
                self.push_block_scope();
                for node in stmts {
                    self.gen_decl_or_stmt(node);
                }
                self.pop_block_scope();
            }
            Stmt::ForEach {
                var_name,
                expr,
                body,
                ..
            } => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                let continue_label = self.new_label();
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
                // BUG-015: registrar el ciclo para que 'romper'/'continuar'
                // tengan destino. 'continuar' debe saltar al incremento del
                // índice (no a start_label), o el bucle no avanzaría nunca.
                self.loop_labels.push(LoopLabels {
                    break_label: end_label,
                    continue_label,
                    loop_name: None,
                });
                self.push_block_scope();
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                self.pop_block_scope();
                self.loop_labels.pop();
                self.emit(Instr::Label(continue_label));
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
                let temp = format!("__mt_{}", self.temp_counter);
                self.temp_counter += 1;
                self.gen_expr(value);
                self.emit(Instr::Store(temp.clone()));
                let el = self.new_label();
                let end_l = self.new_label();
                self.emit_if_let_pattern(&temp, pattern, el);
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
                let temp = format!("__mt_{}", self.temp_counter);
                self.temp_counter += 1;
                self.gen_expr(value);
                self.emit(Instr::Store(temp.clone()));
                let else_l = self.new_label();
                let end_l = self.new_label();
                self.emit_if_let_pattern(&temp, pattern, else_l);
                self.emit(Instr::Jmp(end_l));
                self.emit(Instr::Label(else_l));
                for n in else_body {
                    self.gen_decl_or_stmt(n);
                }
                self.emit(Instr::Label(end_l));
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
            Stmt::InlineAsm { code, .. } => {
                self.emit(Instr::ConstStr(code.clone()));
                self.emit(Instr::Call("__ffi_asm".to_string(), 1));
            }
            Stmt::InlineC { code, .. } => {
                self.emit(Instr::ConstStr(code.clone()));
                self.emit(Instr::Call("__ffi_c_eval".to_string(), 1));
            }
            Stmt::InlineRust { code, .. } => {
                self.emit(Instr::ConstStr(code.clone()));
                self.emit(Instr::Call("__ffi_rust_eval".to_string(), 1));
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
                // BUG-063: el ambito de bloque tiene prioridad sobre el mapa de capturas.
                let resolved = self.resolve_var(name);
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
                    match op {
                        BinOp::And => {
                            let temp = format!("__sc_{}", self.temp_counter);
                            self.temp_counter += 1;
                            let false_label = self.new_label();
                            let end_label = self.new_label();
                            self.gen_expr(left);
                            self.emit(Instr::Store(temp.clone()));
                            self.emit(Instr::Load(temp.clone()));
                            self.emit(Instr::JmpIf(false_label));
                            self.gen_expr(right);
                            self.emit(Instr::Jmp(end_label));
                            self.emit(Instr::Label(false_label));
                            self.emit(Instr::Load(temp));
                            self.emit(Instr::Label(end_label));
                        }
                        BinOp::Or => {
                            let temp = format!("__sc_{}", self.temp_counter);
                            self.temp_counter += 1;
                            let eval_label = self.new_label();
                            let end_label = self.new_label();
                            self.gen_expr(left);
                            self.emit(Instr::Store(temp.clone()));
                            self.emit(Instr::Load(temp.clone()));
                            self.emit(Instr::JmpIf(eval_label));
                            self.emit(Instr::Load(temp));
                            self.emit(Instr::Jmp(end_label));
                            self.emit(Instr::Label(eval_label));
                            self.gen_expr(right);
                            self.emit(Instr::Label(end_label));
                        }
                        _ => {
                            self.gen_expr(left);
                            self.gen_expr(right);
                            self.emit(Instr::Binary(match op {
                                BinOp::Add => Op::Add,
                                BinOp::Concat => Op::Concat,
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
                                BinOp::BitOr => Op::BitOr,
                                BinOp::BitAnd => Op::BitAnd,
                                BinOp::BitXor => Op::BitXor,
                                BinOp::ShiftLeft => Op::ShiftLeft,
                                BinOp::ShiftRight => Op::ShiftRight,
                                _ => unreachable!(),
                            }));
                        }
                    }
                }
            }
            Expr::Unary { op, operand, .. } => {
                self.gen_expr(operand);
                self.emit(Instr::Unary(match op {
                    UnOp::Negate => Op::Negate,
                    UnOp::Not => Op::Not,
                    UnOp::BitNot => Op::BitNot,
                }));
            }
            Expr::Call { callee, args, .. } => {
                let callee_inner = match callee.as_ref() {
                    Expr::Grouping { expr, .. } => expr.as_ref(),
                    Expr::Cast { expr, .. } => expr.as_ref(),
                    other => other,
                };
                match callee_inner {
                    Expr::Ident { name, .. } => {
                        if self.fn_names.contains(name)
                            || name.starts_with("__")
                            || matches!(
                                name.as_str(),
                                "imprimir"
                                    | "print"
                                    | "leer"
                                    | "read"
                                    | "a_texto"
                                    | "to_texto"
                                    | "__str_from"
                                    // Conversiones y matemáticas públicas (BUG-001/002/007)
                                    | "a_entero"
                                    | "to_int"
                                    | "to_entero"
                                    | "a_decimal"
                                    | "to_float"
                                    | "a_numero"
                                    | "to_number"
                                    | "a_entero_seguro"
                                    | "to_int_safe"
                                    | "a_decimal_seguro"
                                    | "to_float_safe"
                                    | "es_numero"
                                    | "is_number"
                                    | "abs"
                                    | "absoluto"
                                    | "minimo"
                                    | "min"
                                    | "maximo"
                                    | "max"
                                    | "raiz"
                                    | "sqrt"
                                    | "potencia"
                                    | "pow"
                                    | "piso"
                                    | "floor"
                                    | "techo"
                                    | "ceil"
                                    | "redondear"
                                    | "round"
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
                                    | "__str_chr"
                                    | "__str_caracter"
                                    | "__file_read"
                                    | "__leer_archivo"
                                    | "__file_write"
                                    | "__escribir_archivo"
                                    | "__file_append"
                                    | "__agregar_archivo"
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
                                    | "__ffi_llamar_nv"
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
                                    | "__file_write_binary"
                                    | "__escribir_archivo_bin"
                                    | "__str_slice"
                                    | "__str_subcadena"
                                    | "__str_concat_list"
                                    | "__str_concatenar_lista"
                                    | "__str_starts_with"
                                    | "__str_empieza_con"
                                    | "__str_to_chars"
                                    | "__str_a_caracteres"
                                    | "__str_reemplazar"
                                    | "__str_replace"
                                    | "__str_subcadena_chars"
                                    | "__str_slice_chars"
                                    | "__str_a_entero"
                                    | "__texto_a_entero"
                                    | "__num_a_f64_bytes"
                                    | "__numero_a_bytes_f64"
                                    | "__file_bytes"
                                    | "__leer_bytes"
                                    | "__a_f64_bytes"
                                    | "__bytes_a_f64"
                                    | "__codegen_a_nvc"
                                    | "__compile_nv"
                                    | "__compilar_nv"
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
                            // BUG-008: copia de vuelta de los parámetros
                            // `prestado mut`. La VM conserva el marco de la
                            // llamada recién retornada, así que se lee el valor
                            // final del parámetro y se guarda en la variable
                            // que el llamador pasó como argumento.
                            self.emit_mut_borrow_writeback(name, args);
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
            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                let i_temp = format!("__rng_i_{}", self.temp_counter);
                self.temp_counter += 1;
                let cmp_temp = format!("__rng_c_{}", self.temp_counter);
                self.temp_counter += 1;
                self.emit(Instr::ArrayNew(0));
                self.gen_expr(start);
                self.emit(Instr::Store(i_temp.clone()));
                self.emit(Instr::Label(start_label));
                self.emit(Instr::Load(i_temp.clone()));
                self.emit(Instr::Store(cmp_temp.clone()));
                self.emit(Instr::Load(cmp_temp.clone()));
                self.gen_expr(end);
                self.emit(Instr::Binary(if *inclusive {
                    Op::LessEqual
                } else {
                    Op::Less
                }));
                self.emit(Instr::JmpIf(end_label));
                self.emit(Instr::Load(cmp_temp.clone()));
                self.emit(Instr::ArrayPush);
                self.emit(Instr::Load(cmp_temp.clone()));
                self.emit(Instr::ConstInt(1));
                self.emit(Instr::Binary(Op::Add));
                self.emit(Instr::Store(i_temp.clone()));
                self.emit(Instr::Jmp(start_label));
                self.emit(Instr::Label(end_label));
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
                    self.emit(Instr::Call(fname.clone(), args.len() + 1));
                    // BUG-020: en una llamada a método el receptor es el
                    // parámetro 0. Si está declarado `prestado mut self`, hay
                    // que copiarle de vuelta el valor final igual que se hace
                    // con las funciones libres, o la mutación se pierde.
                    let mut recv_args: Vec<Expr> = Vec::with_capacity(args.len() + 1);
                    recv_args.push(expr.as_ref().clone());
                    recv_args.extend(args.iter().cloned());
                    self.emit_mut_borrow_writeback(&fname, &recv_args);
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
                            } else {
                                // BUG-033: `c.items.agregar(x)` y `m[i].agregar(x)`
                                // hacían el `ArrayPush` pero nunca guardaban la
                                // lista resultante: el receptor no era un `Ident`,
                                // así que `var_name` era `None` y la mutación se
                                // perdía SIN ERROR ALGUNO. Como `ArrayPush` deja la
                                // lista actualizada en la pila, la escribimos de
                                // vuelta en su sitio reutilizando la misma
                                // maquinaria que las asignaciones normales.
                                self.emit_container_writeback(expr.as_ref());
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
            Expr::Cast { expr, .. } => {
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
            Expr::SafeFieldAccess { expr, field, .. } => {
                self.gen_expr(expr);
                self.emit(Instr::ConstStr(field.clone()));
                self.emit(Instr::StructGet);
            }
            Expr::Elvis { expr, default, .. } => {
                let else_label = self.new_label();
                let end_label = self.new_label();
                self.gen_expr(expr);
                self.emit(Instr::JmpIf(else_label));
                self.gen_expr(expr);
                self.emit(Instr::Jmp(end_label));
                self.emit(Instr::Label(else_label));
                self.gen_expr(default);
                self.emit(Instr::Label(end_label));
            }
            Expr::Comprehension {
                expr: inner_expr,
                var_name,
                iter,
                condition,
                ..
            } => {
                let out_arr = format!("__comp_out_{}", self.temp_counter);
                let i_temp = format!("__comp_i_{}", self.temp_counter);
                let len_temp = format!("__comp_len_{}", self.temp_counter);
                let iter_temp = format!("__comp_iter_{}", self.temp_counter);
                self.temp_counter += 1;

                self.gen_expr(iter);
                self.emit(Instr::Store(iter_temp.clone()));

                self.emit(Instr::Load(iter_temp.clone()));
                self.emit(Instr::ArrayLen);
                self.emit(Instr::Store(len_temp.clone()));

                self.emit(Instr::ArrayNew(0));
                self.emit(Instr::Store(out_arr.clone()));

                self.emit(Instr::ConstInt(0));
                self.emit(Instr::Store(i_temp.clone()));

                let loop_start = self.new_label();
                let loop_end = self.new_label();
                let skip_label = self.new_label();

                self.emit(Instr::Label(loop_start));
                self.emit(Instr::Load(i_temp.clone()));
                self.emit(Instr::Load(len_temp.clone()));
                self.emit(Instr::Binary(Op::Less));
                self.emit(Instr::JmpIf(loop_end));

                self.emit(Instr::Load(iter_temp.clone()));
                self.emit(Instr::Load(i_temp.clone()));
                self.emit(Instr::ArrayGet);
                self.emit(Instr::Store(var_name.clone()));

                if let Some(cond) = condition {
                    self.gen_expr(cond);
                    self.emit(Instr::JmpIf(skip_label));
                }

                self.emit(Instr::Load(out_arr.clone()));
                self.gen_expr(inner_expr);
                self.emit(Instr::ArrayPush);
                self.emit(Instr::Store(out_arr.clone()));

                self.emit(Instr::Label(skip_label));
                self.emit(Instr::Load(i_temp.clone()));
                self.emit(Instr::ConstInt(1));
                self.emit(Instr::Binary(Op::Add));
                self.emit(Instr::Store(i_temp.clone()));

                self.emit(Instr::Jmp(loop_start));
                self.emit(Instr::Label(loop_end));

                self.emit(Instr::Load(out_arr.clone()));
            }
            Expr::Query {
                var_name,
                source,
                where_clause,
                order_by: _,
                descending: _,
                select_expr,
                ..
            } => {
                let out_arr = format!("__q_out_{}", self.temp_counter);
                let i_temp = format!("__q_i_{}", self.temp_counter);
                let len_temp = format!("__q_len_{}", self.temp_counter);
                let src_temp = format!("__q_src_{}", self.temp_counter);
                self.temp_counter += 1;

                self.gen_expr(source);
                self.emit(Instr::Store(src_temp.clone()));

                self.emit(Instr::Load(src_temp.clone()));
                self.emit(Instr::ArrayLen);
                self.emit(Instr::Store(len_temp.clone()));

                self.emit(Instr::ArrayNew(0));
                self.emit(Instr::Store(out_arr.clone()));

                self.emit(Instr::ConstInt(0));
                self.emit(Instr::Store(i_temp.clone()));

                let loop_start = self.new_label();
                let loop_end = self.new_label();
                let skip_label = self.new_label();

                self.emit(Instr::Label(loop_start));
                self.emit(Instr::Load(i_temp.clone()));
                self.emit(Instr::Load(len_temp.clone()));
                self.emit(Instr::Binary(Op::Less));
                self.emit(Instr::JmpIf(loop_end));

                self.emit(Instr::Load(src_temp.clone()));
                self.emit(Instr::Load(i_temp.clone()));
                self.emit(Instr::ArrayGet);
                self.emit(Instr::Store(var_name.clone()));

                if let Some(cond) = where_clause {
                    self.gen_expr(cond);
                    self.emit(Instr::JmpIf(skip_label));
                }

                self.emit(Instr::Load(out_arr.clone()));
                self.gen_expr(select_expr);
                self.emit(Instr::ArrayPush);
                self.emit(Instr::Store(out_arr.clone()));

                self.emit(Instr::Label(skip_label));
                self.emit(Instr::Load(i_temp.clone()));
                self.emit(Instr::ConstInt(1));
                self.emit(Instr::Binary(Op::Add));
                self.emit(Instr::Store(i_temp.clone()));

                self.emit(Instr::Jmp(loop_start));
                self.emit(Instr::Label(loop_end));

                self.emit(Instr::Load(out_arr.clone()));
            }
            Expr::Comptime { expr, .. } => {
                self.gen_expr(expr);
            }
        }
    }

    fn compile_lambda(&mut self, params: &[Param], body: &[DeclOrStmt]) -> String {
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        let mut captured = Vec::new();
        self.collect_variable_refs(body, &param_names, &mut captured);
        // BUG-060: quitar la autorreferencia de una lambda recursiva.
        if let Some(ref propio) = self.self_binding {
            captured.retain(|v| v != propio);
        }
        // BUG-021: `collect_variable_refs` también reporta los nombres que el
        // cuerpo *asigna* (así es como se declara una lambda anidada:
        // `interna = funcion() {...};`). Ésos son locales de la lambda, no
        // capturas del entorno: si se renombran a `__cap_N_x` el `Store` guarda
        // en `x` y la lectura busca `__cap_N_x`, que nunca existe.
        let mut locales = Vec::new();
        collect_assigned_names(body, &mut locales);
        // BUG-052: dentro de otra función hay que afinar. `collect_assigned_names`
        // mezcla las DECLARACIONES (`entero x = 0;`) con las simples
        // ASIGNACIONES (`n = n + 1;`). Para el contador clásico eso es fatal:
        // `n = n + 1` sobre la `n` del entorno es una mutación de la captura,
        // no una local nueva, y descartarla dejaba a la closure sin `n`.
        // Sólo las declaraciones de verdad son locales de la lambda.
        // BUG-148: mismo criterio que abajo — dentro de cualquier marco propio
        // sólo las declaraciones de verdad son locales de la lambda.
        let dentro_de_funcion = self.is_in_lambda
            || self
                .current_func
                .as_deref()
                .is_some_and(|f| f != "__main__");
        if dentro_de_funcion {
            let mut declaradas = Vec::new();
            collect_declared_names(body, &mut declaradas);
            captured.retain(|v| !declaradas.contains(v));
        } else {
            captured.retain(|v| !locales.contains(v));
        }
        // BUG-032: una lambda creada DENTRO de otra función (o de otra lambda)
        // no puede usar los slots globales `__cap_*`: dos closures fabricadas
        // por la misma factoría compartirían el slot y devolverían valores
        // erróneos en silencio (`mk(5)` y `mk(100)` daban ambas 101). Para ese
        // caso se anota la lista de nombres capturados en la propia función; la
        // VM los resuelve al crear la closure (`FuncRef`) y se los lleva
        // consigo, así que la closure sigue siendo válida cuando el marco que
        // la creó ya ha muerto. Cada `FuncRef` produce un entorno propio, de
        // modo que las instancias quedan aisladas entre sí.
        // BUG-148: la elección de mecanismo dependía sólo de `is_in_lambda`,
        // que nunca se activa al compilar una función normal. Una lambda
        // declarada dentro de `funcion crear() { ... }` caía por tanto en la
        // rama de slots globales `__cap_N_x`, que se llenan en el marco de la
        // función envolvente y mueren con él: al devolver la closure y
        // llamarla desde fuera fallaba con «Variable '__cap_1_n' no definida».
        // Y con dos closures sobre la misma variable, la segunda reutilizaba
        // el `capture_map` de la primera y las mutaciones dejaban de verse.
        // El criterio correcto es «estoy dentro de algún marco», no «dentro de
        // una lambda»: `__main__` es el único sitio donde los slots globales
        // sobreviven.
        let en_marco_propio = self.is_in_lambda
            || self
                .current_func
                .as_deref()
                .is_some_and(|f| f != "__main__");
        let mut env_captures: Vec<String> = Vec::new();
        if en_marco_propio {
            for var_name in &captured {
                if !param_names.contains(var_name)
                    && !env_captures.contains(var_name)
                    // Las funciones de nivel superior se resuelven por nombre,
                    // no son variables que haya que capturar.
                    && !self.program.funcs.contains_key(var_name)
                {
                    env_captures.push(var_name.clone());
                }
            }
        }
        if !en_marco_propio {
            for var_name in &captured {
                if !param_names.contains(var_name) && !self.capture_map.contains_key(var_name) {
                    let cap_name = format!("__cap_{}_{}", self.lambda_counter, var_name);
                    // BUG-017: registrar el renombrado no bastaba — el `Store`
                    // de la variable original ya se había emitido, así que el
                    // slot `__cap_N_x` nunca se llenaba y la lambda fallaba con
                    // "Variable '__cap_N_x' no definida". Copiamos aquí el valor
                    // al slot (captura por valor en el momento de crear la
                    // lambda). Se emite en la función envolvente, que es la que
                    // sigue activa en `current_instrs`.
                    self.emit(Instr::Load(var_name.clone()));
                    self.emit(Instr::Store(cap_name.clone()));
                    self.capture_map.insert(var_name.clone(), cap_name);
                }
            }
        }

        let func = Func {
            name: lambda_name.clone(),
            params: param_names,
            entry: 0,
            instrs: Vec::new(),
            captures: env_captures,
        };
        self.program.funcs.insert(lambda_name.clone(), func);
        let saved_instrs = std::mem::take(&mut self.current_instrs);
        let saved_func = self.current_func.clone();
        let saved_temp = self.temp_counter;
        let saved_label = self.label_counter;
        let saved_loop = std::mem::take(&mut self.loop_labels);
        let saved_is_lambda = self.is_in_lambda;
        let saved_deferred = std::mem::take(&mut self.deferred);
        self.current_func = Some(lambda_name.clone());
        self.current_instrs = Vec::new();
        self.temp_counter = 0;
        // BUG-062: NO se reinicia `label_counter`. `codegen` resuelve las
        // etiquetas con un ÚNICO mapa global `label -> posición`, así que si la
        // lambda vuelve a numerar desde 0 su `L0` sobrescribe el `L0` de la
        // función envolvente y los saltos aterrizan en otra función. El síntoma
        // era desconcertante: un `si/sino` antes de una lambda recursiva
        // imprimía las DOS ramas, y la recursión no terminaba nunca.
        self.is_in_lambda = true;
        // BUG-063: la lambda es otro marco: ámbito de bloque limpio. Las
        // capturas siguen resolviéndose por `capture_map`, que no se toca.
        let saved_scopes = std::mem::take(&mut self.block_scopes);
        self.push_block_scope();
        for node in body {
            self.gen_decl_or_stmt(node);
        }
        self.block_scopes = saved_scopes;
        // BUG-028: idem para lambdas — sus `posponer` son suyos.
        if !matches!(self.current_instrs.last(), Some(Instr::Return)) {
            self.emit_deferred();
        }
        self.deferred = saved_deferred;
        // BUG-010: idem para lambdas.
        if !matches!(self.current_instrs.last(), Some(Instr::Return)) {
            self.emit(Instr::Return);
        }
        self.finalize_func();
        self.current_func = saved_func;
        self.current_instrs = saved_instrs;
        self.temp_counter = saved_temp;
        // `label_counter` se deja como está: las etiquetas deben ser únicas en
        // todo el programa, no por función.
        let _ = saved_label;
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
            Stmt::Posponer { body, .. } => self.collect_variable_refs(body, params, out),
            Stmt::TryCatch {
                try_body,
                catch_body,
                ..
            } => {
                self.collect_variable_refs(try_body, params, out);
                self.collect_variable_refs(catch_body, params, out);
            }
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
                // BUG-029: el destino de una llamada escrito como identificador
                // simple (`doblar(21)`, `imprimir(...)`) es un NOMBRE DE
                // FUNCIÓN, no una variable del entorno. Si se apunta como
                // captura, la lambda intenta leer `__cap_N_imprimir` y muere
                // con "Variable 'imprimir' no definida". Sólo descendemos
                // cuando el callee es una expresión de verdad (una lambda
                // guardada en una variable, `f()` donde `f` es un valor, etc.),
                // que se detecta porque el nombre no es una función conocida.
                match callee.as_ref() {
                    Expr::Ident { name, .. } => {
                        if !self.fn_names.contains(name)
                            && !name.starts_with("__")
                            && !is_public_builtin_name(name)
                        {
                            self.collect_expr_refs(callee, params, out);
                        }
                    }
                    other => self.collect_expr_refs(other, params, out),
                }
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
            Expr::Cast { expr, .. } => self.collect_expr_refs(expr, params, out),
            Expr::StructInit { fields, .. } => {
                for (_, v) in fields {
                    self.collect_expr_refs(v, params, out);
                }
            }
            Expr::FieldAccess { expr, .. } => self.collect_expr_refs(expr, params, out),
            Expr::SafeFieldAccess { expr, .. } => self.collect_expr_refs(expr, params, out),
            Expr::Elvis { expr, default, .. } => {
                self.collect_expr_refs(expr, params, out);
                self.collect_expr_refs(default, params, out);
            }
            Expr::Comprehension {
                expr,
                iter,
                condition,
                ..
            } => {
                self.collect_expr_refs(expr, params, out);
                self.collect_expr_refs(iter, params, out);
                if let Some(cond) = condition {
                    self.collect_expr_refs(cond, params, out);
                }
            }
            Expr::Query {
                source,
                where_clause,
                order_by,
                select_expr,
                ..
            } => {
                self.collect_expr_refs(source, params, out);
                if let Some(w) = where_clause {
                    self.collect_expr_refs(w, params, out);
                }
                if let Some(o) = order_by {
                    self.collect_expr_refs(o, params, out);
                }
                self.collect_expr_refs(select_expr, params, out);
            }
            Expr::Tuple { items, .. } => {
                for i in items {
                    self.collect_expr_refs(i, params, out);
                }
            }
            // BUG-032: hay que descender en las lambdas anidadas. Con tres
            // niveles (`externa -> media -> interna`), la lambda del medio no
            // menciona `n` en su propio cuerpo, sólo la más interna lo usa; si
            // no se mira dentro, `media` no captura `n` y no puede pasárselo a
            // `interna`, que muere con "Variable 'n' no definida". Los
            // parámetros de la lambda interior son suyos, así que se excluyen.
            Expr::Lambda {
                params: inner_params,
                body,
                ..
            } => {
                let mut inner_scope: Vec<String> = params.to_vec();
                for p in inner_params {
                    if !inner_scope.contains(&p.name) {
                        inner_scope.push(p.name.clone());
                    }
                }
                // Lo que la lambda interior declara es suyo, no del entorno.
                let mut inner_locals = Vec::new();
                collect_assigned_names(body, &mut inner_locals);
                for l in inner_locals {
                    if !inner_scope.contains(&l) {
                        inner_scope.push(l);
                    }
                }
                let mut inner_refs = Vec::new();
                self.collect_variable_refs(body, &inner_scope, &mut inner_refs);
                for r in inner_refs {
                    if !params.contains(&r) && !out.contains(&r) {
                        out.push(r);
                    }
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

    // Genera la comprobación de un patrón contra `expr` en un `elegir`.
    // JmpIf salta cuando el tope es FALSY (vm.rs), por lo que:
    // - Patrón concreto: `x != pat` → si NO matchea no salta (caen los checks
    //   siguientes); si matchea, NotEqual es falso → salta al body.
    // - Rango (`0..5`): fuera de rango → salta a fail; dentro → Jmp al body.
    fn emit_if_let_pattern(&mut self, temp: &str, pattern: &Expr, else_label: usize) {
        // Carga el valor capturado en `temp` y emite el test de tipo + bindings.
        self.emit(Instr::Load(temp.to_string()));
        match pattern {
            Expr::Algun { expr, .. } => {
                self.emit(Instr::MatchType(0));
                self.emit(Instr::JmpIf(else_label));
                self.bind_payload(temp, expr);
            }
            Expr::Exito { expr, .. } => {
                self.emit(Instr::MatchType(1));
                self.emit(Instr::JmpIf(else_label));
                self.bind_payload(temp, expr);
            }
            Expr::Error { expr, .. } => {
                self.emit(Instr::MatchType(2));
                self.emit(Instr::JmpIf(else_label));
                self.bind_payload(temp, expr);
            }
            Expr::Ninguno { .. } => {
                self.emit(Instr::MatchType(0));
                self.emit(Instr::Unary(Op::Not));
                self.emit(Instr::JmpIf(else_label));
            }
            Expr::Ident { name, .. } => {
                self.emit(Instr::Load(temp.to_string()));
                self.emit(Instr::Store(name.clone()));
            }
            _ => {
                // Fallback: comparación por igualdad (pattern como valor).
                self.emit(Instr::Load(temp.to_string()));
                self.gen_expr(pattern);
                self.emit(Instr::Binary(Op::Equal));
                self.emit(Instr::JmpIf(else_label));
            }
        }
    }

    fn bind_payload(&mut self, temp: &str, pattern: &Expr) {
        self.emit(Instr::Load(temp.to_string()));
        self.emit(Instr::MatchPayload);
        match pattern {
            Expr::Ident { name, .. } => {
                self.emit(Instr::Store(name.clone()));
            }
            Expr::List { items, .. } => {
                let t2 = format!("__mt_{}", self.temp_counter);
                self.temp_counter += 1;
                self.emit(Instr::Store(t2.clone()));
                for (i, it) in items.iter().enumerate() {
                    if let Expr::Ident { name, .. } = it {
                        self.emit(Instr::Load(t2.clone()));
                        self.emit(Instr::TupleAccess(i));
                        self.emit(Instr::Store(name.clone()));
                    }
                }
            }
            _ => {}
        }
    }

    /// BUG-008: tras llamar a una función con parámetros `prestado mut`, copia
    /// el valor final de cada uno de esos parámetros a la variable que el
    /// llamador pasó, de modo que la mutación sea visible fuera de la función.
    ///
    /// Sólo se escribe de vuelta cuando el argumento es una variable simple
    /// (un *lvalue*); pasar un literal o el resultado de una expresión no tiene
    /// destino al que copiar y simplemente se ignora.
    /// BUG-033: guarda el valor que hay en lo alto de la pila de vuelta en el
    /// contenedor del que salió `receiver` (`base.campo` o `base[i]`), y repite
    /// el proceso hacia arriba hasta llegar a una variable con nombre. Sin esto
    /// las mutaciones sobre listas alcanzadas a través de un campo o de un
    /// índice se descartaban en silencio.
    /// BUG-064: reconoce `agregar(receptor, x)` / `push(...)` usado como
    /// SENTENCIA y devuelve el receptor si es un destino al que se puede
    /// escribir de vuelta. Sólo se activa en posición de sentencia: si alguien
    /// usa el valor de retorno (`sea n = agregar(l, x);`) se respeta la
    /// semántica funcional de siempre.
    fn agregar_como_sentencia(&self, expr: &Expr) -> Option<Expr> {
        let Expr::Call { callee, args, .. } = expr else {
            return None;
        };
        let Expr::Ident { name, .. } = callee.as_ref() else {
            return None;
        };
        if (name != "agregar" && name != "push") || args.len() != 2 {
            return None;
        }
        // El usuario puede haber definido su propia función con ese nombre.
        if self.fn_names.contains(name.as_str()) {
            return None;
        }
        match args.first()? {
            e @ (Expr::Ident { .. } | Expr::FieldAccess { .. } | Expr::Index { .. }) => {
                Some(e.clone())
            }
            _ => None,
        }
    }

    fn emit_container_writeback(&mut self, receiver: &Expr) {
        match receiver {
            // `base.campo` : [.. , nuevo_valor] -> StructSet sobre `base`
            Expr::FieldAccess {
                expr: base, field, ..
            } => {
                let tmp = format!("__wb_{}", self.temp_counter);
                self.temp_counter += 1;
                self.emit(Instr::Store(tmp.clone()));
                self.gen_expr(base);
                self.emit(Instr::ConstStr(field.clone()));
                self.emit(Instr::Load(tmp));
                self.emit(Instr::StructSet);
                self.finish_writeback(base);
            }
            // `base[i]` : [.. , nuevo_valor] -> ArraySet sobre `base`
            Expr::Index {
                expr: base, index, ..
            } => {
                let tmp = format!("__wb_{}", self.temp_counter);
                self.temp_counter += 1;
                self.emit(Instr::Store(tmp.clone()));
                self.gen_expr(base);
                self.gen_expr(index);
                self.emit(Instr::Load(tmp));
                self.emit(Instr::ArraySet);
                self.finish_writeback(base);
            }
            // Cualquier otra cosa (p. ej. el retorno de una llamada) no tiene
            // sitio donde volver: se descarta como antes.
            _ => {}
        }
    }

    /// Cierra un write-back: si la base es una variable la guarda, y si es otro
    /// contenedor sigue subiendo recursivamente (`a.b.c`, `a[i].b`, ...).
    fn finish_writeback(&mut self, base: &Expr) {
        match base {
            Expr::Ident { name, .. } => {
                let name = name.clone();
                self.emit_store_syncing_capture(&name);
            }
            other => self.emit_container_writeback(other),
        }
    }

    fn emit_mut_borrow_writeback(&mut self, fn_name: &str, args: &[Expr]) {
        let Some(indices) = self.mut_borrow_params.get(fn_name).cloned() else {
            return;
        };
        for idx in indices {
            let Some(arg) = args.get(idx) else {
                continue;
            };
            match arg {
                Expr::Ident { name, .. } => {
                    let target = self
                        .capture_map
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.clone());
                    // `__frame_param(i)` devuelve el valor del parámetro `i`
                    // del marco que acaba de retornar (la VM lo conserva hasta
                    // esta lectura).
                    self.emit(Instr::ConstInt(idx as i64));
                    self.emit(Instr::Call("__frame_param".to_string(), 1));
                    self.emit(Instr::Store(target));
                }
                // BUG-147: sólo se copiaba de vuelta cuando el argumento era
                // una variable simple. Al pasar `s.l` o `l[0]` a un parámetro
                // `prestado mut`, el `continue` descartaba la mutación **en
                // silencio**: el programa compilaba, `check` lo daba por bueno
                // y la llamada no tenía ningún efecto. La maquinaria para
                // escribir en `base.campo` y `base[i]` ya existía —la usa
                // `agregar` desde BUG-033/064—, así que se reutiliza.
                Expr::FieldAccess { .. } | Expr::Index { .. } => {
                    self.emit(Instr::ConstInt(idx as i64));
                    self.emit(Instr::Call("__frame_param".to_string(), 1));
                    self.emit_container_writeback(arg);
                }
                // Un literal o el resultado de una expresión no tiene destino
                // al que copiar; se ignora igual que antes.
                _ => continue,
            }
        }
    }

    fn emit_match_pattern(
        &mut self,
        expr: &Expr,
        pattern: &Expr,
        fail_label: usize,
        body_label: usize,
    ) {
        match pattern {
            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                self.gen_expr(expr);
                self.gen_expr(start);
                self.emit(Instr::Binary(Op::GreaterEqual));
                self.emit(Instr::JmpIf(fail_label));
                self.gen_expr(expr);
                self.gen_expr(end);
                self.emit(Instr::Binary(if *inclusive {
                    Op::LessEqual
                } else {
                    Op::Less
                }));
                self.emit(Instr::JmpIf(fail_label));
                self.emit(Instr::Jmp(body_label));
            }
            Expr::Algun { .. } | Expr::Exito { .. } | Expr::Error { .. } => {
                let temp = format!("__mt_{}", self.temp_counter);
                self.temp_counter += 1;
                self.gen_expr(expr);
                self.emit(Instr::Store(temp.clone()));
                self.emit_if_let_pattern(&temp, pattern, fail_label);
                self.emit(Instr::Jmp(body_label));
            }
            Expr::Tuple { items, .. } => {
                let temp = format!("__mt_tup_{}", self.temp_counter);
                self.temp_counter += 1;
                self.gen_expr(expr);
                self.emit(Instr::Store(temp.clone()));
                for (idx, item) in items.iter().enumerate() {
                    match item {
                        Expr::Ident { name, .. } if name != "_" => {
                            self.emit(Instr::Load(temp.clone()));
                            self.emit(Instr::TupleAccess(idx));
                            self.emit(Instr::Store(name.clone()));
                        }
                        Expr::Ident { name, .. } if name == "_" => {}
                        _ => {
                            self.emit(Instr::Load(temp.clone()));
                            self.emit(Instr::TupleAccess(idx));
                            self.gen_expr(item);
                            self.emit(Instr::Binary(Op::Equal));
                            self.emit(Instr::JmpIf(fail_label));
                        }
                    }
                }
                self.emit(Instr::Jmp(body_label));
            }
            // BUG-003: `caso Figura::Circulo(r):` — compara la variante y liga
            // los datos capturados a variables antes de ejecutar el cuerpo.
            // Sin argumentos (`caso Color::Rojo:`) sigue siendo una comparación
            // de variante, pero por tag en vez de igualdad estructural, para que
            // funcione igual con variantes que llevan datos.
            Expr::EnumCtor { variant, args, .. } => {
                // Convención de `emit_match_pattern`: si el patrón NO coincide
                // se cae al siguiente test (patrones OR: `caso A | B:`); sólo
                // se salta a `body_label` cuando coincide. Por eso el fallo va
                // a una etiqueta local y no directamente a `fail_label`.
                let next_label = self.new_label();
                let temp = format!("__mt_en_{}", self.temp_counter);
                self.temp_counter += 1;
                self.gen_expr(expr);
                self.emit(Instr::Store(temp.clone()));

                // ¿La variante coincide? __enum_variante(v) == "Variante"
                self.emit(Instr::Load(temp.clone()));
                self.emit(Instr::Call("__enum_variante".to_string(), 1));
                self.emit(Instr::ConstStr(variant.clone()));
                self.emit(Instr::Binary(Op::Equal));
                self.emit(Instr::JmpIf(next_label));

                // Liga cada dato posicional. Un identificador captura; `_` ignora;
                // cualquier otra expresión se compara por igualdad (patrón literal).
                for (i, arg) in args.iter().enumerate() {
                    match arg {
                        Expr::Ident { name, .. } if name == "_" => {}
                        Expr::Ident { name, .. } => {
                            self.emit(Instr::Load(temp.clone()));
                            self.emit(Instr::ConstInt(i as i64));
                            self.emit(Instr::Call("__enum_campo".to_string(), 2));
                            self.emit(Instr::Store(name.clone()));
                        }
                        other => {
                            self.emit(Instr::Load(temp.clone()));
                            self.emit(Instr::ConstInt(i as i64));
                            self.emit(Instr::Call("__enum_campo".to_string(), 2));
                            self.gen_expr(other);
                            self.emit(Instr::Binary(Op::Equal));
                            self.emit(Instr::JmpIf(next_label));
                        }
                    }
                }
                self.emit(Instr::Jmp(body_label));
                self.emit(Instr::Label(next_label));
            }
            Expr::StructInit { fields, .. } => {
                let temp = format!("__mt_st_{}", self.temp_counter);
                self.temp_counter += 1;
                self.gen_expr(expr);
                self.emit(Instr::Store(temp.clone()));
                for (field_name, field_val) in fields {
                    match field_val {
                        Expr::Ident { name, .. } if name != "_" => {
                            self.emit(Instr::Load(temp.clone()));
                            self.emit(Instr::ConstStr(field_name.clone()));
                            self.emit(Instr::StructGet);
                            self.emit(Instr::Store(name.clone()));
                        }
                        Expr::Ident { name, .. } if name == "_" => {}
                        _ => {
                            self.emit(Instr::Load(temp.clone()));
                            self.emit(Instr::ConstStr(field_name.clone()));
                            self.emit(Instr::StructGet);
                            self.gen_expr(field_val);
                            self.emit(Instr::Binary(Op::Equal));
                            self.emit(Instr::JmpIf(fail_label));
                        }
                    }
                }
                self.emit(Instr::Jmp(body_label));
            }
            // BUG-031: `caso _:` es un comodín, no una variable. Antes caía en
            // la rama genérica de abajo, que emitía `Load("_")` y reventaba en
            // tiempo de ejecución con `Variable '_' no definida` — pese a que
            // `lumen check` daba el programa por válido. Coincide siempre, así
            // que saltamos directamente al cuerpo.
            Expr::Ident { name, .. } if name == "_" => {
                self.emit(Instr::Jmp(body_label));
            }
            _ => {
                self.gen_expr(expr);
                self.gen_expr(pattern);
                self.emit(Instr::Binary(Op::NotEqual));
                self.emit(Instr::JmpIf(body_label));
            }
        }
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
            // BUG-109: `i64::MIN / -1` y `i64::MIN % -1` desbordan; plegarlos
            // con `/` y `%` a secas hacía pánico al COMPILAR. Se usa la misma
            // semántica envolvente que Add/Sub/Mul (que ya usaban
            // `overflowing_*`) y que ahora aplica también la VM.
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Div)) => {
                if *b != 0 {
                    Some(Instr::ConstInt(a.overflowing_div(*b).0))
                } else {
                    None
                }
            }
            (Instr::ConstInt(a), Instr::ConstInt(b), Instr::Binary(Op::Mod)) => {
                if *b != 0 {
                    Some(Instr::ConstInt(a.overflowing_rem(*b).0))
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

        // Run neuro-symbolic superoptimization passes (strength reduction, identities, FMA)
        for _ in 0..3 {
            let new = Self::neuro_symbolic_pass(&current);
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

    /// BUG-106: esta pasada aplicaba «reducción de fuerza» convirtiendo
    /// `x * 2`, `x * 4` y `x * 8` en desplazamientos de bits (`x << 1|2|3`).
    ///
    /// La transformación **sólo es válida para enteros**, pero el IR en este
    /// punto no lleva información de tipos: el patrón se reconocía mirando
    /// únicamente la constante literal, así que también se aplicaba cuando el
    /// operando izquierdo era un decimal. El resultado era incoherente entre
    /// backends para una expresión tan corriente como `precio * 2`:
    ///
    /// - En la VM, `ShiftLeft` rechaza los floats ⇒ el programa moría con
    ///   «ShiftLeft requires integers», un operador que no aparecía en el
    ///   código fuente.
    /// - En el binario nativo, el shift truncaba el float a entero
    ///   (`lumen_rt.h`) ⇒ `2.5 * 2` imprimía **4** en vez de 5, en silencio.
    ///
    /// Recuperar la optimización con seguridad exige propagar tipos hasta el
    /// IR y aplicarla sólo cuando ambos operandos son enteros. Multiplicar por
    /// una potencia de dos no es un cuello de botella que justifique un
    /// resultado erróneo, así que la pasada se deja como identidad: se conserva
    /// la función (y su punto de llamada) para no alterar la estructura del
    /// pipeline y para que el día que haya tipos en el IR el sitio esté claro.
    pub fn neuro_symbolic_pass(instrs: &[Instr]) -> Vec<Instr> {
        instrs.to_vec()
    }
}

/// Nombres que un cuerpo de lambda **asigna** en su propio ámbito (BUG-021).
///
/// Se usa para excluirlos de la lista de capturas: una lambda anidada se
/// declara con una asignación (`interna = funcion() {...};`) y por tanto es una
/// variable local, no una captura del entorno. Sólo recorre las sentencias del
/// nivel de la lambda y sus bloques de control; no desciende a cuerpos de otras
/// lambdas, cuyos locales pertenecen a su propio ámbito.
fn collect_assigned_names(body: &[DeclOrStmt], out: &mut Vec<String>) {
    fn walk_stmt(stmt: &Stmt, out: &mut Vec<String>) {
        match stmt {
            Stmt::Assignment { name, .. } => {
                if !out.contains(name) {
                    out.push(name.clone());
                }
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_assigned_names(then_body, out);
                if let Some(eb) = else_body {
                    collect_assigned_names(eb, out);
                }
            }
            Stmt::While { body, .. } => collect_assigned_names(body, out),
            Stmt::For { body, .. } => collect_assigned_names(body, out),
            Stmt::ForEach { body, .. } => collect_assigned_names(body, out),
            Stmt::Block { stmts, .. } => collect_assigned_names(stmts, out),
            Stmt::Match { arms, default, .. } => {
                for arm in arms {
                    collect_assigned_names(&arm.body, out);
                }
                if let Some(d) = default {
                    collect_assigned_names(d, out);
                }
            }
            Stmt::TryCatch {
                try_body,
                catch_body,
                ..
            } => {
                collect_assigned_names(try_body, out);
                collect_assigned_names(catch_body, out);
            }
            _ => {}
        }
    }

    for node in body {
        match node {
            DeclOrStmt::Stmt(stmt) => walk_stmt(stmt, out),
            DeclOrStmt::Decl(Decl::Variable { name, .. }) => {
                if !out.contains(name) {
                    out.push(name.clone());
                }
            }
            DeclOrStmt::Decl(_) => {}
        }
    }
}

/// BUG-052: nombres que el cuerpo DECLARA de verdad (`entero x = ...`,
/// `sea x = ...`), a diferencia de `collect_assigned_names`, que también cuenta
/// las simples asignaciones (`x = x + 1`). Para decidir si una lambda captura
/// una variable hay que distinguirlas: `n = n + 1` sobre una `n` del entorno es
/// una MUTACIÓN de la captura, no la declaración de una local. Confundirlas
/// hacía que el contador clásico (`sea inc = funcion() { n = n + 1; ... }`)
/// perdiera la captura y muriese con "Variable 'n' no definida".
fn collect_declared_names(body: &[DeclOrStmt], out: &mut Vec<String>) {
    fn walk_stmt(stmt: &Stmt, out: &mut Vec<String>) {
        match stmt {
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_declared_names(then_body, out);
                if let Some(eb) = else_body {
                    collect_declared_names(eb, out);
                }
            }
            Stmt::While { body, .. } => collect_declared_names(body, out),
            Stmt::For { body, .. } => collect_declared_names(body, out),
            Stmt::ForEach { body, .. } => collect_declared_names(body, out),
            Stmt::Block { stmts, .. } => collect_declared_names(stmts, out),
            Stmt::Match { arms, default, .. } => {
                for arm in arms {
                    collect_declared_names(&arm.body, out);
                }
                if let Some(d) = default {
                    collect_declared_names(d, out);
                }
            }
            Stmt::TryCatch {
                try_body,
                catch_body,
                ..
            } => {
                collect_declared_names(try_body, out);
                collect_declared_names(catch_body, out);
            }
            _ => {}
        }
    }

    for node in body {
        match node {
            DeclOrStmt::Stmt(stmt) => walk_stmt(stmt, out),
            DeclOrStmt::Decl(Decl::Variable { name, .. }) => {
                if !out.contains(name) {
                    out.push(name.clone());
                }
            }
            DeclOrStmt::Decl(_) => {}
        }
    }
}

/// BUG-029: nombres de builtins públicos (sin prefijo `__`) que pueden
/// aparecer como destino de una llamada. Al recolectar las capturas de una
/// lambda no deben confundirse con variables del entorno: `imprimir(...)`
/// dentro de una lambda no captura nada.
fn is_public_builtin_name(name: &str) -> bool {
    matches!(
        name,
        "imprimir"
            | "print"
            | "leer"
            | "read"
            | "a_texto"
            | "to_texto"
            | "a_entero"
            | "to_int"
            | "to_entero"
            | "a_decimal"
            | "to_float"
            | "a_numero"
            | "to_number"
            | "a_entero_seguro"
            | "to_int_safe"
            | "a_decimal_seguro"
            | "to_float_safe"
            | "es_numero"
            | "is_number"
            | "abs"
            | "absoluto"
            | "minimo"
            | "min"
            | "maximo"
            | "max"
            | "raiz"
            | "sqrt"
            | "potencia"
            | "pow"
            | "piso"
            | "floor"
            | "techo"
            | "ceil"
            | "redondear"
            | "round"
            | "largo"
            | "len"
            | "agregar"
            | "push"
    )
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
        Type::Prestado { inner, .. } => type_to_impl_name(inner),
        Type::Dueno(inner) => type_to_impl_name(inner),
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
            captures: vec![],
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
