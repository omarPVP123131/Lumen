use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs;

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
        Self { search_paths, visited: HashSet::new() }
    }

    pub fn with_default_search_paths() -> Self {
        Self::new(Vec::new())
    }

    pub fn resolve_imports(&mut self, source: &str, base_path: &Path) -> Result<Program, ModuleError> {
        self.visited.clear();
        let program = parse_source(source, base_path)?;
        self.flatten(program, base_path)
    }

    fn flatten(&mut self, program: Program, current_dir: &Path) -> Result<Program, ModuleError> {
        let mut result = Vec::new();
        for node in program {
            match node {
                DeclOrStmt::Stmt(Stmt::Import { path, alias, span }) => {
                    let resolved = self.resolve_path(&path, current_dir)?;
                    if !self.visited.insert(resolved.clone()) {
                        return Err(ModuleError::Circular { path: resolved, span });
                    }
                    let source = fs::read_to_string(&resolved)
                        .map_err(|e| ModuleError::Io {
                            path: resolved.clone(),
                            message: format!("No se pudo leer '{}': {}", resolved.display(), e),
                        })?;
                    let imported_program = parse_source(&source, &resolved)?;
                    let parent = resolved.parent().unwrap_or(Path::new("."));
                    let flat = self.flatten(imported_program, parent)?;
                    self.visited.remove(&resolved);
                    let prefix = alias.unwrap_or_else(|| {
                        resolved.file_stem()
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
            details: lex_errors.iter().map(|e| format!("{} [{}:{}]: {} ({})", e.code, e.pos.line, e.pos.col, e.message, e.suggestion)).collect(),
        });
    }
    let parser = Parser::new(tokens);
    let (program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        return Err(ModuleError::Parse {
            path: path.to_path_buf(),
            details: parse_errors.iter().map(|e| format!("{} [{}:{}]: {} ({})", e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion)).collect(),
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
        Decl::Variable { var_type, name, init, .. } => {
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
        Decl::Function { return_type, name, params, body, .. } => {
            prefix_type(return_type, prefix);
            if top_level {
                *name = format!("{}_{}", prefix, name);
            }
            for p in params.iter_mut() {
                prefix_type(&mut p.param_type, prefix);
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
        Decl::Struct { name, fields, .. } => {
            if top_level {
                *name = format!("{}_{}", prefix, name);
            }
            for field in fields.iter_mut() {
                prefix_type(&mut field.field_type, prefix);
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
        Stmt::If { condition, then_body, else_body, .. } => {
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
        Stmt::While { condition, body, .. } => {
            prefix_expr(condition, prefix, locals);
            let mut while_locals = locals.clone();
            for node in body.iter_mut() {
                prefix_node(node, prefix, &mut while_locals, false);
            }
        }
        Stmt::For { init, condition, update, body, .. } => {
            let mut for_locals = locals.clone();
            if let Decl::Variable { name, .. } = init.as_mut() {
                for_locals.insert(name.clone());
            }
            prefix_expr(condition, prefix, &mut for_locals);
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
        Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Import { .. } => {}
        Stmt::Match { expr, arms, default, .. } => {
            prefix_expr(expr, prefix, locals);
            for arm in arms.iter_mut() {
                prefix_expr(&mut arm.value, prefix, locals);
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
        Expr::Call { callee, args, .. } => {
            prefix_expr(callee, prefix, locals);
            for arg in args.iter_mut() {
                prefix_expr(arg, prefix, locals);
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
        Expr::Index { expr: target, index, .. } => {
            prefix_expr(target, prefix, locals);
            prefix_expr(index, prefix, locals);
        }
        Expr::MethodCall { expr: target, args, .. } => {
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
        Expr::StructInit { struct_name, fields, .. } => {
            *struct_name = format!("{}_{}", prefix, struct_name);
            for (_, value) in fields.iter_mut() {
                prefix_expr(value, prefix, locals);
            }
        }
        Expr::FieldAccess { expr: target, .. } => {
            prefix_expr(target, prefix, locals);
        }
    }
}

fn prefix_type(t: &mut Type, prefix: &str) {
    match t {
        Type::Lista(inner) => prefix_type(inner, prefix),
        Type::Func { param_types, return_type } => {
            for p in param_types.iter_mut() {
                prefix_type(p, prefix);
            }
            prefix_type(return_type, prefix);
        }
        Type::Struct(name) => {
            *name = format!("{}_{}", prefix, name);
        }
        _ => {}
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "imprimir" | "print" | "leer" | "read")
}
