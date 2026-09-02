use lumen_lexer::Lexer;
use lumen_parser::ast::*;
use lumen_parser::Parser;

/// Formatea código fuente LÚMEN.
/// Retorna el código formateado o una lista de errores.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FmtConfig {
    pub indent_spaces: usize,
}

impl Default for FmtConfig {
    fn default() -> Self {
        Self { indent_spaces: 4 }
    }
}

pub fn format_source(source: &str) -> Result<String, Vec<String>> {
    let config = load_config();
    format_source_with_config(source, &config)
}

pub fn load_config() -> FmtConfig {
    if let Ok(content) = std::fs::read_to_string(".lumen-fmt.toml") {
        toml::from_str(&content).unwrap_or_default()
    } else {
        FmtConfig::default()
    }
}

pub fn format_source_with_config(source: &str, config: &FmtConfig) -> Result<String, Vec<String>> {
    let lexer = Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        return Err(lex_errors
            .iter()
            .map(|e| {
                format!(
                    "{} [{}:{}]: {} ({})",
                    e.code, e.pos.line, e.pos.col, e.message, e.suggestion
                )
            })
            .collect());
    }

    let parser = Parser::new(tokens);
    let (program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        return Err(parse_errors
            .iter()
            .map(|e| {
                format!(
                    "{} [{}:{}]: {} ({})",
                    e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
                )
            })
            .collect());
    }

    let mut fmt = Formatter::new(config.indent_spaces);
    fmt.fmt_program(&program);
    Ok(fmt.output.trim_end().to_string() + "\n")
}

struct Formatter {
    output: String,
    indent: usize,
    indent_spaces: usize,
}

impl Formatter {
    fn new(indent_spaces: usize) -> Self {
        Self {
            output: String::new(),
            indent: 0,
            indent_spaces,
        }
    }

    fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }
    fn push_indent(&mut self) {
        for _ in 0..self.indent {
            for _ in 0..self.indent_spaces {
                self.output.push(' ');
            }
        }
    }
    fn newline(&mut self) {
        self.output.push('\n');
    }
    fn indent_inc(&mut self) {
        self.indent += 1;
    }
    fn indent_dec(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    fn fmt_program(&mut self, program: &[DeclOrStmt]) {
        for (i, node) in program.iter().enumerate() {
            if i > 0 {
                self.newline();
            }
            self.fmt_decl_or_stmt(node);
        }
    }

    fn fmt_decl_or_stmt(&mut self, node: &DeclOrStmt) {
        match node {
            DeclOrStmt::Decl(d) => self.fmt_decl(d),
            DeclOrStmt::Stmt(s) => self.fmt_stmt(s, true),
        }
    }

    fn fmt_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Variable {
                var_type,
                name,
                init,
                ..
            } => {
                self.push_indent();
                self.push(&format_type(var_type));
                self.push(" ");
                self.push(name);
                if let Some(val) = init {
                    self.push(" = ");
                    self.fmt_expr(val);
                }
                self.push(";");
                self.newline();
            }
            Decl::Const {
                var_type,
                name,
                value,
                ..
            } => {
                self.push_indent();
                self.push("const ");
                self.push(&format_type(var_type));
                self.push(" ");
                self.push(name);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Decl::Function {
                return_type,
                name,
                params,
                body,
                type_params,
                ..
            } => {
                if !type_params.is_empty() {
                    self.push_indent();
                    self.push("funcion ");
                    self.push(&format_type(return_type));
                    self.push(" ");
                    self.push(name);
                    self.push("<");
                    self.push(&type_params.join(", "));
                    self.push(">");
                    self.push("(");
                    self.fmt_params(params);
                    self.push(") ");
                } else {
                    self.push_indent();
                    self.push("funcion ");
                    self.push(&format_type(return_type));
                    self.push(" ");
                    self.push(name);
                    self.push("(");
                    self.fmt_params(params);
                    self.push(") ");
                }
                self.fmt_block(body);
            }
            Decl::Struct { name, fields, .. } => {
                self.push_indent();
                self.push("estructura ");
                self.push(name);
                self.push(" {");
                self.newline();
                self.indent_inc();
                for f in fields {
                    self.push_indent();
                    self.push(&f.name);
                    self.push(": ");
                    self.push(&format_type(&f.field_type));
                    self.push(",");
                    self.newline();
                }
                self.indent_dec();
                self.push_indent();
                self.push("}");
                self.newline();
            }
            Decl::Enum { name, variants, .. } => {
                self.push_indent();
                self.push("enum ");
                self.push(name);
                self.push(" { ");
                for (i, v) in variants.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.push(&v.name);
                    if !v.types.is_empty() {
                        self.push("(");
                        for (j, t) in v.types.iter().enumerate() {
                            if j > 0 {
                                self.push(", ");
                            }
                            self.push(&format_type(t));
                        }
                        self.push(")");
                    }
                }
                self.push(" }");
                self.newline();
            }
            Decl::Rasgo { name, methods, .. } => {
                self.push_indent();
                self.push("rasgo ");
                self.push(name);
                self.push(" {");
                self.newline();
                self.indent_inc();
                for m in methods {
                    self.push_indent();
                    self.push("funcion ");
                    self.push(&format_type(&m.return_type));
                    self.push(" ");
                    self.push(&m.name);
                    self.push("(");
                    for (j, p) in m.params.iter().enumerate() {
                        if j > 0 {
                            self.push(", ");
                        }
                        self.push(&p.name);
                        self.push(": ");
                        self.push(&format_type(&p.param_type));
                    }
                    self.push(");");
                    self.newline();
                }
                self.indent_dec();
                self.push_indent();
                self.push("}");
                self.newline();
            }
            Decl::ImplRasgo {
                trait_name,
                target_type,
                methods,
                associated_types,
                ..
            } => {
                self.push_indent();
                self.push("impl ");
                // Impl inherente (sin rasgo): `impl Cuenta { ... }` — NO emitir " para "
                // que produce sintaxis inválida `impl  para Cuenta` (bug fmt v3.2.0)
                if !trait_name.is_empty() {
                    self.push(trait_name);
                    self.push(" para ");
                }
                self.push(&format_type(target_type));
                self.push(" {");
                self.newline();
                self.indent_inc();
                for at in associated_types {
                    self.push_indent();
                    self.push(&format!(
                        "tipo {} = {};",
                        at.name,
                        format_type(&at.target_type)
                    ));
                    self.newline();
                }
                for m in methods {
                    if let Decl::Function {
                        return_type,
                        name,
                        params,
                        body,
                        ..
                    } = m
                    {
                        self.push_indent();
                        self.push("funcion ");
                        self.push(&format_type(return_type));
                        self.push(" ");
                        self.push(name);
                        self.push("(");
                        self.fmt_params(params);
                        self.push(") ");
                        self.fmt_block(body);
                    }
                }
                self.indent_dec();
                self.push_indent();
                self.push("}");
                self.newline();
            }
            Decl::Destructure { targets, init, .. } => {
                // Sin este brazo, `entero a, texto b = tupla;` se borraba con fmt
                self.push_indent();
                for (i, t) in targets.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    if let Some(ref ty) = t.var_type {
                        self.push(&format_type(ty));
                        self.push(" ");
                    }
                    self.push(&t.name);
                }
                self.push(" = ");
                self.fmt_expr(init);
                self.push(";");
                self.newline();
            }
        }
    }

    fn fmt_stmt(&mut self, stmt: &Stmt, top_level: bool) {
        match stmt {
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("si ");
                let mut cond = condition.as_ref();
                while let Expr::Grouping { expr, .. } = cond {
                    cond = expr.as_ref();
                }
                self.fmt_expr(cond);
                self.push(" ");
                let has_else = else_body.as_ref().map(|eb| !eb.is_empty()).unwrap_or(false);
                self.fmt_block_inline(then_body, !has_else);
                if let Some(ref eb) = else_body {
                    if !eb.is_empty() {
                        self.push(" sino ");
                        if eb.len() == 1 {
                            if let DeclOrStmt::Stmt(s @ Stmt::If { .. }) = &eb[0] {
                                self.fmt_stmt(s, false);
                            } else {
                                self.fmt_block(eb);
                            }
                        } else {
                            self.fmt_block(eb);
                        }
                    }
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("mientras ");
                let mut cond = condition.as_ref();
                while let Expr::Grouping { expr, .. } = cond {
                    cond = expr.as_ref();
                }
                self.fmt_expr(cond);
                self.push(" ");
                self.fmt_block(body);
            }
            Stmt::ForEach {
                var_name,
                expr,
                body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("para ");
                self.push(var_name);
                self.push(" en ");
                self.fmt_expr(expr);
                self.push(" ");
                self.fmt_block(body);
            }
            Stmt::Return { value, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("retornar");
                if let Some(v) = value {
                    self.push(" ");
                    self.fmt_expr(v);
                }
                self.push(";");
                self.newline();
            }
            Stmt::Break { label, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("romper");
                if let Some(l) = label {
                    self.push(" ");
                    self.push(l);
                }
                self.push(";");
                self.newline();
            }
            Stmt::Continue { label, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("continuar");
                if let Some(l) = label {
                    self.push(" ");
                    self.push(l);
                }
                self.push(";");
                self.newline();
            }
            Stmt::Expr { expr, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.fmt_expr(expr);
                self.push(";");
                self.newline();
            }
            Stmt::Assignment { name, value, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push(name);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Stmt::Match { expr, arms, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("elegir (");
                self.fmt_expr(expr);
                self.push(") {");
                self.newline();
                self.indent_inc();
                for arm in arms {
                    self.push_indent();
                    self.push("caso ");
                    self.fmt_expr(&arm.value);
                    if let Some(ref guard) = arm.guard {
                        self.push(" si ");
                        self.fmt_expr(guard);
                    }
                    self.push(": ");
                    self.fmt_block(&arm.body);
                }
                self.indent_dec();
                self.push_indent();
                self.push("}");
                self.newline();
            }
            Stmt::Block { stmts, .. } => {
                self.fmt_block(stmts);
            }
            Stmt::Posponer { body, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("posponer ");
                self.fmt_block(body);
            }
            Stmt::TryCatch {
                try_body,
                err_var,
                catch_body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("intentar ");
                self.fmt_block(try_body);
                self.push(&format!(" atrapar ({}) ", err_var));
                self.fmt_block(catch_body);
            }
            Stmt::Import { path, alias, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push("importar ");
                self.push(&format!("\"{}\"", path));
                if let Some(a) = alias {
                    self.push(" como ");
                    self.push(a);
                }
                self.push(";");
                self.newline();
            }
            Stmt::InlineAsm { code, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push(&format!("ensamblador {{ \"{}\" }}", escape_string(code)));
                self.newline();
            }
            Stmt::InlineC { code, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push(&format!("bloque_c {{ \"{}\" }}", escape_string(code)));
                self.newline();
            }
            Stmt::InlineRust { code, .. } => {
                if top_level {
                    self.push_indent();
                }
                self.push(&format!("bloque_rust {{ \"{}\" }}", escape_string(code)));
                self.newline();
            }
            Stmt::FieldAssign {
                expr, field, value, ..
            } => {
                // CRÍTICO: sin este brazo, fmt BORRABA silenciosamente asignaciones
                // `obj.campo = valor;` (bug de data loss reportado por QA v3.2.0)
                if top_level {
                    self.push_indent();
                }
                self.fmt_expr(expr);
                self.push(&format!(".{} = ", field));
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Stmt::ArraySet {
                arr, index, value, ..
            } => {
                // Idem: `arr[i] = v;` también se borraba con el catch-all vacío
                if top_level {
                    self.push_indent();
                }
                self.fmt_expr(arr);
                self.push("[");
                self.fmt_expr(index);
                self.push("] = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Stmt::IfLet {
                pattern,
                value,
                then_body,
                else_body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("si sea ");
                self.fmt_expr(pattern);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(" ");
                self.fmt_block(then_body);
                if let Some(eb) = else_body {
                    self.push(" sino ");
                    self.fmt_block(eb);
                }
                self.newline();
            }
            Stmt::GuardLet {
                pattern,
                value,
                else_body,
                ..
            } => {
                if top_level {
                    self.push_indent();
                }
                self.push("sea ");
                self.fmt_expr(pattern);
                self.push(" = ");
                self.fmt_expr(value);
                self.push(" sino ");
                self.fmt_block(else_body);
                self.newline();
            }
            Stmt::Destructure { targets, value, .. } => {
                if top_level {
                    self.push_indent();
                }
                for (i, t) in targets.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    if let Some(ref ty) = t.var_type {
                        self.push(&format_type(ty));
                        self.push(" ");
                    }
                    self.push(&t.name);
                }
                self.push(" = ");
                self.fmt_expr(value);
                self.push(";");
                self.newline();
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
                ..
            } => {
                // Séptimo statement que el catch-all borraba: `para (i=0; i<n; i=i+1) {}`
                if top_level {
                    self.push_indent();
                }
                self.push("para (");
                if let Decl::Variable {
                    ref var_type,
                    ref name,
                    init: Some(ref init_expr),
                    ..
                } = *init.clone()
                {
                    // `para (i = 0; ...)` sin tipo usa Type::Struct("Infer") — no imprimirlo
                    let is_inferred = matches!(var_type, Type::Struct(s) if s == "Infer");
                    if !is_inferred {
                        self.push(&format_type(var_type));
                        self.push(" ");
                    }
                    self.push(name);
                    self.push(" = ");
                    self.fmt_expr(init_expr);
                }
                self.push("; ");
                self.fmt_expr(condition);
                self.push("; ");
                if let Stmt::Assignment {
                    ref name,
                    ref value,
                    ..
                } = *update.clone()
                {
                    self.push(name);
                    self.push(" = ");
                    self.fmt_expr(value);
                }
                self.push(") ");
                self.fmt_block(body);
            }
        }
    }

    fn fmt_block(&mut self, body: &[DeclOrStmt]) {
        self.fmt_block_inline(body, true);
    }

    fn fmt_block_inline(&mut self, body: &[DeclOrStmt], trailing_newline: bool) {
        self.push("{");
        self.newline();
        self.indent_inc();
        for node in body {
            self.fmt_decl_or_stmt(node);
        }
        self.indent_dec();
        self.push_indent();
        self.push("}");
        if trailing_newline {
            self.newline();
        }
    }

    fn fmt_params(&mut self, params: &[Param]) {
        for (i, p) in params.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            // Receptor: `este`/`self`/`yo` o tipo especial Self del parser — sin anotación,
            // SALVO que declare `prestado mut` (receiver mutable, v3.3.5+): el tipo
            // DEBE imprimirse o el formatter perdería la semántica por referencia.
            let is_receiver = p.name == "self"
                || p.name == "yo"
                || p.name == "este"
                || matches!(&p.param_type, Type::Struct(s) if s == "Self");
            let is_mut_receiver = matches!(&p.param_type, Type::Prestado { mutable: true, .. });
            if is_receiver && !is_mut_receiver {
                self.push(&p.name);
            } else if is_mut_receiver {
                // `prestado mut este`: el interior es el placeholder Self del
                // parser; la forma canónica usa solo el nombre del receptor
                self.push("prestado mut ");
                self.push(&p.name);
            } else {
                self.push(&format_type(&p.param_type));
                self.push(" ");
                self.push(&p.name);
            }
        }
    }

    fn fmt_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int { value, .. } => self.push(&value.to_string()),
            Expr::Float { value, .. } => self.push(&value.to_string()),
            Expr::Str { value, .. } => self.push(&format!("\"{}\"", escape_string(value))),
            Expr::Bool { value, .. } => self.push(if *value { "verdadero" } else { "falso" }),
            Expr::Ident { name, .. } => self.push(name),
            Expr::Binary {
                op, left, right, ..
            } => {
                self.fmt_expr(left);
                self.push(" ");
                self.push(fmt_binop(op));
                self.push(" ");
                self.fmt_expr(right);
            }
            Expr::Unary { op, operand, .. } => {
                self.push(fmt_unop(op));
                self.fmt_expr(operand);
            }
            Expr::Ternary {
                condition,
                true_branch,
                false_branch,
                ..
            } => {
                self.fmt_expr(condition);
                self.push(" ? ");
                self.fmt_expr(true_branch);
                self.push(" : ");
                self.fmt_expr(false_branch);
            }
            Expr::Call {
                callee,
                args,
                type_args,
                ..
            } => {
                self.fmt_expr(callee);
                if !type_args.is_empty() {
                    self.push("<");
                    for (i, t) in type_args.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.push(&format_type(t));
                    }
                    self.push(">");
                }
                self.push("(");
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.fmt_expr(a);
                }
                self.push(")");
            }
            Expr::List { items, .. } => {
                self.push("[");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.fmt_expr(item);
                }
                self.push("]");
            }
            Expr::Index { expr, index, .. } => {
                self.fmt_expr(expr);
                self.push("[");
                self.fmt_expr(index);
                self.push("]");
            }
            Expr::MethodCall {
                expr, method, args, ..
            } => {
                self.fmt_expr(expr);
                self.push(".");
                self.push(method);
                self.push("(");
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.fmt_expr(a);
                }
                self.push(")");
            }
            Expr::FieldAccess { expr, field, .. } => {
                self.fmt_expr(expr);
                self.push(".");
                self.push(field);
            }
            Expr::SafeFieldAccess { expr, field, .. } => {
                self.fmt_expr(expr);
                self.push("?.");
                self.push(field);
            }
            Expr::Elvis { expr, default, .. } => {
                self.fmt_expr(expr);
                self.push(" ?: ");
                self.fmt_expr(default);
            }
            Expr::Comprehension {
                expr,
                var_name,
                iter,
                condition,
                ..
            } => {
                self.push("[");
                self.fmt_expr(expr);
                self.push(&format!(" para {} en ", var_name));
                self.fmt_expr(iter);
                if let Some(cond) = condition {
                    self.push(" si ");
                    self.fmt_expr(cond);
                }
                self.push("]");
            }
            Expr::Query {
                var_name,
                source,
                where_clause,
                select_expr,
                ..
            } => {
                self.push(&format!("consultar {} en ", var_name));
                self.fmt_expr(source);
                if let Some(w) = where_clause {
                    self.push(" donde ");
                    self.fmt_expr(w);
                }
                self.push(" seleccionar ");
                self.fmt_expr(select_expr);
            }
            Expr::StructInit {
                struct_name: name,
                fields,
                ..
            } => {
                self.push(name);
                self.push(" { ");
                for (i, (fname, fexpr)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.push(fname);
                    self.push(": ");
                    self.fmt_expr(fexpr);
                }
                self.push(" }");
            }
            Expr::Grouping { expr, .. } => {
                self.push("(");
                self.fmt_expr(expr);
                self.push(")");
            }
            Expr::Cast {
                expr, cast_type, ..
            } => {
                self.push("(");
                self.fmt_expr(expr);
                self.push(" como ");
                self.push(&format_type(cast_type));
                self.push(")");
            }
            Expr::Tuple { items, .. } => {
                self.push("(");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.fmt_expr(item);
                }
                self.push(")");
            }
            Expr::EnumCtor {
                enum_name,
                variant,
                args,
                ..
            } => {
                self.push(enum_name);
                self.push("::");
                self.push(variant);
                if !args.is_empty() {
                    self.push("(");
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        self.fmt_expr(a);
                    }
                    self.push(")");
                }
            }
            Expr::Comptime { expr, .. } => {
                self.push("en_tiempo_compilacion { ");
                self.fmt_expr(expr);
                self.push(" }");
            }
            Expr::Range {
                start,
                end,
                inclusive,
                ..
            } => {
                // Sin este brazo, `0..5` se formateaba como cadena vacía → `lista<entero> r = ;`
                self.fmt_expr(start);
                self.push(if *inclusive { "..=" } else { ".." });
                self.fmt_expr(end);
            }
            Expr::Algun { expr, .. } => {
                self.push("algun(");
                self.fmt_expr(expr);
                self.push(")");
            }
            Expr::Ninguno { .. } => {
                self.push("ninguno");
            }
            Expr::Exito { expr, .. } => {
                self.push("exito(");
                self.fmt_expr(expr);
                self.push(")");
            }
            Expr::Error { expr, .. } => {
                self.push("error(");
                self.fmt_expr(expr);
                self.push(")");
            }
            Expr::Intentar { expr, .. } => {
                self.push("intentar ");
                self.fmt_expr(expr);
            }
            Expr::TupleAccess { expr, index, .. } => {
                self.fmt_expr(expr);
                self.push(&format!(".{}", index));
            }
            Expr::Lambda { params, body, .. } => {
                self.push("funcion(");
                self.fmt_params(params);
                self.push(") ");
                self.fmt_block(body);
            }
            _ => {}
        }
    }
}

fn format_type(t: &Type) -> String {
    match t {
        Type::Entero => "entero".to_string(),
        Type::Decimal => "decimal".to_string(),
        Type::Texto => "texto".to_string(),
        Type::Booleano => "booleano".to_string(),
        Type::Numero => "numero".to_string(),
        Type::Lista(inner) => format!("lista<{}>", format_type(inner)),
        Type::Resultado { ok, err } => {
            format!("Resultado<{}, {}>", format_type(ok), format_type(err))
        }
        Type::Opcion(inner) => format!("Opcion<{}>", format_type(inner)),
        Type::Func {
            param_types,
            return_type,
        } => {
            let params: Vec<String> = param_types.iter().map(format_type).collect();
            format!(
                "funcion({}) {}",
                params.join(", "),
                format_type(return_type)
            )
        }
        Type::Struct(name) => name.clone(),
        Type::Tuple(types) => {
            let inner: Vec<String> = types.iter().map(format_type).collect();
            format!("({})", inner.join(", "))
        }
        Type::GenericStruct { name, .. } => name.clone(),
        Type::ImplTrait(name) => format!("impl {}", name),
        Type::Prestado { inner, mutable } => {
            if *mutable {
                format!("prestado mut {}", format_type(inner))
            } else {
                format!("prestado {}", format_type(inner))
            }
        }
        Type::Dueno(inner) => format!("dueno {}", format_type(inner)),
    }
}

fn fmt_binop(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Concat => "++",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Equal => "==",
        BinOp::NotEqual => "!=",
        BinOp::Less => "<",
        BinOp::LessEqual => "<=",
        BinOp::Greater => ">",
        BinOp::GreaterEqual => ">=",
        BinOp::And => "&&",
        BinOp::Or => "||",
        BinOp::BitOr => "|",
        BinOp::BitAnd => "&",
        BinOp::BitXor => "^",
        BinOp::ShiftLeft => "<<",
        BinOp::ShiftRight => ">>",
    }
}

fn fmt_unop(op: &UnOp) -> &'static str {
    match op {
        UnOp::Negate => "-",
        UnOp::Not => "!",
        UnOp::BitNot => "~",
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple() {
        let result = format_source("imprimir(\"hola\");").unwrap();
        assert!(result.contains("hola"));
    }

    #[test]
    fn test_format_function() {
        let src = "funcion entero suma(entero a,entero b){retornar a+b;}";
        let result = format_source(src).unwrap();
        assert!(result.contains("funcion entero suma"));
    }

    // === REGRESIÓN QA v3.2.0: fmt borraba statements silenciosamente ===

    /// QA #1 (crítico): `obj.campo = valor;` desaparecía tras fmt
    #[test]
    fn test_fmt_preserves_field_assign() {
        let src = "estructura Cuenta { saldo: entero, }\nfuncion entero principal() { Cuenta cuenta = Cuenta { saldo: 1000 }; cuenta.saldo = 500; retornar 0; }";
        let result = format_source(src).unwrap();
        assert!(
            result.contains("cuenta.saldo = 500"),
            "fmt BORRÓ la asignación a campo: {}",
            result
        );
    }

    #[test]
    fn test_fmt_preserves_field_assign_selfref() {
        let src = "funcion void f() { cuenta.saldo = cuenta.saldo + 250; }";
        let result = format_source(src).unwrap();
        assert!(
            result.contains("cuenta.saldo"),
            "borró self-ref field assign"
        );
    }

    /// QA: `arr[i] = v;` también se borraba
    #[test]
    fn test_fmt_preserves_array_set() {
        let src = "funcion void f() { lista<entero> arr=[1,2]; arr[0] = 99; }";
        let result = format_source(src).unwrap();
        assert!(
            result.contains("arr[0] = 99")
                || result.contains("arr[0]= 99")
                || result.contains("arr[0]"),
            "borró ArraySet: {}",
            result
        );
    }

    /// QA: `para (init; cond; paso) {}` clásico se borraba
    #[test]
    fn test_fmt_preserves_classic_for() {
        let src = "funcion void f() { para (entero i = 0; i < 3; i = i + 1) { imprimir(i); } }";
        let result = format_source(src).unwrap();
        assert!(result.contains("para ("), "borró for clásico: {}", result);
        assert!(result.contains("i < 3"), "perdió condición del for");
    }

    /// Expresiones que se formateaban como cadena vacía
    #[test]
    fn test_fmt_preserves_range_expr() {
        let src = "funcion void f() { lista<entero> r = 0..5; }";
        let result = format_source(src).unwrap();
        assert!(result.contains(".."), "borró rango: {}", result);
    }

    #[test]
    fn test_fmt_preserves_algun_ninguno() {
        let src = "funcion void f(opcion<entero> o) { si sea algun(x) = o { imprimir(x); } }";
        let result = format_source(src).unwrap();
        assert!(result.contains("algun"), "borró patrón algun: {}", result);
    }

    /// Idempotencia semántica: fmt dos veces == fmt una vez
    #[test]
    fn test_fmt_idempotent_field_assign() {
        let src = "estructura C { saldo: entero, }\nfuncion void f() { C c = C { saldo: 1 }; c.saldo = 2; }";
        let once = format_source(src).unwrap();
        let twice = format_source(&once).unwrap();
        assert_eq!(once, twice, "fmt no es idempotente");
        assert!(twice.contains("c.saldo = 2"));
    }
}

#[cfg(test)]
mod tests_v338 {
    use super::*;

    fn fmt_of(src: &str) -> String {
        let once = format_source(src).expect("fmt fallo");
        format_source(&once).expect("fmt 2do pase fallo")
    }

    #[test]
    fn test_fmt_prestado_mut_este_idempotent() {
        // v3.3.5+: receiver mutable de método sobrevive al formatter
        let src = "estructura C { saldo: entero }\nimpl C {\n\tfuncion vacio dup(prestado mut este) {\n\t\teste.saldo = este.saldo * 2;\n\t}\n}\n";
        let once = fmt_of(src);
        assert!(
            once.contains("prestado mut este"),
            "fmt perdió el receiver: {}",
            once
        );
        assert_eq!(once, fmt_of(&once), "fmt no es idempotente");
    }
}
