use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use lumen_lexer::token::Span;
use lumen_lexer::Lexer;
use lumen_parser::ast::*;
use lumen_parser::Parser;

#[derive(Debug)]
pub enum ModuleError {
    Io { path: PathBuf, message: String },
    Lex { path: PathBuf, details: Vec<String> },
    Parse { path: PathBuf, details: Vec<String> },
    Circular { path: PathBuf, span: Span },
}

pub struct ModuleLoader {
    search_paths: Vec<PathBuf>,
    visited: HashSet<PathBuf>,
}

impl ModuleLoader {
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths,
            visited: HashSet::new(),
        }
    }

    pub fn with_default_search_paths() -> Self {
        Self::new(Vec::new())
    }

    pub fn resolve_imports(
        &mut self,
        source: &str,
        base_path: &Path,
    ) -> Result<Program, ModuleError> {
        self.visited.clear();
        let program = parse_source(source, base_path)?;
        self.flatten(program, base_path)
    }

    fn flatten(&mut self, program: Program, current_dir: &Path) -> Result<Program, ModuleError> {
        let mut result = Vec::new();
        for node in program {
            match node {
                DeclOrStmt::Stmt(Stmt::Import { path, alias, span }) => {
                    if path == "ingles" || path == "english" {
                        continue;
                    }
                    let resolved = self.resolve_path(&path, current_dir)?;
                    if !self.visited.insert(resolved.clone()) {
                        return Err(ModuleError::Circular {
                            path: resolved,
                            span,
                        });
                    }
                    let source = fs::read_to_string(&resolved).map_err(|e| ModuleError::Io {
                        path: resolved.clone(),
                        message: format!("No se pudo leer '{}': {}", resolved.display(), e),
                    })?;
                    let imported_program = parse_source(&source, &resolved)?;
                    let parent = resolved.parent().unwrap_or(Path::new("."));
                    let flat = self.flatten(imported_program, parent)?;
                    self.visited.remove(&resolved);
                    let prefix = alias.unwrap_or_else(|| {
                        resolved
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("module")
                            .to_string()
                    });
                    let mut prefixed = flat;
                    prefix_program(&mut prefixed, &prefix);
                    result.extend(prefixed);
                }
                other => result.push(other),
            }
        }
        Ok(result)
    }

    fn resolve_path(&self, path: &str, current_dir: &Path) -> Result<PathBuf, ModuleError> {
        if path.contains('.') || path.contains('/') || path.contains('\\') {
            let p = current_dir.join(path);
            if p.exists() {
                return Ok(fs::canonicalize(&p).unwrap_or(p));
            }
            for sp in &self.search_paths {
                let p = sp.join(path);
                if p.exists() {
                    return Ok(fs::canonicalize(&p).unwrap_or(p));
                }
            }
            Err(ModuleError::Io {
                path: current_dir.join(path),
                message: format!("Archivo no encontrado: '{}'", path),
            })
        } else {
            let extensions = [".nv", ".lumen"];
            for ext in &extensions {
                let p = current_dir.join(format!("{}{}", path, ext));
                if p.exists() {
                    return Ok(fs::canonicalize(&p).unwrap_or(p));
                }
            }
            for sp in &self.search_paths {
                for ext in &extensions {
                    let p = sp.join(format!("{}{}", path, ext));
                    if p.exists() {
                        return Ok(fs::canonicalize(&p).unwrap_or(p));
                    }
                }
            }
            Err(ModuleError::Io {
                path: current_dir.join(format!("{}.nv", path)),
                message: format!("Módulo no encontrado: '{}'", path),
            })
        }
    }
}

fn parse_source(source: &str, path: &Path) -> Result<Program, ModuleError> {
    let lexer = Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        return Err(ModuleError::Lex {
            path: path.to_path_buf(),
            details: lex_errors
                .iter()
                .map(|e| {
                    format!(
                        "{} [{}:{}]: {} ({})",
                        e.code, e.pos.line, e.pos.col, e.message, e.suggestion
                    )
                })
                .collect(),
        });
    }
    let parser = Parser::new(tokens);
    let (program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        return Err(ModuleError::Parse {
            path: path.to_path_buf(),
            details: parse_errors
                .iter()
                .map(|e| {
                    format!(
                        "{} [{}:{}]: {} ({})",
                        e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
                    )
                })
                .collect(),
        });
    }
    Ok(program)
}

fn prefix_program(program: &mut Program, prefix: &str) {
    let mut locals = HashSet::new();
    for node in program.iter_mut() {
        prefix_node(node, prefix, &mut locals, true);
    }
}

fn prefix_node(node: &mut DeclOrStmt, prefix: &str, locals: &mut HashSet<String>, top_level: bool) {
    match node {
        DeclOrStmt::Decl(d) => prefix_decl(d, prefix, locals, top_level),
        DeclOrStmt::Stmt(s) => prefix_stmt(s, prefix, locals, top_level),
    }
}

fn prefix_decl(decl: &mut Decl, prefix: &str, locals: &mut HashSet<String>, top_level: bool) {
    match decl {
        Decl::Variable {
            var_type,
            name,
            init,
            ..
        } => {
            prefix_type(var_type, prefix);
            if top_level {
                *name = format!("{}_{}", prefix, name);
            } else {
                locals.insert(name.clone());
            }
            if let Some(expr) = init {
                prefix_expr(expr, prefix, locals);
            }
        }
        Decl::Const {
            var_type,
            name,
            value,
            ..
        } => {
            prefix_type(var_type, prefix);
            if top_level {
                *name = format!("{}_{}", prefix, name);
            } else {
                locals.insert(name.clone());
            }
            prefix_expr(value, prefix, locals);
        }
        Decl::Destructure { targets, init, .. } => {
            for target in targets.iter_mut() {
                if let Some(ref mut t_type) = target.var_type {
                    prefix_type(t_type, prefix);
                }
                if target.name != "_" {
                    if top_level {
                        target.name = format!("{}_{}", prefix, target.name);
                    } else {
                        locals.insert(target.name.clone());
                    }
                }
            }
            prefix_expr(init, prefix, locals);
        }
        Decl::Function {
            return_type,
            name,
            params,
            body,
            type_params,
            ..
        } => {
            let type_params_set: HashSet<String> = type_params.iter().cloned().collect();
            prefix_type_with_params(return_type, prefix, &type_params_set);
            if top_level {
                *name = format!("{}_{}", prefix, name);
            }
            for p in params.iter_mut() {
                prefix_type_with_params(&mut p.param_type, prefix, &type_params_set);
                if let Some(default) = &mut p.default {
                    prefix_expr(default, prefix, locals);
                }
            }
            let mut func_locals = locals.clone();
            for p in params.iter() {
                func_locals.insert(p.name.clone());
            }
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut func_locals, false);
            }
        }
        Decl::Struct {
            name,
            fields,
            type_params,
            ..
        } => {
            if top_level {
                *name = format!("{}_{}", prefix, name);
            }
            let type_params_set: HashSet<String> = type_params.iter().cloned().collect();
            for field in fields.iter_mut() {
                prefix_type_with_params(&mut field.field_type, prefix, &type_params_set);
            }
        }
        Decl::Enum { name, variants, .. } => {
            if top_level {
                *name = format!("{}_{}", prefix, name);
            }
            for variant in variants.iter_mut() {
                for t in variant.types.iter_mut() {
                    prefix_type(t, prefix);
                }
            }
        }
        Decl::Rasgo { name, methods, .. } => {
            if top_level {
                *name = format!("{}_{}", prefix, name);
            }
            for method in methods.iter_mut() {
                prefix_type(&mut method.return_type, prefix);
                for p in method.params.iter_mut() {
                    prefix_type(&mut p.param_type, prefix);
                }
            }
        }
        Decl::ImplRasgo {
            trait_name: _,
            target_type,
            associated_types,
            methods,
            ..
        } => {
            prefix_type(target_type, prefix);
            for assoc in associated_types.iter_mut() {
                prefix_type(&mut assoc.target_type, prefix);
            }
            for method_decl in methods.iter_mut() {
                prefix_decl(method_decl, prefix, locals, top_level);
            }
        }
    }
}

fn prefix_stmt(stmt: &mut Stmt, prefix: &str, locals: &mut HashSet<String>, _top_level: bool) {
    match stmt {
        Stmt::Assignment { name, value, .. } => {
            if !locals.contains(name.as_str()) {
                *name = format!("{}_{}", prefix, name);
            }
            prefix_expr(value, prefix, locals);
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            prefix_expr(condition, prefix, locals);
            let mut if_locals = locals.clone();
            for node in then_body.iter_mut() {
                prefix_node(node, prefix, &mut if_locals, false);
            }
            if let Some(body) = else_body {
                let mut else_locals = locals.clone();
                for node in body.iter_mut() {
                    prefix_node(node, prefix, &mut else_locals, false);
                }
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            prefix_expr(condition, prefix, locals);
            let mut while_locals = locals.clone();
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut while_locals, false);
            }
        }
        Stmt::For {
            init,
            condition,
            update,
            body,
            ..
        } => {
            let mut for_locals = locals.clone();
            if let Decl::Variable { name, .. } = init.as_mut() {
                for_locals.insert(name.clone());
            }
            prefix_expr(condition, prefix, &for_locals);
            prefix_stmt(update, prefix, &mut for_locals, false);
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut for_locals, false);
            }
        }
        Stmt::Return { value, .. } => {
            if let Some(expr) = value {
                prefix_expr(expr, prefix, locals);
            }
        }
        Stmt::ForEach {
            var_name,
            expr,
            body,
            ..
        } => {
            let mut foreach_locals = locals.clone();
            foreach_locals.insert(var_name.clone());
            prefix_expr(expr, prefix, &foreach_locals);
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut foreach_locals, false);
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Import { .. } => {}
        Stmt::GuardLet {
            value, else_body, ..
        } => {
            prefix_expr(value, prefix, locals);
            let mut guard_locals = locals.clone();
            for node in else_body.iter_mut() {
                prefix_node(node, prefix, &mut guard_locals, false);
            }
        }
        Stmt::Match {
            expr,
            arms,
            default,
            ..
        } => {
            prefix_expr(expr, prefix, locals);
            for arm in arms.iter_mut() {
                prefix_expr(&mut arm.value, prefix, locals);
                if let Some(ref mut guard) = arm.guard {
                    prefix_expr(guard, prefix, locals);
                }
                let mut arm_locals = locals.clone();
                for node in arm.body.iter_mut() {
                    prefix_node(node, prefix, &mut arm_locals, false);
                }
            }
            if let Some(body) = default {
                let mut def_locals = locals.clone();
                for node in body.iter_mut() {
                    prefix_node(node, prefix, &mut def_locals, false);
                }
            }
        }
        Stmt::Expr { expr, .. } => {
            prefix_expr(expr, prefix, locals);
        }
        Stmt::FieldAssign { expr, value, .. } => {
            prefix_expr(expr, prefix, locals);
            prefix_expr(value, prefix, locals);
        }
        Stmt::Block { stmts, .. } => {
            let mut block_locals = locals.clone();
            for node in stmts.iter_mut() {
                prefix_node(node, prefix, &mut block_locals, false);
            }
        }
        Stmt::Destructure { targets, value, .. } => {
            for target in targets.iter_mut() {
                if target.name != "_" && !locals.contains(target.name.as_str()) {
                    target.name = format!("{}_{}", prefix, target.name);
                }
            }
            prefix_expr(value, prefix, locals);
        }
        Stmt::IfLet {
            value,
            then_body,
            else_body,
            ..
        } => {
            prefix_expr(value, prefix, locals);
            for node in then_body.iter_mut() {
                prefix_node(node, prefix, locals, false);
            }
            if let Some(eb) = else_body {
                for node in eb.iter_mut() {
                    prefix_node(node, prefix, locals, false);
                }
            }
        }
    }
}

fn prefix_expr(expr: &mut Expr, prefix: &str, locals: &HashSet<String>) {
    match expr {
        Expr::Int { .. } | Expr::Float { .. } | Expr::Str { .. } | Expr::Bool { .. } => {}
        Expr::Ident { name, .. } => {
            if !locals.contains(name.as_str()) && !is_builtin(name) {
                *name = format!("{}_{}", prefix, name);
            }
        }
        Expr::Binary { left, right, .. } => {
            prefix_expr(left, prefix, locals);
            prefix_expr(right, prefix, locals);
        }
        Expr::Unary { operand, .. } => {
            prefix_expr(operand, prefix, locals);
        }
        Expr::Call {
            callee,
            args,
            type_args,
            ..
        } => {
            prefix_expr(callee, prefix, locals);
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals);
            }
            for ta in type_args.iter_mut() {
                prefix_type(ta, prefix);
            }
        }
        Expr::Grouping { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals);
        }
        Expr::List { items, .. } => {
            for item in items.iter_mut() {
                prefix_expr(item, prefix, locals);
            }
        }
        Expr::Index {
            expr: target,
            index,
            ..
        } => {
            prefix_expr(target, prefix, locals);
            prefix_expr(index, prefix, locals);
        }
        Expr::MethodCall {
            expr: target, args, ..
        } => {
            prefix_expr(target, prefix, locals);
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals);
            }
        }
        Expr::Lambda { params, body, .. } => {
            let mut lambda_locals = locals.clone();
            for p in params.iter_mut() {
                prefix_type(&mut p.param_type, prefix);
                lambda_locals.insert(p.name.clone());
            }
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut lambda_locals, false);
            }
        }
        Expr::StructInit {
            struct_name,
            fields,
            type_args,
            ..
        } => {
            *struct_name = format!("{}_{}", prefix, struct_name);
            for (_, value) in fields.iter_mut() {
                prefix_expr(value, prefix, locals);
            }
            for ta in type_args.iter_mut() {
                prefix_type(ta, prefix);
            }
        }
        Expr::FieldAccess { expr: target, .. } => {
            prefix_expr(target, prefix, locals);
        }
        Expr::Exito { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals);
        }
        Expr::Error { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals);
        }
        Expr::Intentar { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals);
        }
        Expr::Algun { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals);
        }
        Expr::Ninguno { .. } => {}
        Expr::Tuple { items, .. } => {
            for item in items.iter_mut() {
                prefix_expr(item, prefix, locals);
            }
        }
        Expr::TupleAccess { expr: target, .. } => {
            prefix_expr(target, prefix, locals);
        }
        Expr::EnumCtor {
            enum_name, args, ..
        } => {
            if !locals.contains(enum_name.as_str()) && !is_builtin(enum_name) {
                *enum_name = format!("{}_{}", prefix, enum_name);
            }
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals);
            }
        }
        Expr::Ternary {
            condition,
            true_branch,
            false_branch,
            ..
        } => {
            prefix_expr(condition, prefix, locals);
            prefix_expr(true_branch, prefix, locals);
            prefix_expr(false_branch, prefix, locals);
        }
        Expr::Esperar { expr, .. } => {
            prefix_expr(expr, prefix, locals);
        }
    }
}

fn prefix_type_with_params(t: &mut Type, prefix: &str, type_params: &HashSet<String>) {
    match t {
        Type::Struct(name) if type_params.contains(name.as_str()) => {
            // Don't prefix type parameter names
        }
        Type::GenericStruct { name, args } => {
            if !type_params.contains(name.as_str()) {
                *name = format!("{}_{}", prefix, name);
            }
            for arg in args.iter_mut() {
                prefix_type_with_params(arg, prefix, type_params);
            }
        }
        Type::Lista(inner) => prefix_type_with_params(inner, prefix, type_params),
        Type::Func {
            param_types,
            return_type,
        } => {
            for p in param_types.iter_mut() {
                prefix_type_with_params(p, prefix, type_params);
            }
            prefix_type_with_params(return_type, prefix, type_params);
        }
        Type::Struct(name) => {
            *name = format!("{}_{}", prefix, name);
        }
        Type::Resultado { ok, err } => {
            prefix_type_with_params(ok, prefix, type_params);
            prefix_type_with_params(err, prefix, type_params);
        }
        Type::Opcion(inner) => {
            prefix_type_with_params(inner, prefix, type_params);
        }
        Type::Tuple(types) => {
            for t in types.iter_mut() {
                prefix_type_with_params(t, prefix, type_params);
            }
        }
        _ => {}
    }
}

fn prefix_type(t: &mut Type, prefix: &str) {
    match t {
        Type::Lista(inner) => prefix_type(inner, prefix),
        Type::GenericStruct { name, args } => {
            *name = format!("{}_{}", prefix, name);
            for arg in args.iter_mut() {
                prefix_type(arg, prefix);
            }
        }
        Type::Func {
            param_types,
            return_type,
        } => {
            for p in param_types.iter_mut() {
                prefix_type(p, prefix);
            }
            prefix_type(return_type, prefix);
        }
        Type::Struct(name) => {
            *name = format!("{}_{}", prefix, name);
        }
        Type::Resultado { ok, err } => {
            prefix_type(ok, prefix);
            prefix_type(err, prefix);
        }
        Type::Opcion(inner) => {
            prefix_type(inner, prefix);
        }
        Type::Tuple(types) => {
            for t in types.iter_mut() {
                prefix_type(t, prefix);
            }
        }
        _ => {}
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "imprimir"
            | "print"
            | "leer"
            | "read"
            | "a_texto"
            | "to_texto"
            | "__str_from"
            | "__str_len"
            | "__str_longitud"
            | "__str_ord"
            | "__str_codigo"
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
    )
}
