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
    emitted: HashSet<PathBuf>,
    known_prefixes: HashSet<String>,
}

impl ModuleLoader {
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths,
            visited: HashSet::new(),
            emitted: HashSet::new(),
            known_prefixes: HashSet::new(),
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
        self.emitted.clear();
        self.known_prefixes.clear();
        let program = parse_source(source, base_path)?;
        self.flatten(program, base_path)
    }

    fn flatten(&mut self, program: Program, current_path: &Path) -> Result<Program, ModuleError> {
        // Canonicalizar para comparar rutas de forma robusta (Windows: fs::canonicalize
        // añade el prefijo \\?\ — comparar crudo vs canonical nunca da igualdad).
        let current_norm = fs::canonicalize(current_path)
            .unwrap_or_else(|_| current_path.to_path_buf());
        let mut result = Vec::new();
        for node in program {
            match node {
                DeclOrStmt::Stmt(Stmt::Import { path, alias, span }) => {
                    if path == "ingles" || path == "english" {
                        continue;
                    }
                    let current_dir = if current_norm.is_dir() {
                        current_norm.clone()
                    } else {
                        current_norm
                            .parent()
                            .unwrap_or(Path::new("."))
                            .to_path_buf()
                    };
                    let resolved = self.resolve_path(&path, &current_dir, &current_norm)?;
                    if resolved == current_norm {
                        // Self-import: el archivo se importa a sí mismo por nombre
                        // (p. ej. `examples/graficos_avanzado.nv` importando
                        // "graficos_avanzado.nv"). No-op para romper el ciclo.
                        continue;
                    }
                    if !self.emitted.insert(resolved.clone()) {
                        // Ya aplanado antes (import directo + transitivo):
                        // p. ej. `tui_core.nv` vía `tui.nv` y también directo.
                        // La copia ya insertada lleva su prefijo correcto.
                        continue;
                    }
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
                    let flat = self.flatten(imported_program, &resolved)?;
                    self.visited.remove(&resolved);
                    let prefix = alias.unwrap_or_else(|| {
                        resolved
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("module")
                            .to_string()
                    });
                    self.known_prefixes.insert(prefix.clone());
                    let mut prefixed = flat;
                    prefix_program(&mut prefixed, &prefix, &self.known_prefixes);
                    result.extend(prefixed);
                }
                other => result.push(other),
            }
        }
        Ok(result)
    }

    fn resolve_path(&self, path: &str, current_dir: &Path, current_path: &Path) -> Result<PathBuf, ModuleError> {
        let extensions = [".nv", ".lumen"];
        // Skip de auto-importación: si la ruta as-is cae sobre el archivo que se
        // está aplanando (p. ej. `examples/graficos_avanzado.nv` importando
        // "graficos_avanzado.nv"), continuar hacia los search_paths (stdlib).
        let is_self = |p: &Path| -> bool {
            fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()) == current_path
        };
        if path.contains('.') || path.contains('/') || path.contains('\\') {
            // Try as-is (full path with extension)
            let p = current_dir.join(path);
            if p.exists() && !is_self(&p) {
                return Ok(fs::canonicalize(&p).unwrap_or(p));
            }
            for sp in &self.search_paths {
                let p = sp.join(path);
                if p.exists() {
                    return Ok(fs::canonicalize(&p).unwrap_or(p));
                }
            }
            // Try with extensions
            for ext in &extensions {
                let p = current_dir.join(format!("{}{}", path, ext));
                if p.exists() && !is_self(&p) {
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
                path: current_dir.join(path),
                message: format!("Archivo no encontrado: '{}'", path),
            })
        } else {
            for ext in &extensions {
                let p = current_dir.join(format!("{}{}", path, ext));
                if p.exists() && !is_self(&p) {
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

fn prefix_program(program: &mut Program, prefix: &str, known: &HashSet<String>) {
    let mut locals = HashSet::new();
    for node in program.iter_mut() {
        prefix_node(node, prefix, &mut locals, true, known);
    }
}
fn prefix_node(
    node: &mut DeclOrStmt,
    prefix: &str,
    locals: &mut HashSet<String>,
    top_level: bool,
    known: &HashSet<String>,
) {
    match node {
        DeclOrStmt::Decl(d) => prefix_decl(d, prefix, locals, top_level, known),
        DeclOrStmt::Stmt(s) => prefix_stmt(s, prefix, locals, top_level, known),
    }
}

fn prefix_decl(
    decl: &mut Decl,
    prefix: &str,
    locals: &mut HashSet<String>,
    top_level: bool,
    known: &HashSet<String>,
) {
    match decl {
        Decl::Variable {
            var_type,
            name,
            init,
            ..
        } => {
            prefix_type(var_type, prefix, known);
            if top_level {
                if !is_known_prefixed(name, known) {
                    *name = format!("{}_{}", prefix, name);
                }
            } else {
                locals.insert(name.clone());
            }
            if let Some(expr) = init {
                prefix_expr(expr, prefix, locals, known);
            }
        }
        Decl::Const {
            var_type,
            name,
            value,
            ..
        } => {
            prefix_type(var_type, prefix, known);
            if top_level {
                if !is_known_prefixed(name, known) {
                    *name = format!("{}_{}", prefix, name);
                }
            } else {
                locals.insert(name.clone());
            }
            prefix_expr(value, prefix, locals, known);
        }
        Decl::Destructure { targets, init, .. } => {
            for target in targets.iter_mut() {
                if let Some(ref mut t_type) = target.var_type {
                    prefix_type(t_type, prefix, known);
                }
                if target.name != "_" {
                    if top_level {
                        if !is_known_prefixed(&target.name, known) {
                            target.name = format!("{}_{}", prefix, target.name);
                        }
                    } else {
                        locals.insert(target.name.clone());
                    }
                }
            }
            prefix_expr(init, prefix, locals, known);
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
            prefix_type_with_params(return_type, prefix, &type_params_set, known);
            if top_level {
                if !is_known_prefixed(name, known) {
                    *name = format!("{}_{}", prefix, name);
                }
            }
            for p in params.iter_mut() {
                prefix_type_with_params(&mut p.param_type, prefix, &type_params_set, known);
                if let Some(default) = &mut p.default {
                    prefix_expr(default, prefix, locals, known);
                }
            }
            let mut func_locals = locals.clone();
            for p in params.iter() {
                func_locals.insert(p.name.clone());
            }
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut func_locals, false, known);
            }
        }
        Decl::Struct {
            name,
            fields,
            type_params,
            ..
        } => {
            if top_level {
                if !is_known_prefixed(name, known) {
                    *name = format!("{}_{}", prefix, name);
                }
            }
            let type_params_set: HashSet<String> = type_params.iter().cloned().collect();
            for field in fields.iter_mut() {
                prefix_type_with_params(&mut field.field_type, prefix, &type_params_set, known);
            }
        }
        Decl::Enum { name, variants, .. } => {
            if top_level {
                if !is_known_prefixed(name, known) {
                    *name = format!("{}_{}", prefix, name);
                }
            }
            for variant in variants.iter_mut() {
                for t in variant.types.iter_mut() {
                    prefix_type(t, prefix, known);
                }
            }
        }
        Decl::Rasgo { name, methods, .. } => {
            if top_level {
                if !is_known_prefixed(name, known) {
                    *name = format!("{}_{}", prefix, name);
                }
            }
            for method in methods.iter_mut() {
                prefix_type(&mut method.return_type, prefix, known);
                for p in method.params.iter_mut() {
                    prefix_type(&mut p.param_type, prefix, known);
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
            prefix_type(target_type, prefix, known);
            for assoc in associated_types.iter_mut() {
                prefix_type(&mut assoc.target_type, prefix, known);
            }
            for method_decl in methods.iter_mut() {
                prefix_decl(method_decl, prefix, locals, top_level, known);
            }
        }
    }
}

fn prefix_stmt(
    stmt: &mut Stmt,
    prefix: &str,
    locals: &mut HashSet<String>,
    _top_level: bool,
    known: &HashSet<String>,
) {
    match stmt {
        Stmt::Assignment { name, value, .. } => {
            if !locals.contains(name.as_str()) && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            prefix_expr(value, prefix, locals, known);
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            prefix_expr(condition, prefix, locals, known);
            let mut if_locals = locals.clone();
            for node in then_body.iter_mut() {
                prefix_node(node, prefix, &mut if_locals, false, known);
            }
            if let Some(body) = else_body {
                let mut else_locals = locals.clone();
                for node in body.iter_mut() {
                    prefix_node(node, prefix, &mut else_locals, false, known);
                }
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            prefix_expr(condition, prefix, locals, known);
            let mut while_locals = locals.clone();
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut while_locals, false, known);
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
            prefix_expr(condition, prefix, &for_locals, known);
            prefix_stmt(update, prefix, &mut for_locals, false, known);
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut for_locals, false, known);
            }
        }
        Stmt::Return { value, .. } => {
            if let Some(expr) = value {
                prefix_expr(expr, prefix, locals, known);
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
            prefix_expr(expr, prefix, &foreach_locals, known);
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut foreach_locals, false, known);
            }
        }
        Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Import { .. } => {}
        Stmt::GuardLet {
            value, else_body, ..
        } => {
            prefix_expr(value, prefix, locals, known);
            let mut guard_locals = locals.clone();
            for node in else_body.iter_mut() {
                prefix_node(node, prefix, &mut guard_locals, false, known);
            }
        }
        Stmt::Match {
            expr,
            arms,
            default,
            ..
        } => {
            prefix_expr(expr, prefix, locals, known);
            for arm in arms.iter_mut() {
                prefix_expr(&mut arm.value, prefix, locals, known);
                if let Some(ref mut guard) = arm.guard {
                    prefix_expr(guard, prefix, locals, known);
                }
                let mut arm_locals = locals.clone();
                for node in arm.body.iter_mut() {
                    prefix_node(node, prefix, &mut arm_locals, false, known);
                }
            }
            if let Some(body) = default {
                let mut def_locals = locals.clone();
                for node in body.iter_mut() {
                    prefix_node(node, prefix, &mut def_locals, false, known);
                }
            }
        }
        Stmt::Expr { expr, .. } => {
            prefix_expr(expr, prefix, locals, known);
        }
        Stmt::FieldAssign { expr, value, .. } => {
            prefix_expr(expr, prefix, locals, known);
            prefix_expr(value, prefix, locals, known);
        }
        Stmt::ArraySet { arr, index, value, .. } => {
            prefix_expr(arr, prefix, locals, known);
            prefix_expr(index, prefix, locals, known);
            prefix_expr(value, prefix, locals, known);
        }
        Stmt::Block { stmts, .. } => {
            let mut block_locals = locals.clone();
            for node in stmts.iter_mut() {
                prefix_node(node, prefix, &mut block_locals, false, known);
            }
        }
        Stmt::Destructure { targets, value, .. } => {
            for target in targets.iter_mut() {
                if target.name != "_" && !locals.contains(target.name.as_str()) {
                    target.name = format!("{}_{}", prefix, target.name);
                }
            }
            prefix_expr(value, prefix, locals, known);
        }
        Stmt::IfLet {
            value,
            then_body,
            else_body,
            ..
        } => {
            prefix_expr(value, prefix, locals, known);
            for node in then_body.iter_mut() {
                prefix_node(node, prefix, locals, false, known);
            }
            if let Some(eb) = else_body {
                for node in eb.iter_mut() {
                    prefix_node(node, prefix, locals, false, known);
                }
            }
        }
    }
}

fn prefix_expr(expr: &mut Expr, prefix: &str, locals: &HashSet<String>, known: &HashSet<String>) {
    match expr {
        Expr::Int { .. } | Expr::Float { .. } | Expr::Str { .. } | Expr::Bool { .. } => {}
        Expr::Ident { name, .. } => {
            if !locals.contains(name.as_str()) && !is_builtin(name) && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
        }
        Expr::Binary { left, right, .. } => {
            prefix_expr(left, prefix, locals, known);
            prefix_expr(right, prefix, locals, known);
        }
        Expr::Unary { operand, .. } => {
            prefix_expr(operand, prefix, locals, known);
        }
        Expr::Call {
            callee,
            args,
            type_args,
            ..
        } => {
            prefix_expr(callee, prefix, locals, known);
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals, known);
            }
            for ta in type_args.iter_mut() {
                prefix_type(ta, prefix, known);
            }
        }
        Expr::Grouping { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known);
        }
        Expr::Cast {
            expr: inner,
            cast_type,
            ..
        } => {
            prefix_expr(inner, prefix, locals, known);
            if let Type::Struct(name) = cast_type {
                if !name.is_empty() && name != "Infer" && !is_known_prefixed(name, known) {
                    let prefixed = format!("{}{}", prefix, name);
                    *name = prefixed;
                }
            } else if let Type::Lista(inner_t) = cast_type {
                if let Type::Struct(name) = inner_t.as_mut() {
                    if !name.is_empty() && name != "Infer" && !is_known_prefixed(name, known) {
                        let prefixed = format!("{}{}", prefix, name);
                        *name = prefixed;
                    }
                }
            }
        }
        Expr::List { items, .. } => {
            for item in items.iter_mut() {
                prefix_expr(item, prefix, locals, known);
            }
        }
        Expr::Index {
            expr: target,
            index,
            ..
        } => {
            prefix_expr(target, prefix, locals, known);
            prefix_expr(index, prefix, locals, known);
        }
        Expr::MethodCall {
            expr: target, args, ..
        } => {
            prefix_expr(target, prefix, locals, known);
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals, known);
            }
        }
        Expr::Lambda { params, body, .. } => {
            let mut lambda_locals = locals.clone();
            for p in params.iter_mut() {
                prefix_type(&mut p.param_type, prefix, known);
                lambda_locals.insert(p.name.clone());
            }
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut lambda_locals, false, known);
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
                prefix_expr(value, prefix, locals, known);
            }
            for ta in type_args.iter_mut() {
                prefix_type(ta, prefix, known);
            }
        }
        Expr::FieldAccess { expr: target, .. } => {
            prefix_expr(target, prefix, locals, known);
        }
        Expr::Exito { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known);
        }
        Expr::Error { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known);
        }
        Expr::Intentar { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known);
        }
        Expr::Algun { expr: inner, .. } => {
            prefix_expr(inner, prefix, locals, known);
        }
        Expr::Ninguno { .. } => {}
        Expr::Tuple { items, .. } => {
            for item in items.iter_mut() {
                prefix_expr(item, prefix, locals, known);
            }
        }
        Expr::TupleAccess { expr: target, .. } => {
            prefix_expr(target, prefix, locals, known);
        }
        Expr::EnumCtor {
            enum_name, args, ..
        } => {
            if !locals.contains(enum_name.as_str()) && !is_builtin(enum_name) {
                *enum_name = format!("{}_{}", prefix, enum_name);
            }
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals, known);
            }
        }
        Expr::Ternary {
            condition,
            true_branch,
            false_branch,
            ..
        } => {
            prefix_expr(condition, prefix, locals, known);
            prefix_expr(true_branch, prefix, locals, known);
            prefix_expr(false_branch, prefix, locals, known);
        }
        Expr::Esperar { expr, .. } => {
            prefix_expr(expr, prefix, locals, known);
        }
    }
}

fn prefix_type_with_params(
    t: &mut Type,
    prefix: &str,
    type_params: &HashSet<String>,
    known: &HashSet<String>,
) {
    match t {
        Type::Struct(name) if type_params.contains(name.as_str()) => {
            // Don't prefix type parameter names
        }
        Type::GenericStruct { name, args } => {
            if !type_params.contains(name.as_str()) && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            for arg in args.iter_mut() {
                prefix_type_with_params(arg, prefix, type_params, known);
            }
        }
        Type::Lista(inner) => prefix_type_with_params(inner, prefix, type_params, known),
        Type::Func {
            param_types,
            return_type,
        } => {
            for p in param_types.iter_mut() {
                prefix_type_with_params(p, prefix, type_params, known);
            }
            prefix_type_with_params(return_type, prefix, type_params, known);
        }
        Type::Struct(name) if name != "Infer" && !is_known_prefixed(name, known) => {
            *name = format!("{}_{}", prefix, name);
        }
        Type::Resultado { ok, err } => {
            prefix_type_with_params(ok, prefix, type_params, known);
            prefix_type_with_params(err, prefix, type_params, known);
        }
        Type::Opcion(inner) => {
            prefix_type_with_params(inner, prefix, type_params, known);
        }
        Type::Tuple(types) => {
            for t in types.iter_mut() {
                prefix_type_with_params(t, prefix, type_params, known);
            }
        }
        _ => {}
    }
}

fn prefix_type(t: &mut Type, prefix: &str, known: &HashSet<String>) {
    match t {
        Type::Lista(inner) => prefix_type(inner, prefix, known),
        Type::GenericStruct { name, args } => {
            if !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
            for arg in args.iter_mut() {
                prefix_type(arg, prefix, known);
            }
        }
        Type::Func {
            param_types,
            return_type,
        } => {
            for p in param_types.iter_mut() {
                prefix_type(p, prefix, known);
            }
            prefix_type(return_type, prefix, known);
        }
        Type::Struct(name) => {
            if name != "Infer" && !is_known_prefixed(name, known) {
                *name = format!("{}_{}", prefix, name);
            }
        }
        Type::Resultado { ok, err } => {
            prefix_type(ok, prefix, known);
            prefix_type(err, prefix, known);
        }
        Type::Opcion(inner) => {
            prefix_type(inner, prefix, known);
        }
        Type::Tuple(types) => {
            for t in types.iter_mut() {
                prefix_type(t, prefix, known);
            }
        }
        _ => {}
    }
}

fn is_known_prefixed(name: &str, known: &HashSet<String>) -> bool {
    known
        .iter()
        .any(|p| !p.is_empty() && name.starts_with(&format!("{}_", p)))
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "imprimir"
            | "print"
            | "largo"
            | "len"
            | "agregar"
            | "push"
            | "leer"
            | "read"
            | "a_texto"
            | "to_texto"
            | "__str_from"
            | "__str_len"
            | "__str_longitud"
            | "__str_ord"
            | "__str_codigo"
            | "__str_chr"
            | "__str_caracter"
            | "__str_slice"
            | "__str_subcadena"
            | "__str_concat_list"
            | "__str_concatenar_lista"
            | "__str_starts_with"
            | "__str_empieza_con"
            | "__str_to_chars"
            | "__str_a_caracteres"
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
             | "__file_append"
             | "__agregar_archivo"
            | "__file_write_binary"
            | "__escribir_archivo_bin"
            | "__num_a_f64_bytes"
            | "__numero_a_bytes_f64"
            | "__file_bytes"
            | "__leer_bytes"
            | "__a_f64_bytes"
            | "__bytes_a_f64"
            | "__codegen_a_nvc"
            | "__compile_nv"
            | "__compilar_nv"
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
            | "__str_replace"
            | "__str_reemplazar"
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
}
