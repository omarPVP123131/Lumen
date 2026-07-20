use crate::ir::*;
use lumen_parser::ast::{BinOp, Decl, DeclOrStmt, Expr, Param, Stmt, UnOp};
use std::collections::{HashMap, HashSet};

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
    fn_names: HashSet<String>,
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
        }
    }

    pub fn build(mut self, program: &[DeclOrStmt]) -> crate::ir::Program {
        let has_toplevel_code = program.iter().any(|node| {
            !matches!(
                node,
                DeclOrStmt::Decl(Decl::Function { .. })
                    | DeclOrStmt::Decl(Decl::Struct { .. })
                    | DeclOrStmt::Decl(Decl::Enum { .. })
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
            self.current_func = Some("__main__".to_string());
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
                if !self
                    .current_instrs
                    .iter()
                    .any(|i| matches!(i, Instr::Return))
                {
                    self.emit(Instr::Return);
                }
                self.finalize_func();
                if self.program.funcs.contains_key("__main__") {
                    self.current_func = Some("__main__".to_string());
                    self.current_instrs = Vec::new();
                } else {
                    self.current_func = None;
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
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assignment { name, value, .. } => {
                self.gen_expr(value);
                self.emit(Instr::Store(name.clone()));
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
            Expr::Binary {
                op, left, right, ..
            } => {
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
                match callee_inner {
                    Expr::Ident { name, .. } => {
                        if self.fn_names.contains(name)
                            || matches!(name.as_str(), "imprimir" | "print" | "leer" | "read")
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
                expr, method, args, ..
            } => {
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
        }
    }

    fn compile_lambda(&mut self, params: &[Param], body: &[DeclOrStmt]) -> String {
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
        lambda_name
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
                    if a % b == 0 {
                        Some(Instr::ConstInt(a / b))
                    } else {
                        Some(Instr::ConstFloat(*a as f64 / *b as f64))
                    }
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
        let instrs = vec![Instr::ConstFloat(3.14), Instr::Unary(Op::Negate)];
        let folded = IRBuilder::fold_constants_pass(&instrs);
        assert_eq!(folded.len(), 1);
        assert!(matches!(folded[0], Instr::ConstFloat(v) if (v - (-3.14)).abs() < f64::EPSILON));
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
