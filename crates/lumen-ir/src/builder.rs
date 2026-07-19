use std::collections::HashMap;
use lumen_parser::ast::{DeclOrStmt, Decl, Stmt, Expr, BinOp, UnOp};
use crate::ir::*;

struct LoopLabels {
    break_label: usize,
    continue_label: usize,
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
        }
    }

    pub fn build(mut self, program: &[DeclOrStmt]) -> crate::ir::Program {
        let has_toplevel_code = program.iter().any(|node| {
            !matches!(node, DeclOrStmt::Decl(Decl::Function { .. }))
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
            }
        }

        for node in program {
            if let DeclOrStmt::Decl(Decl::Function { name, params, .. }) = node {
                let defaults: Vec<Option<Expr>> = params.iter().map(|p| {
                    p.default.clone().map(|boxed| *boxed)
                }).collect();
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
            self.current_func = Some("__main__".to_string());
        }

        for node in program {
            self.gen_decl_or_stmt(node);
        }

        if self.current_instrs.last().map_or(true, |i| !matches!(i, Instr::Halt)) {
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
                    self.emit(Instr::Store(name.clone()));
                }
            }
            Decl::Function { name, body, .. } => {
                self.finalize_func();
                self.current_func = Some(name.clone());
                self.current_instrs = Vec::new();
                self.temp_counter = 0;
                self.label_counter = 0;
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                if !self.current_instrs.iter().any(|i| matches!(i, Instr::Return)) {
                    self.emit(Instr::Return);
                }
                self.finalize_func();
                if self.program.funcs.contains_key("__main__") {
                    self.current_func = Some("__main__".to_string());
                    self.current_instrs = Vec::new();
                }
            }
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assignment { name, value, .. } => {
                self.gen_expr(value);
                self.emit(Instr::Store(name.clone()));
            }
            Stmt::If { condition, then_body, else_body, .. } => {
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
            Stmt::While { condition, body, .. } => {
                let start_label = self.new_label();
                let end_label = self.new_label();
                self.emit(Instr::Label(start_label));
                self.gen_expr(condition);
                self.emit(Instr::JmpIf(end_label));
                self.loop_labels.push(LoopLabels {
                    break_label: end_label,
                    continue_label: start_label,
                });
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                self.loop_labels.pop();
                self.emit(Instr::Jmp(start_label));
                self.emit(Instr::Label(end_label));
            }
            Stmt::For { init, condition, update, body, .. } => {
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
            Stmt::Expr { expr, .. } => {
                self.gen_expr(expr);
            }
            Stmt::Break { .. } => {
                if let Some(labels) = self.loop_labels.last() {
                    self.emit(Instr::Jmp(labels.break_label));
                }
            }
            Stmt::Continue { .. } => {
                if let Some(labels) = self.loop_labels.last() {
                    self.emit(Instr::Jmp(labels.continue_label));
                }
            }
            Stmt::Match { expr, arms, default, .. } => {
                let end_label = self.new_label();
                let mut next_label = self.new_label();
                for arm in arms {
                    self.emit(Instr::Label(next_label));
                    self.gen_expr(expr);
                    self.gen_expr(&arm.value);
                    self.emit(Instr::Binary(Op::Equal));
                    next_label = self.new_label();
                    self.emit(Instr::JmpIf(next_label));
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
                self.emit(Instr::Load(name.clone()));
            }
            Expr::Binary { op, left, right, .. } => {
                self.gen_expr(left);
                self.gen_expr(right);
                self.emit(Instr::Binary(match op {
                    BinOp::Add => Op::Add,
                    BinOp::Sub => Op::Sub,
                    BinOp::Mul => Op::Mul,
                    BinOp::Div => Op::Div,
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
                let callee_name = match callee_inner {
                    Expr::Ident { name, .. } => name.clone(),
                    Expr::Lambda { params, body, .. } => {
                        let lambda_name = format!("__lambda_{}", self.lambda_counter);
                        self.lambda_counter += 1;
                        let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
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
                        self.current_func = Some(lambda_name.clone());
                        self.current_instrs = Vec::new();
                        self.temp_counter = 0;
                        self.label_counter = 0;
                        for node in body {
                            self.gen_decl_or_stmt(node);
                        }
                        if !self.current_instrs.iter().any(|i| matches!(i, Instr::Return)) {
                            self.emit(Instr::Return);
                        }
                        self.finalize_func();
                        self.current_func = saved_func;
                        self.current_instrs = saved_instrs;
                        self.temp_counter = saved_temp;
                        self.label_counter = saved_label;
                        self.loop_labels = saved_loop;
                        lambda_name
                    }
                    _ => String::new(),
                };
                for arg in args {
                    self.gen_expr(arg);
                }
                let defaults = self.default_params.get(&callee_name).cloned();
                let argc = if let Some(defaults) = defaults {
                    let mut count = args.len();
                    for i in args.len()..defaults.len() {
                        if let Some(default_expr) = &defaults[i] {
                            self.gen_expr(default_expr);
                            count += 1;
                        }
                    }
                    count
                } else {
                    args.len()
                };
                if !callee_name.is_empty() {
                    self.emit(Instr::Call(callee_name, argc));
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
            Expr::MethodCall { expr, method, args, .. } => {
                let var_name = match expr.as_ref() {
                    Expr::Ident { name, .. } => Some(name.clone()),
                    _ => None,
                };
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
            Expr::Grouping { expr, .. } => {
                self.gen_expr(expr);
            }
            Expr::Lambda { params, body, .. } => {
                let lambda_name = format!("__lambda_{}", self.lambda_counter);
                self.lambda_counter += 1;
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
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
                self.current_func = Some(lambda_name.clone());
                self.current_instrs = Vec::new();
                self.temp_counter = 0;
                self.label_counter = 0;
                for node in body {
                    self.gen_decl_or_stmt(node);
                }
                if !self.current_instrs.iter().any(|i| matches!(i, Instr::Return)) {
                    self.emit(Instr::Return);
                }
                self.finalize_func();
                self.current_func = saved_func;
                self.current_instrs = saved_instrs;
                self.temp_counter = saved_temp;
                self.label_counter = saved_label;
                self.loop_labels = saved_loop;
            }
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
            }
        }
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
        let source = "booleano flag = verdadero; si (flag) { numero x = 1; } sino { numero y = 2; }";
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
}
