use lumen_lexer::Lexer;
use lumen_parser::ast::*;
use lumen_parser::Parser;

/// Formatea código fuente LÚMEN.
/// Retorna el código formateado o una lista de errores.
pub fn format_source(source: &str) -> Result<String, Vec<String>> {
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

    let mut fmt = Formatter::new();
    fmt.fmt_program(&program);
    Ok(fmt.output.trim_end().to_string() + "\n")
}

struct Formatter {
    output: String,
    indent: usize,
}

impl Formatter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn push(&mut self, s: &str) {
        self.output.push_str(s);
    }
    fn push_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
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
                ..
            } => {
                self.push_indent();
                self.push("impl ");
                self.push(trait_name);
                self.push(" para ");
                self.push(&format_type(target_type));
                self.push(" {");
                self.newline();
                self.indent_inc();
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
            _ => {}
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
                self.push("(");
                self.fmt_expr(condition);
                self.push(") ");
                self.fmt_block(then_body);
                if let Some(ref eb) = else_body {
                    if !eb.is_empty() {
                        self.push(" sino ");
                        if eb.len() == 1 {
                            if let DeclOrStmt::Stmt(Stmt::If { .. }) = &eb[0] {
                                self.fmt_stmt(
                                    if let DeclOrStmt::Stmt(s) = &eb[0] {
                                        s
                                    } else {
                                        return;
                                    },
                                    false,
                                );
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
                self.push("(");
                self.fmt_expr(condition);
                self.push(") ");
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
            _ => {}
        }
    }

    fn fmt_block(&mut self, body: &[DeclOrStmt]) {
        self.push("{");
        self.newline();
        self.indent_inc();
        for node in body {
            self.fmt_decl_or_stmt(node);
        }
        self.indent_dec();
        self.push_indent();
        self.push("}");
        self.newline();
    }

    fn fmt_params(&mut self, params: &[Param]) {
        for (i, p) in params.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            if p.name == "self" || p.name == "yo" {
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
            let params: Vec<String> = param_types.iter().map(|t| format_type(t)).collect();
            format!(
                "funcion({}) {}",
                params.join(", "),
                format_type(return_type)
            )
        }
        Type::Struct(name) => name.clone(),
        Type::Tuple(types) => {
            let inner: Vec<String> = types.iter().map(|t| format_type(t)).collect();
            format!("({})", inner.join(", "))
        }
        Type::GenericStruct { name, .. } => name.clone(),
    }
}

fn fmt_binop(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
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
    }
}

fn fmt_unop(op: &UnOp) -> &'static str {
    match op {
        UnOp::Negate => "-",
        UnOp::Not => "!",
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
}
