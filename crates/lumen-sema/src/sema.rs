use crate::error::SemError;
use lumen_lexer::token::{Pos, Span};
use lumen_parser::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Numero,
    Entero,
    Decimal,
    Texto,
    Booleano,
    Void,
    Lista(Box<TypeInfo>),
    Func {
        param_types: Vec<TypeInfo>,
        return_type: Box<TypeInfo>,
    },
    Struct {
        name: String,
        fields: Vec<(String, TypeInfo)>,
    },
    Resultado {
        ok: Box<TypeInfo>,
        err: Box<TypeInfo>,
    },
    Opcion(Box<TypeInfo>),
    Enum(String),
    Tuple(Vec<TypeInfo>),
    TypeVar(String),
    Prestado {
        inner: Box<TypeInfo>,
        mutable: bool,
    },
    Dueno(Box<TypeInfo>),
}

#[derive(Clone)]
#[allow(dead_code)]
struct Symbol {
    var_type: TypeInfo,
    name: String,
    declared: Span,
}

struct Scope {
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    fn define(&mut self, name: &str, var_type: TypeInfo, span: Span) -> Result<(), SemError> {
        if self.symbols.contains_key(name) {
            return Err(SemError {
                code: "E032".to_string(),
                message: format!("La variable '{}' ya está declarada en este ámbito", name),
                span,
                suggestion: format!(
                    "Usa un nombre diferente o elimina la declaración anterior de '{}'",
                    name
                ),
            });
        }
        self.symbols.insert(
            name.to_string(),
            Symbol {
                var_type,
                name: name.to_string(),
                declared: span,
            },
        );
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

type FuncSig = (TypeInfo, Vec<TypeInfo>, usize, Vec<String>);
type StructDef = (Vec<(String, TypeInfo)>, Vec<String>);
type TraitSig = (
    Vec<(String, Vec<TypeInfo>, TypeInfo)>,
    Vec<AssociatedTypeDef>,
);
type ImplKey = (String, String);

#[derive(Debug, Clone)]
pub struct AssociatedTypeDef {
    pub name: String,
    pub default: Option<TypeInfo>,
}

pub struct SemanticAnalyzer {
    scopes: Vec<Scope>,
    functions: HashMap<String, FuncSig>,
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, Vec<(String, Vec<TypeInfo>)>>,
    traits: HashMap<String, TraitSig>,
    impls: HashMap<ImplKey, Vec<usize>>,
    impl_methods: Vec<(String, FuncSig)>,
    seen_impls: std::collections::HashSet<ImplKey>,
    type_param_bounds: HashMap<String, Vec<(String, String)>>,
    errors: Vec<SemError>,
    loop_depth: usize,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            traits: HashMap::new(),
            impls: HashMap::new(),
            impl_methods: Vec::new(),
            seen_impls: std::collections::HashSet::new(),
            type_param_bounds: HashMap::new(),
            errors: Vec::new(),
            loop_depth: 0,
        }
    }

    pub fn analyze(mut self, program: &mut Program) -> Vec<SemError> {
        self.collect_enums(program);
        self.collect_traits(program);
        self.collect_structs(program);
        self.collect_impls(program);
        self.collect_functions(program);
        self.analyze_program(program);
        self.resolve_operator_overloads(program);
        self.errors
    }

    fn resolve_operator_overloads(&self, program: &mut Program) {
        // Build a map of known variable types from the program
        let mut var_types: HashMap<String, TypeInfo> = HashMap::new();
        Self::collect_var_types(program, &mut var_types);
        self.resolve_op_program(program, &mut var_types);
    }

    fn collect_var_types(program: &mut Program, var_types: &mut HashMap<String, TypeInfo>) {
        for node in program.iter() {
            if let DeclOrStmt::Decl(Decl::Variable {
                name,
                init: Some(init),
                ..
            }) = node
            {
                let t = Self::infer_static_type(init);
                if t != TypeInfo::Void {
                    var_types.insert(name.clone(), t);
                }
            }
            if let DeclOrStmt::Stmt(Stmt::Assignment { name, value, .. }) = node {
                let t = Self::infer_static_type(value);
                if t != TypeInfo::Void {
                    var_types.insert(name.clone(), t);
                }
            }
            // Recurse into function bodies
            if let DeclOrStmt::Decl(Decl::Function { body, .. }) = node {
                for child in body {
                    Self::collect_var_types_in_block(child, var_types);
                }
            }
        }
    }

    fn collect_var_types_in_block(node: &DeclOrStmt, var_types: &mut HashMap<String, TypeInfo>) {
        match node {
            DeclOrStmt::Decl(Decl::Variable {
                name,
                init: Some(init),
                ..
            }) => {
                let t = Self::infer_static_type(init);
                if t != TypeInfo::Void {
                    var_types.insert(name.clone(), t);
                }
            }
            DeclOrStmt::Stmt(stmt) => Self::collect_var_types_in_stmt(stmt, var_types),
            _ => {}
        }
    }

    fn collect_var_types_in_stmt(stmt: &Stmt, var_types: &mut HashMap<String, TypeInfo>) {
        match stmt {
            Stmt::Block { stmts, .. } => {
                for s in stmts {
                    Self::collect_var_types_in_block(s, var_types);
                }
            }
            Stmt::Posponer { body, .. } => {
                for s in body {
                    Self::collect_var_types_in_block(s, var_types);
                }
            }
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                for s in then_body {
                    Self::collect_var_types_in_block(s, var_types);
                }
                if let Some(eb) = else_body {
                    for s in eb {
                        Self::collect_var_types_in_block(s, var_types);
                    }
                }
            }
            Stmt::While { body, .. } => {
                for s in body {
                    Self::collect_var_types_in_block(s, var_types);
                }
            }
            Stmt::ForEach {
                expr: foreach_expr,
                body,
                ..
            } => {
                Self::collect_var_types_in_expr(foreach_expr, var_types);
                for s in body {
                    Self::collect_var_types_in_block(s, var_types);
                }
            }
            _ => {}
        }
    }

    fn collect_var_types_in_expr(_expr: &Expr, _var_types: &mut HashMap<String, TypeInfo>) {
        // Skip — type handled by infer_static_type
    }

    fn infer_static_type(expr: &Expr) -> TypeInfo {
        match expr {
            Expr::Int { .. } => TypeInfo::Entero,
            Expr::Float { .. } => TypeInfo::Decimal,
            Expr::Str { .. } => TypeInfo::Texto,
            Expr::Bool { .. } => TypeInfo::Booleano,
            Expr::StructInit { struct_name, .. } => TypeInfo::Struct {
                name: struct_name.clone(),
                fields: vec![],
            },
            Expr::List { items, .. } => {
                if items.is_empty() {
                    TypeInfo::Lista(Box::new(TypeInfo::Numero))
                } else {
                    TypeInfo::Lista(Box::new(Self::infer_static_type(&items[0])))
                }
            }
            _ => TypeInfo::Void,
        }
    }

    fn resolve_op_program(&self, program: &mut Program, var_types: &mut HashMap<String, TypeInfo>) {
        for node in program.iter_mut() {
            self.resolve_op_decl_or_stmt_var(node, var_types);
        }
    }

    fn resolve_op_decl_or_stmt_var(
        &self,
        dos: &mut DeclOrStmt,
        var_types: &mut HashMap<String, TypeInfo>,
    ) {
        match dos {
            DeclOrStmt::Stmt(stmt) => self.resolve_op_stmt_var(stmt, var_types),
            DeclOrStmt::Decl(decl) => self.resolve_op_decl_var(decl, var_types),
        }
    }

    fn resolve_op_decl_var(&self, decl: &mut Decl, var_types: &mut HashMap<String, TypeInfo>) {
        match decl {
            Decl::Function { body, .. } => {
                for node in body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(node, var_types);
                }
            }
            Decl::Variable {
                name,
                init: Some(init),
                ..
            } => {
                let t = Self::infer_static_type(init);
                if t != TypeInfo::Void {
                    var_types.insert(name.clone(), t);
                }
                self.resolve_op_expr_var(init, var_types);
            }
            _ => {}
        }
    }

    fn resolve_op_stmt_var(&self, stmt: &mut Stmt, var_types: &mut HashMap<String, TypeInfo>) {
        match stmt {
            Stmt::Expr { expr, .. } => self.resolve_op_expr_var(expr, var_types),
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.resolve_op_expr_var(condition, var_types);
                for n in then_body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(n, var_types);
                }
                if let Some(eb) = else_body {
                    for n in eb.iter_mut() {
                        self.resolve_op_decl_or_stmt_var(n, var_types);
                    }
                }
            }
            Stmt::IfLet {
                value,
                then_body,
                else_body,
                ..
            } => {
                self.resolve_op_expr_var(value, var_types);
                for n in then_body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(n, var_types);
                }
                if let Some(eb) = else_body {
                    for n in eb.iter_mut() {
                        self.resolve_op_decl_or_stmt_var(n, var_types);
                    }
                }
            }
            Stmt::GuardLet {
                value, else_body, ..
            } => {
                self.resolve_op_expr_var(value, var_types);
                for n in else_body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(n, var_types);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.resolve_op_expr_var(condition, var_types);
                for n in body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(n, var_types);
                }
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
                ..
            } => {
                self.resolve_op_decl_var(init, var_types);
                self.resolve_op_expr_var(condition, var_types);
                self.resolve_op_stmt_var(update, var_types);
                for n in body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(n, var_types);
                }
            }
            Stmt::ForEach { expr, body, .. } => {
                self.resolve_op_expr_var(expr, var_types);
                for n in body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(n, var_types);
                }
            }
            Stmt::Match {
                expr,
                arms,
                default,
                ..
            } => {
                self.resolve_op_expr_var(expr, var_types);
                for arm in arms.iter_mut() {
                    self.resolve_op_expr_var(&mut arm.value, var_types);
                    if let Some(ref mut guard) = arm.guard {
                        self.resolve_op_expr_var(guard, var_types);
                    }
                    for n in arm.body.iter_mut() {
                        self.resolve_op_decl_or_stmt_var(n, var_types);
                    }
                }
                if let Some(db) = default {
                    for n in db.iter_mut() {
                        self.resolve_op_decl_or_stmt_var(n, var_types);
                    }
                }
            }
            Stmt::Block { stmts, .. } => {
                for n in stmts.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(n, var_types);
                }
            }
            Stmt::Posponer { body, .. } => {
                for n in body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(n, var_types);
                }
            }
            Stmt::Destructure { value, .. } => self.resolve_op_expr_var(value, var_types),
            Stmt::Assignment { name, value, .. } => {
                self.resolve_op_expr_var(value, var_types);
                if !var_types.contains_key(name) {
                    var_types.insert(name.clone(), Self::infer_static_type(value));
                }
            }
            Stmt::Return {
                value: Some(val), ..
            } => self.resolve_op_expr_var(val, var_types),
            Stmt::Return { value: None, .. } => {}
            _ => {}
        }
    }

    fn resolve_op_expr_var(&self, expr: &mut Expr, var_types: &mut HashMap<String, TypeInfo>) {
        match expr {
            Expr::Binary {
                op,
                left,
                right,
                resolved_method,
                ..
            } => {
                let lt = Self::infer_expr_type_from_map(left, var_types);
                let method_name = Self::op_to_method_name(op);
                if let Some(mname) = method_name {
                    let type_name = match &lt {
                        TypeInfo::Struct { name, .. } => Some(name.clone()),
                        _ => type_info_to_impl_name(&lt),
                    };
                    if let Some(ref type_name) = type_name {
                        if resolved_method.is_none() {
                            if let Some(mangled) = self.find_op_method(type_name, mname) {
                                *resolved_method = Some(mangled);
                            }
                        }
                    }
                }
                self.resolve_op_expr_var(left, var_types);
                self.resolve_op_expr_var(right, var_types);
            }
            Expr::Grouping { expr, .. } => self.resolve_op_expr_var(expr, var_types),
            Expr::Cast { expr, .. } => self.resolve_op_expr_var(expr, var_types),
            Expr::Unary { operand, .. } => self.resolve_op_expr_var(operand, var_types),
            Expr::Call { callee, args, .. } => {
                self.resolve_op_expr_var(callee, var_types);
                for arg in args.iter_mut() {
                    self.resolve_op_expr_var(arg, var_types);
                }
            }
            Expr::MethodCall {
                expr,
                method,
                args,
                ref mut resolved_func,
                span,
            } => {
                let expr_type = Self::infer_expr_type_from_map(expr, var_types);
                let mut arg_types = Vec::new();
                for arg in args.iter_mut() {
                    arg_types.push(Self::infer_expr_type_from_map(arg, var_types));
                }
                if resolved_func.is_none() {
                    if let Some((_, mangled)) =
                        self.resolve_trait_method_mangled(&expr_type, method, &arg_types, span)
                    {
                        *resolved_func = Some(mangled);
                    }
                }
                self.resolve_op_expr_var(expr, var_types);
                for arg in args.iter_mut() {
                    self.resolve_op_expr_var(arg, var_types);
                }
            }
            Expr::FieldAccess { expr, .. } => self.resolve_op_expr_var(expr, var_types),
            Expr::Index { expr, index, .. } => {
                self.resolve_op_expr_var(expr, var_types);
                self.resolve_op_expr_var(index, var_types);
            }
            Expr::Range { start, end, .. } => {
                self.resolve_op_expr_var(start, var_types);
                self.resolve_op_expr_var(end, var_types);
            }
            Expr::StructInit { fields, .. } => {
                for (_, val) in fields.iter_mut() {
                    self.resolve_op_expr_var(val, var_types);
                }
            }
            Expr::List { items, .. } => {
                for item in items.iter_mut() {
                    self.resolve_op_expr_var(item, var_types);
                }
            }
            Expr::Ternary {
                condition,
                true_branch,
                false_branch,
                ..
            } => {
                self.resolve_op_expr_var(condition, var_types);
                self.resolve_op_expr_var(true_branch, var_types);
                self.resolve_op_expr_var(false_branch, var_types);
            }
            Expr::Lambda { body, .. } => {
                for node in body.iter_mut() {
                    self.resolve_op_decl_or_stmt_var(node, var_types);
                }
            }
            _ => {}
        }
    }

    fn infer_expr_type_from_map(expr: &Expr, var_types: &HashMap<String, TypeInfo>) -> TypeInfo {
        match expr {
            Expr::Int { .. } => TypeInfo::Entero,
            Expr::Float { .. } => TypeInfo::Decimal,
            Expr::Str { .. } => TypeInfo::Texto,
            Expr::Bool { .. } => TypeInfo::Booleano,
            Expr::Ident { name, .. } => {
                if name == "verdadero" || name == "true" || name == "falso" || name == "false" {
                    TypeInfo::Booleano
                } else if let Some(t) = var_types.get(name) {
                    t.clone()
                } else {
                    TypeInfo::Void
                }
            }
            Expr::StructInit { struct_name, .. } => TypeInfo::Struct {
                name: struct_name.clone(),
                fields: vec![],
            },
            Expr::Binary {
                left, op, right, ..
            } => {
                let lt = Self::infer_expr_type_from_map(left, var_types);
                let rt = Self::infer_expr_type_from_map(right, var_types);
                match op {
                    BinOp::Equal
                    | BinOp::NotEqual
                    | BinOp::Less
                    | BinOp::LessEqual
                    | BinOp::Greater
                    | BinOp::GreaterEqual => TypeInfo::Booleano,
                    BinOp::Add if lt == TypeInfo::Texto || rt == TypeInfo::Texto => TypeInfo::Texto,
                    _ if is_numeric(&lt) && is_numeric(&rt) => {
                        if lt == TypeInfo::Entero && rt == TypeInfo::Entero {
                            TypeInfo::Entero
                        } else {
                            TypeInfo::Decimal
                        }
                    }
                    _ => TypeInfo::Void,
                }
            }
            _ => TypeInfo::Void,
        }
    }

    fn op_to_method_name(op: &BinOp) -> Option<&'static str> {
        match op {
            BinOp::Add => Some("sumar"),
            BinOp::Sub => Some("restar"),
            BinOp::Mul => Some("multiplicar"),
            BinOp::Div => Some("dividir"),
            BinOp::Mod => Some("modulo"),
            BinOp::Equal => Some("igual"),
            BinOp::NotEqual => Some("diferente"),
            BinOp::Less => Some("menor"),
            BinOp::LessEqual => Some("menor_o_igual"),
            BinOp::Greater => Some("mayor"),
            BinOp::GreaterEqual => Some("mayor_o_igual"),
            BinOp::And
            | BinOp::Or
            | BinOp::BitOr
            | BinOp::BitAnd
            | BinOp::BitXor
            | BinOp::ShiftLeft
            | BinOp::ShiftRight
            | BinOp::Concat => None,
        }
    }

    fn find_op_method(&self, type_name: &str, method: &str) -> Option<String> {
        for (impl_type, trait_name) in self.impls.keys() {
            if impl_type != type_name {
                continue;
            }
            if let Some((methods, _)) = self.traits.get(trait_name) {
                for (tm, _, _) in methods {
                    if tm == method {
                        return Some(format!("{}_{}_{}", type_name, trait_name, method));
                    }
                }
            }
        }
        None
    }

    fn collect_functions(&mut self, program: &Program) {
        for node in program {
            if let DeclOrStmt::Decl(Decl::Function {
                return_type,
                name,
                params,
                type_params,
                type_param_bounds,
                ..
            }) = node
            {
                let ret = self.resolve_type(return_type.clone(), type_params);
                let params_t: Vec<TypeInfo> = params
                    .iter()
                    .map(|p| self.resolve_type(p.param_type.clone(), type_params))
                    .collect();
                let default_count = params.iter().filter(|p| p.default.is_some()).count();
                self.functions.insert(
                    name.clone(),
                    (ret, params_t, default_count, type_params.clone()),
                );
                if !type_param_bounds.is_empty() {
                    self.type_param_bounds
                        .insert(name.clone(), type_param_bounds.clone());
                }
            }
        }
    }

    fn collect_traits(&mut self, program: &Program) {
        for node in program {
            if let DeclOrStmt::Decl(Decl::Rasgo {
                name,
                methods,
                associated_types,
                ..
            }) = node
            {
                let sigs: Vec<(String, Vec<TypeInfo>, TypeInfo)> = methods
                    .iter()
                    .map(|m| {
                        let param_types: Vec<TypeInfo> = m
                            .params
                            .iter()
                            .map(|p| self.type_to_info(p.param_type.clone()))
                            .collect();
                        let ret = self.type_to_info(m.return_type.clone());
                        (m.name.clone(), param_types, ret)
                    })
                    .collect();
                let assoc_types: Vec<AssociatedTypeDef> = associated_types
                    .iter()
                    .map(|at| AssociatedTypeDef {
                        name: at.name.clone(),
                        default: at.default.as_ref().map(|t| self.type_to_info(t.clone())),
                    })
                    .collect();
                self.traits.insert(name.clone(), (sigs, assoc_types));
            }
        }
    }

    fn collect_impls(&mut self, program: &Program) {
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
                if !trait_name.is_empty() && !self.traits.contains_key(trait_name) {
                    continue;
                }
                for method_decl in methods {
                    if let Decl::Function {
                        return_type,
                        name,
                        mut params,
                        type_params,
                        ..
                    } = method_decl.clone()
                    {
                        for p in params.iter_mut() {
                            if let Type::Struct(s) = &p.param_type {
                                if s == "Self"
                                    || s == "self"
                                    || s == "este"
                                    || s.ends_with("_Self")
                                    || s.ends_with("_self")
                                    || s.ends_with("_este")
                                {
                                    p.param_type = target_type.clone();
                                }
                            }
                        }
                        let ret = self.resolve_type(return_type.clone(), &type_params);
                        let params_t: Vec<TypeInfo> = params
                            .iter()
                            .map(|p| self.resolve_type(p.param_type.clone(), &type_params))
                            .collect();
                        let default_count = params.iter().filter(|p| p.default.is_some()).count();
                        let mangled = if trait_name.is_empty() {
                            format!("{}_{}", type_name, name)
                        } else {
                            format!("{}_{}_{}", type_name, trait_name, name)
                        };
                        self.impls
                            .entry((type_name.clone(), trait_name.clone()))
                            .or_default()
                            .push(self.impl_methods.len());
                        self.impl_methods.push((
                            mangled.clone(),
                            (
                                ret.clone(),
                                params_t.clone(),
                                default_count,
                                type_params.clone(),
                            ),
                        ));
                        self.functions
                            .insert(mangled, (ret, params_t, default_count, type_params.clone()));
                    }
                }
            }
        }
    }

    fn collect_enums(&mut self, program: &Program) {
        for node in program {
            if let DeclOrStmt::Decl(Decl::Enum { name, variants, .. }) = node {
                let enum_variants: Vec<(String, Vec<TypeInfo>)> = variants
                    .iter()
                    .map(|v| {
                        let types: Vec<TypeInfo> = v
                            .types
                            .iter()
                            .map(|t| self.type_to_info(t.clone()))
                            .collect();
                        (v.name.clone(), types)
                    })
                    .collect();
                self.enums.insert(name.clone(), enum_variants);
            }
        }
    }

    fn collect_structs(&mut self, program: &Program) {
        // Pass 1: pre-registrar nombres para que referencias recursivas
        // (opcion<Self>) encuentren el nombre ya registrado (fix bug #4)
        for node in program {
            if let DeclOrStmt::Decl(Decl::Struct {
                name, type_params, ..
            }) = node
            {
                self.structs
                    .entry(name.clone())
                    .or_insert_with(|| (Vec::new(), type_params.clone()));
            }
        }
        // Pass 2: resolver campos con los nombres ya registrados
        for node in program {
            if let DeclOrStmt::Decl(Decl::Struct {
                name,
                fields,
                type_params,
                type_param_bounds,
                ..
            }) = node
            {
                let struct_fields: Vec<(String, TypeInfo)> = fields
                    .iter()
                    .map(|f| {
                        (
                            f.name.clone(),
                            self.resolve_type(f.field_type.clone(), type_params),
                        )
                    })
                    .collect();
                self.structs
                    .insert(name.clone(), (struct_fields, type_params.clone()));
                if !type_param_bounds.is_empty() {
                    self.type_param_bounds
                        .insert(name.clone(), type_param_bounds.clone());
                }
            }
        }
    }

    fn analyze_program(&mut self, program: &Program) {
        for node in program {
            self.analyze_decl_or_stmt(node);
        }
    }

    fn analyze_decl_or_stmt(&mut self, node: &DeclOrStmt) -> TypeInfo {
        match node {
            DeclOrStmt::Decl(d) => self.analyze_decl(d),
            DeclOrStmt::Stmt(s) => self.analyze_stmt(s),
        }
    }

    fn analyze_decl(&mut self, decl: &Decl) -> TypeInfo {
        match decl {
            Decl::Variable {
                var_type,
                name,
                init,
                span,
            } => {
                let inferred_type = init
                    .as_ref()
                    .map(|e| self.analyze_expr(e))
                    .unwrap_or_else(|| self.type_to_info(var_type.clone()));
                let declared_type = if matches!(var_type, Type::Struct(name) if name == "Infer") {
                    inferred_type.clone()
                } else {
                    self.type_to_info(var_type.clone())
                };
                if init.is_some() {
                    let init_type = inferred_type;
                    if !can_assign(&declared_type, &init_type) {
                        self.errors.push(SemError {
                            code: "E031".to_string(),
                            message: format!("No puedes asignar un valor de tipo '{:?}' a una variable de tipo '{:?}'", init_type, declared_type),
                            span: *span,
                            suggestion: format!("Usa un valor de tipo '{:?}' en lugar de '{:?}'", declared_type, init_type),
                        });
                    }
                }
                if let Err(e) = self
                    .current_scope()
                    .define(name, declared_type.clone(), *span)
                {
                    self.errors.push(e);
                }
                declared_type
            }
            Decl::Destructure {
                targets,
                init,
                span,
            } => {
                let init_type = self.analyze_expr(init);
                let tuple_types = match &init_type {
                    TypeInfo::Tuple(types) => types.clone(),
                    _ => {
                        self.errors.push(SemError {
                            code: "E068".to_string(),
                            message: format!(
                                "La destructuración requiere una tupla, no '{:?}'",
                                init_type
                            ),
                            span: *span,
                            suggestion: "Usa una expresión de tipo tupla en el lado derecho"
                                .to_string(),
                        });
                        return TypeInfo::Void;
                    }
                };
                if targets.len() != tuple_types.len() {
                    self.errors.push(SemError {
                        code: "E069".to_string(),
                        message: format!("La destructuración espera {} variables pero la tupla tiene {} elementos", targets.len(), tuple_types.len()),
                        span: *span,
                        suggestion: format!("Usa {} variables en la destructuración", tuple_types.len()),
                    });
                    return TypeInfo::Void;
                }
                for (i, target) in targets.iter().enumerate() {
                    if target.name == "_" {
                        continue;
                    }
                    if let Some(ref t_type) = target.var_type {
                        let declared_type = self.type_to_info(t_type.clone());
                        let element_type = &tuple_types[i];
                        if !can_assign(&declared_type, element_type) {
                            self.errors.push(SemError {
                                code: "E031".to_string(),
                                message: format!("No puedes asignar un valor de tipo '{:?}' a la variable '{}' de tipo '{:?}'", element_type, target.name, declared_type),
                                span: target.span,
                                suggestion: format!("Usa un tipo '{:?}' para la variable '{}'", element_type, target.name),
                            });
                        }
                        if let Err(e) =
                            self.current_scope()
                                .define(&target.name, declared_type, target.span)
                        {
                            self.errors.push(e);
                        }
                    } else {
                        let element_type = tuple_types[i].clone();
                        if let Err(e) =
                            self.current_scope()
                                .define(&target.name, element_type, target.span)
                        {
                            self.errors.push(e);
                        }
                    }
                }
                TypeInfo::Void
            }
            Decl::Function {
                return_type,
                name: _,
                params,
                body,
                type_params,
                type_param_bounds: _,
                is_async: _,
                span: _,
            } => {
                self.scopes.push(Scope::new());
                for tp in type_params {
                    if let Err(e) = self.current_scope().define(
                        tp,
                        TypeInfo::TypeVar(tp.clone()),
                        Span::new(Pos::new(0, 0), Pos::new(0, 0)),
                    ) {
                        self.errors.push(e);
                    }
                }
                let mut seen_default = false;
                for p in params {
                    if p.default.is_some() {
                        seen_default = true;
                    } else if seen_default {
                        self.errors.push(SemError {
                            code: "E057".to_string(),
                            message: format!("El parámetro '{}' no tiene valor por defecto pero aparece después de un parámetro con defecto", p.name),
                            span: p.span,
                            suggestion: "Mueve este parámetro antes de los parámetros con valor por defecto".to_string(),
                        });
                    }
                    let pt = self.resolve_type(p.param_type.clone(), type_params);
                    if let Err(e) = self.current_scope().define(&p.name, pt, p.span) {
                        self.errors.push(e);
                    }
                }
                for node in body {
                    let _ret = self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();
                self.resolve_type(return_type.clone(), type_params)
            }
            Decl::Struct {
                name,
                fields,
                type_params,
                type_param_bounds: _,
                span: _,
            } => {
                let struct_fields: Vec<(String, TypeInfo)> = fields
                    .iter()
                    .map(|f| {
                        (
                            f.name.clone(),
                            self.resolve_type(f.field_type.clone(), type_params),
                        )
                    })
                    .collect();
                TypeInfo::Struct {
                    name: name.clone(),
                    fields: struct_fields,
                }
            }
            Decl::Enum {
                name,
                variants: _,
                span: _,
            } => TypeInfo::Enum(name.clone()),
            Decl::Const {
                var_type,
                name,
                value,
                span,
            } => {
                let declared_type = self.type_to_info(var_type.clone());
                let value_type = self.analyze_expr(value);
                if !can_assign(&declared_type, &value_type) {
                    self.errors.push(SemError {
                        code: "E031".to_string(),
                        message: format!("No puedes asignar un valor de tipo '{:?}' a una constante de tipo '{:?}'", value_type, declared_type),
                        span: *span,
                        suggestion: format!("Usa un valor de tipo '{:?}' en lugar de '{:?}'", declared_type, value_type),
                    });
                }
                if let Err(e) = self
                    .current_scope()
                    .define(name, declared_type.clone(), *span)
                {
                    self.errors.push(e);
                }
                declared_type
            }
            Decl::Rasgo { .. } => TypeInfo::Void,
            Decl::ImplRasgo {
                trait_name,
                target_type,
                associated_types,
                methods,
                span,
            } => {
                let type_name = match type_to_impl_name(target_type) {
                    Some(n) => n,
                    None => {
                        self.errors.push(SemError {
                            code: "E074".to_string(),
                            message: format!(
                                "No se puede implementar para el tipo '{:?}'",
                                target_type
                            ),
                            span: *span,
                            suggestion: "Este tipo no soporta implementación de métodos"
                                .to_string(),
                        });
                        return TypeInfo::Void;
                    }
                };
                if !trait_name.is_empty() {
                    let key = (type_name.clone(), trait_name.clone());
                    if self.seen_impls.contains(&key) {
                        self.errors.push(SemError {
                            code: "E085".to_string(),
                            message: format!(
                                "Implementación duplicada: el rasgo '{}' ya está implementado para el tipo '{}'",
                                trait_name, type_name
                            ),
                            span: *span,
                            suggestion: format!(
                                "Elimina la implementación redundante de '{}' para '{}'",
                                trait_name, type_name
                            ),
                        });
                        return TypeInfo::Void;
                    }
                    self.seen_impls.insert(key);
                }
                if trait_name.is_empty() {
                    for method in methods {
                        let mut m = method.clone();
                        if let Decl::Function { params, .. } = &mut m {
                            for p in params.iter_mut() {
                                if let Type::Struct(s) = &p.param_type {
                                    if s == "Self"
                                        || s == "self"
                                        || s == "este"
                                        || s.ends_with("_Self")
                                        || s.ends_with("_self")
                                        || s.ends_with("_este")
                                    {
                                        p.param_type = target_type.clone();
                                    }
                                }
                            }
                        }
                        self.analyze_decl(&m);
                    }
                    return TypeInfo::Void;
                }
                if !self.traits.contains_key(trait_name) {
                    self.errors.push(SemError {
                        code: "E075".to_string(),
                        message: format!("El rasgo '{}' no está definido", trait_name),
                        span: *span,
                        suggestion: format!("Define '{}' antes de implementarlo", trait_name),
                    });
                    return TypeInfo::Void;
                }
                let trait_sig = self.traits[trait_name].clone();
                let (methods_sig, assoc_types) = trait_sig;
                let mut assoc_subst = HashMap::new();
                for assoc in assoc_types {
                    if let Some(default) = assoc.default {
                        assoc_subst.insert(assoc.name, default);
                    }
                }
                for assoc in associated_types {
                    assoc_subst.insert(
                        assoc.name.clone(),
                        self.type_to_info(assoc.target_type.clone()),
                    );
                }
                for (t_mname, t_params, t_ret) in &methods_sig {
                    let expected_params: Vec<TypeInfo> = t_params
                        .iter()
                        .map(|param| substitute_typevars(param, &assoc_subst))
                        .collect();
                    let expected_ret = substitute_typevars(t_ret, &assoc_subst);
                    let found = methods.iter().any(|m| {
                        if let Decl::Function {
                            name,
                            params,
                            return_type,
                            ..
                        } = m
                        {
                            if name != t_mname {
                                return false;
                            }
                            let m_params: Vec<TypeInfo> = params
                                .iter()
                                .map(|p| self.type_to_info(p.param_type.clone()))
                                .collect();
                            let m_ret = self.type_to_info(return_type.clone());
                            m_params.len() == expected_params.len()
                                && m_params
                                    .iter()
                                    .zip(expected_params.iter())
                                    .all(|(a, b)| can_assign(b, a) || can_assign(a, b))
                                && (can_assign(&expected_ret, &m_ret)
                                    || can_assign(&m_ret, &expected_ret))
                        } else {
                            false
                        }
                    });
                    if !found {
                        self.errors.push(SemError {
                            code: "E076".to_string(),
                            message: format!(
                                "Falta implementar el método '{}' del rasgo '{}'",
                                t_mname, trait_name
                            ),
                            span: *span,
                            suggestion: format!(
                                "Agrega la función '{}' en el bloque impl",
                                t_mname
                            ),
                        });
                    }
                }
                TypeInfo::Void
            }
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> TypeInfo {
        match stmt {
            Stmt::Assignment { name, value, span } => {
                let value_type = self.analyze_expr(value);
                if let Some(sym) = self.lookup(name) {
                    // prestado mut (bug #6): asignar a través de la referencia
                    // se valida contra el tipo interior, no contra Prestado.
                    let target = match &sym.var_type {
                        TypeInfo::Prestado { inner, .. } => (**inner).clone(),
                        other => other.clone(),
                    };
                    if !can_assign(&target, &value_type) {
                        self.errors.push(SemError {
                            code: "E031".to_string(),
                            message: format!("No puedes asignar un valor de tipo '{:?}' a la variable '{}' de tipo '{:?}'", value_type, name, target),
                            span: *span,
                            suggestion: format!("Usa un valor de tipo '{:?}' para asignar a '{}'", target, name),
                        });
                    }
                } else if matches!(value_type, TypeInfo::Func { .. }) {
                    if let Err(e) = self.current_scope().define(name, value_type.clone(), *span) {
                        self.errors.push(e);
                    }
                } else {
                    self.errors.push(SemError {
                        code: "E033".to_string(),
                        message: format!("La variable '{}' no está declarada", name),
                        span: *span,
                        suggestion: format!("Declara '{}' antes de usarla", name),
                    });
                }
                value_type
            }
            Stmt::IfLet {
                pattern,
                value,
                then_body,
                else_body,
                span,
            } => {
                self.analyze_expr(value);
                self.scopes.push(Scope::new());
                self.bind_pattern_vars(pattern, *span);
                for n in then_body {
                    self.analyze_decl_or_stmt(n);
                }
                self.scopes.pop();
                if let Some(eb) = else_body {
                    self.scopes.push(Scope::new());
                    for n in eb {
                        self.analyze_decl_or_stmt(n);
                    }
                    self.scopes.pop();
                }
                TypeInfo::Void
            }
            Stmt::GuardLet {
                pattern,
                value,
                else_body,
                span,
            } => {
                self.analyze_expr(value);
                self.bind_pattern_vars(pattern, *span);
                self.scopes.push(Scope::new());
                for n in else_body {
                    self.analyze_decl_or_stmt(n);
                }
                self.scopes.pop();
                TypeInfo::Void
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let cond_type = self.analyze_expr(condition);
                if cond_type != TypeInfo::Booleano {
                    self.errors.push(SemError {
                        code: "E034".to_string(),
                        message: format!(
                            "La condición del 'si' debe ser booleano, no '{:?}'",
                            cond_type
                        ),
                        span: condition.span(),
                        suggestion: "Usa una expresión booleana como condición".to_string(),
                    });
                }
                self.scopes.push(Scope::new());
                for node in then_body {
                    self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();
                if let Some(else_body) = else_body {
                    self.scopes.push(Scope::new());
                    for node in else_body {
                        self.analyze_decl_or_stmt(node);
                    }
                    self.scopes.pop();
                }
                TypeInfo::Void
            }
            Stmt::While {
                condition, body, ..
            } => {
                let cond_type = self.analyze_expr(condition);
                if cond_type != TypeInfo::Booleano {
                    self.errors.push(SemError {
                        code: "E034".to_string(),
                        message: format!(
                            "La condición del 'mientras' debe ser booleano, no '{:?}'",
                            cond_type
                        ),
                        span: condition.span(),
                        suggestion: "Usa una expresión booleana como condición".to_string(),
                    });
                }
                self.loop_depth += 1;
                self.scopes.push(Scope::new());
                for node in body {
                    self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();
                self.loop_depth -= 1;
                TypeInfo::Void
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
                ..
            } => {
                self.scopes.push(Scope::new());
                self.analyze_decl(init);
                let cond_type = self.analyze_expr(condition);
                if cond_type != TypeInfo::Booleano {
                    self.errors.push(SemError {
                        code: "E034".to_string(),
                        message: format!(
                            "La condición del 'para' debe ser booleano, no '{:?}'",
                            cond_type
                        ),
                        span: condition.span(),
                        suggestion: "Usa una expresión booleana como condición".to_string(),
                    });
                }
                self.analyze_stmt(update);
                self.loop_depth += 1;
                self.scopes.push(Scope::new());
                for node in body {
                    self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();
                self.loop_depth -= 1;
                self.scopes.pop();
                TypeInfo::Void
            }
            Stmt::Return { value, .. } => value
                .as_ref()
                .map(|e| self.analyze_expr(e))
                .unwrap_or(TypeInfo::Void),
            Stmt::Break { label: _, span } => {
                if self.loop_depth == 0 {
                    self.errors.push(SemError {
                        code: "E070".to_string(),
                        message: "'romper' fuera de un bucle".to_string(),
                        span: *span,
                        suggestion: "Usa 'romper' solo dentro de 'mientras' o 'para'".to_string(),
                    });
                }
                TypeInfo::Void
            }
            Stmt::Continue { label: _, span } => {
                if self.loop_depth == 0 {
                    self.errors.push(SemError {
                        code: "E055".to_string(),
                        message: "'continuar' solo se puede usar dentro de un ciclo".to_string(),
                        span: *span,
                        suggestion: "Usa 'continuar' dentro de 'mientras' o 'para'".to_string(),
                    });
                }
                TypeInfo::Void
            }
            Stmt::Match {
                expr,
                arms,
                default,
                span,
            } => {
                let expr_type = self.analyze_expr(expr);

                // Exhaustiveness check for enum types
                if default.is_none() {
                    if let TypeInfo::Enum(ref enum_name) = expr_type {
                        if let Some(variants) = self.enums.get(enum_name) {
                            let all_variants: Vec<&String> =
                                variants.iter().map(|(name, _)| name).collect();
                            let mut covered: Vec<&String> = Vec::new();
                            for arm in arms.iter() {
                                if let Expr::EnumCtor { variant, .. } = &arm.value {
                                    covered.push(variant);
                                }
                                for alt in &arm.alt_values {
                                    if let Expr::EnumCtor { variant, .. } = alt {
                                        covered.push(variant);
                                    }
                                }
                            }
                            for var_name in &all_variants {
                                if !covered.contains(var_name) {
                                    self.errors.push(SemError {
                                        code: "E080".to_string(),
                                        message: format!(
                                            "Match no exhaustivo: falta la variante '{}'",
                                            var_name
                                        ),
                                        span: *span,
                                        suggestion: format!(
                                            "Agrega 'caso {}::{}:' o un caso 'defecto'",
                                            enum_name, var_name
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }

                for arm in arms {
                    self.scopes.push(Scope::new());
                    self.bind_pattern_vars(&arm.value, arm.span);
                    let is_range_arm = matches!(&arm.value, Expr::Range { .. });
                    let is_pattern_arm = matches!(
                        &arm.value,
                        Expr::Algun { .. } | Expr::Exito { .. } | Expr::Error { .. }
                    );
                    // QA bug #3: reconocer variantes de enums de usuario como patrones
                    let is_enum_variant_arm = matches!(&arm.value, Expr::EnumCtor { .. })
                        || matches!(&arm.value, Expr::Call { callee, .. }
                            if matches!(**callee, Expr::Ident { .. }));

                    if is_enum_variant_arm {
                        // No analizar como expresión — es un patrón de variante.
                        // Solo verificar que la variante existe en el enum del switch.
                        self.scopes.pop();
                        continue;
                    }

                    let arm_val_type = self.analyze_expr(&arm.value);
                    if !is_range_arm
                        && !is_pattern_arm
                        && arm_val_type != expr_type
                        && !(can_assign(&expr_type, &arm_val_type)
                            || (expr_type == TypeInfo::Decimal && arm_val_type == TypeInfo::Entero))
                    {
                        self.errors.push(SemError {
                            code: "E056".to_string(),
                            message: format!(
                                "El valor del caso debe ser '{:?}', no '{:?}'",
                                expr_type, arm_val_type
                            ),
                            span: arm.span,
                            suggestion: format!(
                                "Usa un valor de tipo '{:?}' en este caso",
                                expr_type
                            ),
                        });
                    }
                    // Validate guard type
                    if let Some(ref guard) = arm.guard {
                        let guard_type = self.analyze_expr(guard);
                        if guard_type != TypeInfo::Booleano {
                            self.errors.push(SemError {
                                code: "E034".to_string(),
                                message: format!(
                                    "La guardia del 'caso' debe ser booleano, no '{:?}'",
                                    guard_type
                                ),
                                span: guard.span(),
                                suggestion: "Usa una expresión booleana como guardia".to_string(),
                            });
                        }
                    }
                    for node in &arm.body {
                        self.analyze_decl_or_stmt(node);
                    }
                    self.scopes.pop();
                }
                if let Some(default_body) = default {
                    self.scopes.push(Scope::new());
                    for node in default_body {
                        self.analyze_decl_or_stmt(node);
                    }
                    self.scopes.pop();
                }
                TypeInfo::Void
            }
            Stmt::FieldAssign {
                expr,
                field,
                value,
                span,
            } => {
                let raw_type = self.analyze_expr(expr);
                let expr_type = match &raw_type {
                    TypeInfo::Prestado { inner, .. } => (**inner).clone(),
                    _ => raw_type.clone(),
                };
                let value_type = self.analyze_expr(value);
                match &expr_type {
                    TypeInfo::Struct { fields, .. } => {
                        let field_type = fields.iter().find(|(name, _)| name == field);
                        match field_type {
                            Some((_, ft)) => {
                                if !can_assign(ft, &value_type) {
                                    self.errors.push(SemError {
                                        code: "E031".to_string(),
                                        message: format!("No puedes asignar un valor de tipo '{:?}' al campo '{}' de tipo '{:?}'", value_type, field, ft),
                                        span: *span,
                                        suggestion: format!("Usa un valor de tipo '{:?}' para el campo '{}'", ft, field),
                                    });
                                }
                            }
                            None => {
                                self.errors.push(SemError {
                                    code: "E059".to_string(),
                                    message: format!(
                                        "El struct no tiene un campo llamado '{}'",
                                        field
                                    ),
                                    span: *span,
                                    suggestion: format!(
                                        "Revisa los campos del struct, '{}' no existe",
                                        field
                                    ),
                                });
                            }
                        }
                    }
                    _ => {
                        self.errors.push(SemError {
                            code: "E060".to_string(),
                            message: format!(
                                "No puedes asignar un campo a un valor de tipo '{:?}'",
                                expr_type
                            ),
                            span: *span,
                            suggestion: "Solo los structs tienen campos asignables".to_string(),
                        });
                    }
                }
                TypeInfo::Void
            }
            Stmt::ArraySet {
                arr,
                index,
                value,
                span,
            } => {
                let raw_arr = self.analyze_expr(arr);
                // Auto-deref Prestado<T> → T para asignación por índice (QA bug #6)
                let arr_type = match &raw_arr {
                    TypeInfo::Prestado { inner, .. } => (**inner).clone(),
                    _ => raw_arr.clone(),
                };
                let _ = self.analyze_expr(index);
                let value_type = self.analyze_expr(value);
                match &arr_type {
                    TypeInfo::Lista(inner) => {
                        if !can_assign(inner, &value_type)
                            && !(**inner == TypeInfo::Numero || value_type == TypeInfo::Numero)
                        {
                            self.errors.push(SemError {
                                code: "E031".to_string(),
                                message: format!(
                                    "No puedes asignar un valor de tipo '{:?}' a un elemento de tipo '{:?}'",
                                    value_type, inner
                                ),
                                span: *span,
                                suggestion: "Usa un valor del mismo tipo que la lista".to_string(),
                            });
                        }
                    }
                    _ => {
                        self.errors.push(SemError {
                            code: "E060".to_string(),
                            message: format!(
                                "Solo puedes asignar por índice a listas, no a '{:?}'",
                                arr_type
                            ),
                            span: *span,
                            suggestion: "Usa una lista como destino de la asignación".to_string(),
                        });
                    }
                }
                TypeInfo::Void
            }
            Stmt::Expr { expr, .. } => self.analyze_expr(expr),
            Stmt::Posponer { body, .. } => {
                self.scopes.push(Scope::new());
                for node in body {
                    self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();
                TypeInfo::Void
            }
            Stmt::TryCatch {
                try_body,
                err_var,
                catch_body,
                span,
            } => {
                self.scopes.push(Scope::new());
                for node in try_body {
                    self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();

                self.scopes.push(Scope::new());
                if let Err(e) = self.current_scope().define(err_var, TypeInfo::Texto, *span) {
                    self.errors.push(e);
                }
                for node in catch_body {
                    self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();
                TypeInfo::Void
            }
            Stmt::Block { stmts, .. } => {
                self.scopes.push(Scope::new());
                for node in stmts {
                    self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();
                TypeInfo::Void
            }
            Stmt::ForEach {
                var_name,
                expr,
                body,
                span,
            } => {
                let expr_type = self.analyze_expr(expr);
                let item_type = match &expr_type {
                    TypeInfo::Lista(inner) => *inner.clone(),
                    _ => {
                        self.errors.push(SemError {
                            code: "E044".to_string(),
                            message: format!(
                                "'para-cada' requiere una lista, no '{:?}'",
                                expr_type
                            ),
                            span: *span,
                            suggestion: "Usa una lista en el ciclo 'para-cada'".to_string(),
                        });
                        TypeInfo::Void
                    }
                };
                self.scopes.push(Scope::new());
                if let Err(e) = self.current_scope().define(var_name, item_type, *span) {
                    self.errors.push(e);
                }
                for node in body {
                    self.analyze_decl_or_stmt(node);
                }
                self.scopes.pop();
                TypeInfo::Void
            }
            Stmt::Import { .. } => TypeInfo::Void,
            Stmt::Destructure {
                targets,
                value,
                span,
            } => {
                let value_type = self.analyze_expr(value);
                let tuple_types = match &value_type {
                    TypeInfo::Tuple(types) => types.clone(),
                    _ => {
                        self.errors.push(SemError {
                            code: "E068".to_string(),
                            message: format!(
                                "La destructuración requiere una tupla, no '{:?}'",
                                value_type
                            ),
                            span: *span,
                            suggestion: "Usa una expresión de tipo tupla en el lado derecho"
                                .to_string(),
                        });
                        return TypeInfo::Void;
                    }
                };
                if targets.len() != tuple_types.len() {
                    self.errors.push(SemError {
                        code: "E069".to_string(),
                        message: format!("La destructuración espera {} variables pero la tupla tiene {} elementos", targets.len(), tuple_types.len()),
                        span: *span,
                        suggestion: format!("Usa {} variables en la destructuración", tuple_types.len()),
                    });
                    return TypeInfo::Void;
                }
                for (i, target) in targets.iter().enumerate() {
                    if target.name == "_" {
                        continue;
                    }
                    let element_type = &tuple_types[i];
                    if let Some(sym) = self.lookup(&target.name) {
                        if !can_assign(&sym.var_type, element_type) {
                            self.errors.push(SemError {
                                code: "E031".to_string(),
                                message: format!("No puedes asignar un valor de tipo '{:?}' a la variable '{}' de tipo '{:?}'", element_type, target.name, sym.var_type),
                                span: target.span,
                                suggestion: format!("Usa un valor de tipo '{:?}' para '{}'", sym.var_type, target.name),
                            });
                        }
                    } else {
                        self.errors.push(SemError {
                            code: "E033".to_string(),
                            message: format!("La variable '{}' no está declarada", target.name),
                            span: target.span,
                            suggestion: format!("Declara '{}' antes de usarla", target.name),
                        });
                    }
                }
                TypeInfo::Void
            }
            Stmt::InlineAsm { .. } => TypeInfo::Void,
            Stmt::InlineC { .. } => TypeInfo::Void,
            Stmt::InlineRust { .. } => TypeInfo::Void,
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) -> TypeInfo {
        match expr {
            Expr::Int { .. } => TypeInfo::Entero,
            Expr::Float { .. } => TypeInfo::Decimal,
            Expr::Str { .. } => TypeInfo::Texto,
            Expr::Bool { .. } => TypeInfo::Booleano,
            Expr::Ident { name, span } => match self.lookup(name) {
                Some(sym) => sym.var_type.clone(),
                None => {
                    self.errors.push(SemError {
                        code: "E033".to_string(),
                        message: format!("La variable '{}' no está declarada", name),
                        span: *span,
                        suggestion: format!("Declara '{}' antes de usarla", name),
                    });
                    TypeInfo::Numero
                }
            },
            Expr::Binary {
                op,
                left,
                right,
                resolved_method: _,
                span,
            } => {
                // prestado mut (bug #6): los operadores ven el tipo interior
                let lt = match self.analyze_expr(left) {
                    TypeInfo::Prestado { inner, .. } => *inner,
                    other => other,
                };
                let rt = match self.analyze_expr(right) {
                    TypeInfo::Prestado { inner, .. } => *inner,
                    other => other,
                };
                let op_method_name = |op: &BinOp| -> &str {
                    match op {
                        BinOp::Add => "sumar",
                        BinOp::Sub => "restar",
                        BinOp::Mul => "multiplicar",
                        BinOp::Div => "dividir",
                        BinOp::Mod => "modulo",
                        BinOp::Equal => "igual",
                        BinOp::NotEqual => "diferente",
                        BinOp::Less => "menor",
                        BinOp::LessEqual => "menor_o_igual",
                        BinOp::Greater => "mayor",
                        BinOp::GreaterEqual => "mayor_o_igual",
                        BinOp::And
                        | BinOp::Or
                        | BinOp::BitOr
                        | BinOp::BitAnd
                        | BinOp::BitXor
                        | BinOp::ShiftLeft
                        | BinOp::ShiftRight
                        | BinOp::Concat => "",
                    }
                };
                let has_op_overload = |t: &TypeInfo, op: &BinOp| -> bool {
                    let mname = op_method_name(op);
                    if mname.is_empty() {
                        return false;
                    }
                    let type_name = match t {
                        TypeInfo::Struct { name, .. } => name.clone(),
                        _ => return false,
                    };
                    for (impl_type, trait_name) in self.impls.keys() {
                        if impl_type != &type_name {
                            continue;
                        }
                        if let Some((methods, _)) = self.traits.get(trait_name) {
                            for (tm, _, _) in methods {
                                if tm == mname {
                                    return true;
                                }
                            }
                        }
                    }
                    false
                };
                match op {
                    BinOp::Concat => {
                        if lt == TypeInfo::Texto && rt == TypeInfo::Texto {
                            TypeInfo::Texto
                        } else if let (TypeInfo::Lista(li), TypeInfo::Lista(ri)) = (&lt, &rt) {
                            if li == ri || **li == TypeInfo::Numero || **ri == TypeInfo::Numero {
                                TypeInfo::Lista(Box::new(if **li == TypeInfo::Numero {
                                    ri.as_ref().clone()
                                } else {
                                    li.as_ref().clone()
                                }))
                            } else {
                                self.errors.push(SemError {
                                    code: "E035".to_string(),
                                    message: format!(
                                        "No se puede concatenar listas de tipos diferentes: {:?} y {:?}",
                                        lt, rt
                                    ),
                                    span: *span,
                                    suggestion: "Usa el mismo tipo de elemento en ambas listas"
                                        .to_string(),
                                });
                                TypeInfo::Void
                            }
                        } else {
                            self.errors.push(SemError {
                                code: "E035".to_string(),
                                message: format!(
                                    "El operador ++ requiere texto o listas: {:?} y {:?}",
                                    lt, rt
                                ),
                                span: *span,
                                suggestion: "Usa texto o listas con el mismo tipo de elemento"
                                    .to_string(),
                            });
                            TypeInfo::Void
                        }
                    }
                    BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::Mod
                    | BinOp::BitOr
                    | BinOp::BitAnd
                    | BinOp::BitXor
                    | BinOp::ShiftLeft
                    | BinOp::ShiftRight => {
                        if matches!(op, BinOp::Add)
                            && ((lt == TypeInfo::Texto
                                && (rt == TypeInfo::Texto
                                    || is_numeric(&rt)
                                    || rt == TypeInfo::Booleano))
                                || (rt == TypeInfo::Texto
                                    && (is_numeric(&lt) || lt == TypeInfo::Booleano)))
                        {
                            TypeInfo::Texto
                        } else if lt == TypeInfo::Entero && rt == TypeInfo::Entero {
                            TypeInfo::Entero
                        } else if is_numeric(&lt) && is_numeric(&rt) {
                            TypeInfo::Decimal
                        } else if has_op_overload(&lt, op) {
                            // Operator overload via trait method — return method's return type
                            // We infer the return type from the trait signature
                            let mname = op_method_name(op);
                            let type_name = match &lt {
                                TypeInfo::Struct { name, .. } => name.clone(),
                                _ => String::new(),
                            };
                            let mut ret = TypeInfo::Void;
                            for (impl_type, trait_name) in self.impls.keys() {
                                if impl_type != &type_name {
                                    continue;
                                }
                                if let Some((methods, _)) = self.traits.get(trait_name) {
                                    for (tm, _params, tr) in methods {
                                        if tm == mname {
                                            ret = tr.clone();
                                        }
                                    }
                                }
                            }
                            ret
                        } else {
                            self.errors.push(SemError {
                                code: "E035".to_string(),
                                message: format!("Operador aritmético requiere números, no '{:?}' y '{:?}'", lt, rt),
                                span: *span,
                                suggestion: "Ambos operandos deben ser numéricos o usar '+' para concatenar textos".to_string(),
                            });
                            TypeInfo::Decimal
                        }
                    }
                    BinOp::Equal | BinOp::NotEqual => {
                        if lt == rt
                            || (is_numeric(&lt) && is_numeric(&rt))
                            || can_assign(&lt, &rt)
                            || can_assign(&rt, &lt)
                            || has_op_overload(&lt, op)
                        {
                            TypeInfo::Booleano
                        } else {
                            self.errors.push(SemError {
                                code: "E036".to_string(),
                                message: format!("No puedes comparar '{:?}' con '{:?}'", lt, rt),
                                span: *span,
                                suggestion: "Ambos operandos deben ser del mismo tipo".to_string(),
                            });
                            TypeInfo::Booleano
                        }
                    }
                    BinOp::Less | BinOp::LessEqual | BinOp::Greater | BinOp::GreaterEqual => {
                        if is_numeric(&lt)
                            || is_numeric(&rt)
                            || lt == rt
                            || has_op_overload(&lt, op)
                        {
                            TypeInfo::Booleano
                        } else {
                            self.errors.push(SemError {
                                code: "E035".to_string(),
                                message: format!(
                                    "Comparación requiere números, no '{:?}' y '{:?}'",
                                    lt, rt
                                ),
                                span: *span,
                                suggestion: "Ambos operandos deben ser numéricos".to_string(),
                            });
                            TypeInfo::Booleano
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        // Truthiness dinámica (paridad con el VM): cualquier valor
                        // es válido; el cortocircuito evalúa is_truthy en runtime.
                        TypeInfo::Booleano
                    }
                }
            }
            Expr::Unary { op, operand, span } => {
                let ot = self.analyze_expr(operand);
                match op {
                    UnOp::Negate => {
                        if !is_numeric(&ot) {
                            self.errors.push(SemError {
                                code: "E038".to_string(),
                                message: format!("No puedes negar un valor de tipo '{:?}'", ot),
                                span: *span,
                                suggestion: "La negación solo aplica a números".to_string(),
                            });
                        }
                        ot
                    }
                    UnOp::Not => {
                        if ot != TypeInfo::Booleano {
                            self.errors.push(SemError {
                                code: "E039".to_string(),
                                message: format!(
                                    "No puedes aplicar '!' a un valor de tipo '{:?}'",
                                    ot
                                ),
                                span: *span,
                                suggestion: "El operador '!' solo aplica a booleanos".to_string(),
                            });
                        }
                        TypeInfo::Booleano
                    }
                    UnOp::BitNot => {
                        if !is_numeric(&ot) {
                            self.errors.push(SemError {
                                code: "E038".to_string(),
                                message: format!(
                                    "No puedes aplicar '~' a un valor de tipo '{:?}'",
                                    ot
                                ),
                                span: *span,
                                suggestion: "El operador '~' solo aplica a números".to_string(),
                            });
                        }
                        TypeInfo::Entero
                    }
                }
            }
            Expr::Call {
                callee,
                args,
                type_args,
                span,
            } => {
                let callee_inner = match callee.as_ref() {
                    Expr::Grouping { expr, .. } => expr.as_ref(),
                    Expr::Cast { expr, .. } => expr.as_ref(),
                    other => other,
                };
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.analyze_expr(arg));
                }
                match callee_inner {
                    Expr::Ident { name, .. } => {
                        let callee = name.clone();
                        let func_info = self.functions.get(&callee).cloned();
                        match func_info {
                            Some((ret_type, param_types, default_count, fn_type_params)) => {
                                // Build substitution map if type_args provided
                                let subst = if !type_args.is_empty() && !fn_type_params.is_empty() {
                                    let mut map = HashMap::new();
                                    for (tp, ta) in fn_type_params.iter().zip(type_args.iter()) {
                                        map.insert(tp.clone(), self.type_to_info(ta.clone()));
                                    }
                                    // Validate type bounds
                                    if let Some(bounds) = self.type_param_bounds.get(&callee) {
                                        for (tp, bound_trait) in bounds {
                                            if let Some(concrete) = map.get(tp) {
                                                let concrete_name = match concrete {
                                                    TypeInfo::Struct { name, .. } => name.clone(),
                                                    TypeInfo::Enum(name) => name.clone(),
                                                    _ => continue,
                                                };
                                                let key =
                                                    (concrete_name.clone(), bound_trait.clone());
                                                if !self.impls.contains_key(&key) {
                                                    self.errors.push(SemError {
                                                        code: "E077".to_string(),
                                                        message: format!(
                                                            "El tipo '{}' no implementa el rasgo '{}' requerido por '{}'",
                                                            concrete_name, bound_trait, callee
                                                        ),
                                                        span: *span,
                                                        suggestion: format!(
                                                            "Implementa el rasgo '{}' para '{}'",
                                                            bound_trait, concrete_name
                                                        ),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    Some(map)
                                } else {
                                    None
                                };
                                // Substitute types
                                let subst_param_types: Vec<TypeInfo> = if let Some(ref s) = subst {
                                    param_types
                                        .iter()
                                        .map(|pt| substitute_typevars(pt, s))
                                        .collect()
                                } else {
                                    param_types.clone()
                                };
                                let subst_ret_type = if let Some(ref s) = subst {
                                    substitute_typevars(&ret_type, s)
                                } else {
                                    ret_type.clone()
                                };
                                let min_args = subst_param_types.len() - default_count;
                                if args.len() < min_args {
                                    self.errors.push(SemError {
                                        code: "E040".to_string(),
                                        message: format!("La función '{}' espera al menos {} argumentos, pero se pasaron {}", callee, min_args, args.len()),
                                        span: *span,
                                        suggestion: format!("Pasa al menos {} argumentos a '{}'", min_args, callee),
                                    });
                                    return subst_ret_type;
                                }
                                if args.len() > subst_param_types.len() {
                                    self.errors.push(SemError {
                                        code: "E040".to_string(),
                                        message: format!("La función '{}' espera como máximo {} argumentos, pero se pasaron {}", callee, subst_param_types.len(), args.len()),
                                        span: *span,
                                        suggestion: format!("Pasa como máximo {} argumentos a '{}'", subst_param_types.len(), callee),
                                    });
                                    return subst_ret_type;
                                }
                                for (i, (got, expected)) in
                                    arg_types.iter().zip(subst_param_types.iter()).enumerate()
                                {
                                    if !can_assign(expected, got) {
                                        self.errors.push(SemError {
                                            code: "E041".to_string(),
                                            message: format!("El argumento {} de '{}' debe ser '{:?}', no '{:?}'", i + 1, callee, expected, got),
                                            span: *span,
                                            suggestion: format!("Pasa un valor de tipo '{:?}' en el argumento {}", expected, i + 1),
                                        });
                                    }
                                }
                                subst_ret_type
                            }
                            None => {
                                if callee == "imprimir"
                                    || callee == "print"
                                    || callee == "leer"
                                    || callee == "read"
                                {
                                    TypeInfo::Void
                                } else if callee == "abs" || callee == "absoluto" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento numérico, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 número (entero o decimal)"
                                                .to_string(),
                                        });
                                    }
                                    match arg_types.first() {
                                        Some(TypeInfo::Entero) => TypeInfo::Entero,
                                        Some(TypeInfo::Decimal) => TypeInfo::Decimal,
                                        _ => TypeInfo::Numero,
                                    }
                                } else if callee == "min"
                                    || callee == "minimo"
                                    || callee == "max"
                                    || callee == "maximo"
                                    || callee == "potencia"
                                    || callee == "pow"
                                {
                                    if args.len() != 2 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 2 argumentos numéricos, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 2 números".to_string(),
                                        });
                                    }
                                    if arg_types.iter().all(|t| *t == TypeInfo::Entero) {
                                        TypeInfo::Entero
                                    } else {
                                        TypeInfo::Decimal
                                    }
                                } else if callee == "raiz" || callee == "sqrt" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento numérico, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 número".to_string(),
                                        });
                                    }
                                    TypeInfo::Decimal
                                } else if callee == "piso"
                                    || callee == "floor"
                                    || callee == "techo"
                                    || callee == "ceil"
                                    || callee == "redondear"
                                    || callee == "round"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento numérico, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 número".to_string(),
                                        });
                                    }
                                    TypeInfo::Entero
                                } else if callee == "a_texto"
                                    || callee == "to_texto"
                                    || callee == "__str_from"
                                {
                                    TypeInfo::Texto
                                } else if callee == "__str_len" || callee == "__str_longitud" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento de tipo texto"
                                                .to_string(),
                                        });
                                    }
                                    if let Some(got) = arg_types.first() {
                                        if !can_assign(&TypeInfo::Texto, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!(
                                                    "'{}' espera 'texto', no '{:?}'",
                                                    callee, got
                                                ),
                                                span: *span,
                                                suggestion: "Pasa un valor de tipo texto"
                                                    .to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Entero
                                } else if callee == "__str_upper"
                                    || callee == "__str_mayusculas"
                                    || callee == "__str_lower"
                                    || callee == "__str_minusculas"
                                    || callee == "__str_trim"
                                    || callee == "__str_recortar"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento de tipo texto"
                                                .to_string(),
                                        });
                                    }
                                    if let Some(got) = arg_types.first() {
                                        if !can_assign(&TypeInfo::Texto, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!(
                                                    "'{}' espera 'texto', no '{:?}'",
                                                    callee, got
                                                ),
                                                span: *span,
                                                suggestion: "Pasa un valor de tipo texto"
                                                    .to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Texto
                                } else if callee == "__str_contains" || callee == "__str_contiene" {
                                    if args.len() != 2 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 2 argumentos, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 2 argumentos de tipo texto"
                                                .to_string(),
                                        });
                                    }
                                    for (i, got) in arg_types.iter().enumerate() {
                                        if !can_assign(&TypeInfo::Texto, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!("El argumento {} de '{}' debe ser 'texto', no '{:?}'", i + 1, callee, got),
                                                span: *span,
                                                suggestion: "Pasa valores de tipo texto".to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Booleano
                                } else if callee == "__str_split" || callee == "__str_dividir" {
                                    if args.len() != 2 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 2 argumentos, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 2 argumentos de tipo texto"
                                                .to_string(),
                                        });
                                    }
                                    for (i, got) in arg_types.iter().enumerate() {
                                        if !can_assign(&TypeInfo::Texto, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!("El argumento {} de '{}' debe ser 'texto', no '{:?}'", i + 1, callee, got),
                                                span: *span,
                                                suggestion: "Pasa valores de tipo texto".to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Lista(Box::new(TypeInfo::Texto))
                                } else if callee == "__file_read" || callee == "__leer_archivo" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento de tipo texto (ruta)"
                                                .to_string(),
                                        });
                                    }
                                    if let Some(got) = arg_types.first() {
                                        if !can_assign(&TypeInfo::Texto, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!(
                                                    "'{}' espera 'texto', no '{:?}'",
                                                    callee, got
                                                ),
                                                span: *span,
                                                suggestion: "Pasa una ruta de tipo texto"
                                                    .to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Resultado {
                                        ok: Box::new(TypeInfo::Texto),
                                        err: Box::new(TypeInfo::Texto),
                                    }
                                } else if callee == "__file_write"
                                    || callee == "__escribir_archivo"
                                    || callee == "__file_append"
                                    || callee == "__agregar_archivo"
                                {
                                    if args.len() != 2 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 2 argumentos, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 2 argumentos: ruta y contenido"
                                                .to_string(),
                                        });
                                    }
                                    for (i, got) in arg_types.iter().enumerate() {
                                        if !can_assign(&TypeInfo::Texto, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!("El argumento {} de '{}' debe ser 'texto', no '{:?}'", i + 1, callee, got),
                                                span: *span,
                                                suggestion: "Pasa valores de tipo texto".to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Resultado {
                                        ok: Box::new(TypeInfo::Booleano),
                                        err: Box::new(TypeInfo::Texto),
                                    }
                                } else if callee == "__file_write_binary"
                                    || callee == "__escribir_archivo_bin"
                                {
                                    if args.len() != 2 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 2 argumentos, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 2 argumentos: ruta y Array<Int>"
                                                .to_string(),
                                        });
                                    }
                                    if !args.is_empty()
                                        && !can_assign(&TypeInfo::Texto, &arg_types[0])
                                    {
                                        self.errors.push(SemError {
                                            code: "E041".to_string(),
                                            message: format!("El argumento 1 de '{}' debe ser 'texto', no '{:?}'", callee, arg_types[0]),
                                            span: *span,
                                            suggestion: "Pasa un valor de tipo texto".to_string(),
                                        });
                                    }
                                    if args.len() >= 2
                                        && !can_assign(
                                            &TypeInfo::Lista(Box::new(TypeInfo::Entero)),
                                            &arg_types[1],
                                        )
                                    {
                                        self.errors.push(SemError {
                                            code: "E041".to_string(),
                                            message: format!("El argumento 2 de '{}' debe ser 'Array<Int>', no '{:?}'", callee, arg_types[1]),
                                            span: *span,
                                            suggestion: "Pasa un valor de tipo Array<Int>".to_string(),
                                        });
                                    }
                                    TypeInfo::Resultado {
                                        ok: Box::new(TypeInfo::Booleano),
                                        err: Box::new(TypeInfo::Texto),
                                    }
                                } else if callee == "__num_a_f64_bytes"
                                    || callee == "__numero_a_bytes_f64"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento: número".to_string(),
                                        });
                                    }
                                    TypeInfo::Lista(Box::new(TypeInfo::Entero))
                                } else if callee == "__file_bytes" || callee == "__leer_bytes" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento (ruta), no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento: ruta del archivo"
                                                .to_string(),
                                        });
                                    }
                                    TypeInfo::Lista(Box::new(TypeInfo::Entero))
                                } else if callee == "__a_f64_bytes" || callee == "__bytes_a_f64" {
                                    TypeInfo::Numero
                                } else if callee == "__compile_nv" || callee == "__compilar_nv" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento (ruta), no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento: ruta del archivo .nv"
                                                .to_string(),
                                        });
                                    }
                                    if !args.is_empty()
                                        && !can_assign(&TypeInfo::Texto, &arg_types[0])
                                    {
                                        self.errors.push(SemError {
                                            code: "E041".to_string(),
                                            message: format!("El argumento 1 de '{}' debe ser 'texto', no '{:?}'", callee, arg_types[0]),
                                            span: *span,
                                            suggestion: "Pasa una ruta de tipo texto".to_string(),
                                        });
                                    }
                                    TypeInfo::Lista(Box::new(TypeInfo::Entero))
                                } else if callee == "__codegen_a_nvc" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento: codegen map".to_string(),
                                        });
                                    }
                                    if !args.is_empty()
                                        && !can_assign(&TypeInfo::Numero, &arg_types[0])
                                    {
                                        self.errors.push(SemError {
                                            code: "E041".to_string(),
                                            message: format!("El argumento 1 de '{}' debe ser 'numero', no '{:?}'", callee, arg_types[0]),
                                            span: *span,
                                            suggestion: "Pasa un mapa de codegen".to_string(),
                                        });
                                    }
                                    TypeInfo::Lista(Box::new(TypeInfo::Entero))
                                } else if callee == "__file_exists" || callee == "__existe_archivo"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento de tipo texto"
                                                .to_string(),
                                        });
                                    }
                                    if let Some(got) = arg_types.first() {
                                        if !can_assign(&TypeInfo::Texto, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!(
                                                    "'{}' espera 'texto', no '{:?}'",
                                                    callee, got
                                                ),
                                                span: *span,
                                                suggestion: "Pasa un valor de tipo texto"
                                                    .to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Booleano
                                } else if callee == "__time_now" || callee == "__tiempo_ahora" {
                                    if !args.is_empty() {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' no espera argumentos, pero se pasaron {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "No pases argumentos a esta función"
                                                .to_string(),
                                        });
                                    }
                                    TypeInfo::Entero
                                } else if callee == "__list_reverse"
                                    || callee == "__lista_invertir"
                                    || callee == "__list_sort"
                                    || callee == "__lista_ordenar"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento de tipo lista"
                                                .to_string(),
                                        });
                                    }
                                    if let Some(got) = arg_types.first() {
                                        match got {
                                            TypeInfo::Lista(_) => {}
                                            _ => {
                                                self.errors.push(SemError {
                                                    code: "E041".to_string(),
                                                    message: format!(
                                                        "'{}' espera 'lista', no '{:?}'",
                                                        callee, got
                                                    ),
                                                    span: *span,
                                                    suggestion: "Pasa una lista".to_string(),
                                                });
                                            }
                                        }
                                    }
                                    arg_types
                                        .first()
                                        .cloned()
                                        .unwrap_or(TypeInfo::Lista(Box::new(TypeInfo::Numero)))
                                } else if callee == "__json_parse" || callee == "__json_parsear" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento de tipo texto (JSON)"
                                                .to_string(),
                                        });
                                    }
                                    TypeInfo::Numero
                                } else if callee == "__json_stringify" || callee == "__json_texto" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento".to_string(),
                                        });
                                    }
                                    TypeInfo::Texto
                                } else if callee == "largo" || callee == "len" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 argumento de tipo lista"
                                                .to_string(),
                                        });
                                    }
                                    if let Some(got) = arg_types.first() {
                                        let base_got = match got {
                                            TypeInfo::Prestado { inner, .. }
                                            | TypeInfo::Dueno(inner) => inner.as_ref(),
                                            other => other,
                                        };
                                        match base_got {
                                            TypeInfo::Lista(_)
                                            | TypeInfo::Texto
                                            | TypeInfo::Numero => {}
                                            _ => {
                                                self.errors.push(SemError {
                                                    code: "E041".to_string(),
                                                    message: format!(
                                                        "'{}' espera 'lista', no '{:?}'",
                                                        callee, got
                                                    ),
                                                    span: *span,
                                                    suggestion: "Pasa una lista o texto"
                                                        .to_string(),
                                                });
                                            }
                                        }
                                    }
                                    TypeInfo::Entero
                                } else if callee == "agregar" || callee == "push" {
                                    if args.len() != 2 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 2 argumentos, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa lista y elemento para agregar"
                                                .to_string(),
                                        });
                                    }
                                    TypeInfo::Void
                                } else if callee == "__tarea_lanzar" || callee == "__task_spawn" {
                                    if args.is_empty() {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!("'{}' espera al menos 1 argumento (nombre de función), no {}", callee, args.len()),
                                            span: *span,
                                            suggestion: "Pasa el nombre de la función como primer argumento".to_string(),
                                        });
                                    }
                                    TypeInfo::Texto
                                } else if callee == "__tarea_esperar" || callee == "__task_await" {
                                    TypeInfo::Entero
                                } else if callee == "__js_eval"
                                    || callee == "__js_evaluar"
                                    || callee == "__js_call"
                                    || callee == "__js_llamar"
                                    || callee == "__gui_ventana"
                                    || callee == "__gui_window"
                                    || callee == "__gui_esperar"
                                    || callee == "__gui_poll"
                                {
                                    TypeInfo::Texto
                                } else if callee == "__gui_mostrar"
                                    || callee == "__gui_show"
                                    || callee == "__gui_cerrar"
                                    || callee == "__gui_close"
                                {
                                    TypeInfo::Booleano
                                } else if callee == "__gui_id" || callee == "__gui_hwnd" {
                                    TypeInfo::Entero
                                } else if callee == "__ffi_cargar" || callee == "__ffi_load" {
                                    TypeInfo::Texto
                                } else if callee == "__ffi_llamar"
                                    || callee == "__ffi_call"
                                    || callee == "__ffi_llamar_nv"
                                    || callee == "__ffi_asignar"
                                    || callee == "__ffi_alloc"
                                    || callee == "__ffi_liberar"
                                    || callee == "__ffi_free"
                                    || callee == "__ffi_escribir"
                                    || callee == "__ffi_write"
                                {
                                    TypeInfo::Entero
                                } else if callee == "__ffi_leer" || callee == "__ffi_read" {
                                    TypeInfo::Texto
                                } else if callee == "__ffi_peek"
                                    || callee == "__ffi_peek_u32"
                                    || callee == "__ffi_peek64"
                                    || callee == "__ffi_peek_ptr"
                                    || callee == "__ffi_peek_byte"
                                    || callee == "__ffi_peek_u8"
                                {
                                    TypeInfo::Entero
                                } else if callee == "__ffi_poke"
                                    || callee == "__ffi_poke_u32"
                                    || callee == "__ffi_poke_byte"
                                    || callee == "__ffi_poke_u8"
                                {
                                    TypeInfo::Void
                                } else if callee == "__aes_encriptar"
                                    || callee == "__aes_encrypt"
                                    || callee == "__aes_desencriptar"
                                    || callee == "__aes_decrypt"
                                {
                                    if args.len() < 2 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!("'{}' espera al menos 2 argumentos (key, data), no {}", callee, args.len()),
                                            span: *span,
                                            suggestion: "Pasa la clave y los datos como argumentos".to_string(),
                                        });
                                    }
                                    TypeInfo::Texto
                                } else if callee == "__timezone_info" || callee == "__zona_info" {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento (zona horaria), no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa el nombre de la zona horaria"
                                                .to_string(),
                                        });
                                    }
                                    TypeInfo::Entero
                                } else if callee == "__duration_new" || callee == "__duracion_nueva"
                                {
                                    TypeInfo::Entero
                                } else if callee == "__duration_secs"
                                    || callee == "__duracion_segundos"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento, no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa la duración en nanosegundos"
                                                .to_string(),
                                        });
                                    }
                                    TypeInfo::Entero
                                } else if callee == "__calendar_hijri"
                                    || callee == "__calendario_hijri"
                                    || callee == "__calendar_persian"
                                    || callee == "__calendario_persa"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento (timestamp), no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa un timestamp Unix".to_string(),
                                        });
                                    }
                                    TypeInfo::Texto
                                } else if callee == "__leer_archivo_async"
                                    || callee == "__file_read_async"
                                    || callee == "__escribir_archivo_async"
                                    || callee == "__file_write_async"
                                    || callee == "__timer_delay"
                                    || callee == "__temporizador_esperar"
                                    || callee == "__tcp_connect_async"
                                    || callee == "__tcp_conectar_async"
                                    || callee == "__hilo_esperar"
                                    || callee == "__thread_join"
                                    || callee == "__canal_recibir"
                                    || callee == "__channel_recv"
                                    || callee == "__actor_recibir"
                                    || callee == "__actor_recv"
                                    || callee == "__generador_siguiente"
                                    || callee == "__generator_next"
                                    || callee == "__stream_colectar"
                                    || callee == "__stream_collect"
                                    || callee == "__stream_desde"
                                    || callee == "__stream_from"
                                    || callee == "__stream_mapear"
                                    || callee == "__stream_map"
                                    || callee == "__stream_filtrar"
                                    || callee == "__stream_filter"
                                    || callee == "__par_mapear"
                                    || callee == "__par_map"
                                    || callee == "__par_unir"
                                    || callee == "__par_join"
                                    || callee == "__seleccionar"
                                    || callee == "__select"
                                    || callee == "__mutex_bloquear"
                                    || callee == "__mutex_lock"
                                    || callee == "__hilo_lanzar"
                                    || callee == "__thread_spawn"
                                    || callee == "__canal_nuevo"
                                    || callee == "__channel_new"
                                    || callee == "__mutex_nuevo"
                                    || callee == "__mutex_new"
                                    || callee == "__actor_nuevo"
                                    || callee == "__actor_new"
                                    || callee == "__generador_nuevo"
                                    || callee == "__generator_new"
                                    || callee == "__scope_lanzar"
                                    || callee == "__scope_spawn"
                                    || callee == "__scope_nuevo"
                                    || callee == "__scope_new"
                                    || callee == "__supervisor_nuevo"
                                    || callee == "__supervisor_new"
                                    || callee == "__cluster_conectar"
                                    || callee == "__cluster_connect"
                                    || callee == "__http_servidor"
                                    || callee == "__http_server"
                                    || callee == "__rwlock_nuevo"
                                    || callee == "__rwlock_new"
                                    || callee == "__arc_nuevo"
                                    || callee == "__arc_new"
                                {
                                    TypeInfo::Texto
                                } else if callee == "__tcp_conectar"
                                    || callee == "__tcp_connect"
                                    || callee == "__tcp_escuchar"
                                    || callee == "__tcp_listen"
                                    || callee == "__canal_enviar"
                                    || callee == "__channel_send"
                                    || callee == "__actor_enviar"
                                    || callee == "__actor_send"
                                    || callee == "__cluster_enviar"
                                    || callee == "__cluster_send"
                                    || callee == "__tcp_aceptar"
                                    || callee == "__tcp_accept"
                                {
                                    TypeInfo::Booleano
                                } else if callee == "__dormir"
                                    || callee == "__sleep"
                                    || callee == "__scope_cancelar"
                                    || callee == "__scope_cancel"
                                    || callee == "__supervisor_agregar"
                                    || callee == "__supervisor_add"
                                    || callee == "__supervisor_iniciar"
                                    || callee == "__supervisor_start"
                                    || callee == "__arc_asignar"
                                    || callee == "__arc_set"
                                {
                                    TypeInfo::Void
                                } else if callee == "__tipo_de" || callee == "__typeof" {
                                    TypeInfo::Texto
                                } else if callee == "__str_ord" || callee == "__str_codigo" {
                                    TypeInfo::Lista(Box::new(TypeInfo::Entero))
                                } else if callee == "__str_chr"
                                    || callee == "__str_caracter"
                                    || callee == "__str_slice"
                                    || callee == "__str_subcadena"
                                    || callee == "__str_concat_list"
                                    || callee == "__str_concatenar_lista"
                                    || callee == "__str_reemplazar"
                                    || callee == "__str_replace"
                                    || callee == "__str_subcadena_chars"
                                    || callee == "__str_slice_chars"
                                {
                                    TypeInfo::Texto
                                } else if callee == "__str_starts_with"
                                    || callee == "__str_empieza_con"
                                {
                                    TypeInfo::Booleano
                                } else if callee == "__str_to_chars"
                                    || callee == "__str_a_caracteres"
                                {
                                    TypeInfo::Lista(Box::new(TypeInfo::Texto))
                                } else if callee == "__map_contiene" || callee == "__map_contains" {
                                    TypeInfo::Booleano
                                } else if callee == "__map_claves" || callee == "__map_keys" {
                                    TypeInfo::Lista(Box::new(TypeInfo::Numero))
                                } else if callee == "__map_obtener"
                                    || callee == "__map_get"
                                    || callee == "__map_nuevo"
                                    || callee == "__map_new"
                                    || callee == "__map_poner"
                                    || callee == "__map_set"
                                {
                                    TypeInfo::Numero
                                } else if callee == "__encoding_utf8"
                                    || callee == "__codificacion_utf8"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento (texto), no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 texto para codificar a UTF-8"
                                                .to_string(),
                                        });
                                    }
                                    if let Some(got) = arg_types.first() {
                                        if !can_assign(&TypeInfo::Texto, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!(
                                                    "'{}' espera 'texto', no '{:?}'",
                                                    callee, got
                                                ),
                                                span: *span,
                                                suggestion: "Pasa un valor de tipo texto"
                                                    .to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Lista(Box::new(TypeInfo::Entero))
                                } else if callee == "__encoding_from_utf8"
                                    || callee == "__desde_utf8"
                                {
                                    if args.len() != 1 {
                                        self.errors.push(SemError {
                                            code: "E040".to_string(),
                                            message: format!(
                                                "'{}' espera 1 argumento (Array<Int>), no {}",
                                                callee,
                                                args.len()
                                            ),
                                            span: *span,
                                            suggestion: "Pasa 1 Array<Int> con bytes UTF-8"
                                                .to_string(),
                                        });
                                    }
                                    if let Some(got) = arg_types.first() {
                                        if !can_assign(
                                            &TypeInfo::Lista(Box::new(TypeInfo::Entero)),
                                            got,
                                        ) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!(
                                                    "'{}' espera 'Array<Int>', no '{:?}'",
                                                    callee, got
                                                ),
                                                span: *span,
                                                suggestion: "Pasa un valor de tipo lista<entero>"
                                                    .to_string(),
                                            });
                                        }
                                    }
                                    TypeInfo::Texto
                                } else if callee == "__regex_is_match"
                                    || callee == "__regex_coincide"
                                {
                                    TypeInfo::Booleano
                                } else if callee == "__regex_reemplazar"
                                    || callee == "__regex_replace"
                                {
                                    // QA: faltaba typing — retornaba Decimal en vez de Texto
                                    TypeInfo::Texto
                                } else if callee == "__http_get"
                                    || callee == "__http_obtener"
                                    || callee == "__http_post"
                                    || callee == "__http_enviar"
                                    || callee == "__hash_sha256"
                                    || callee == "__hash_sha512"
                                {
                                    TypeInfo::Texto
                                } else if callee == "__coro_ceder" || callee == "__coro_yield" {
                                    TypeInfo::Void
                                } else if callee == "__unicode_normalize"
                                    || callee == "__unicode_normalizar"
                                    || callee == "__str_padding_inicio"
                                    || callee == "__str_pad_start"
                                    || callee == "__str_padding_fin"
                                    || callee == "__str_pad_end"
                                    || callee == "__tiempo_formatear"
                                    || callee == "__time_format"
                                    || callee == "__coro_crear"
                                    || callee == "__coro_create"
                                {
                                    TypeInfo::Texto
                                } else if callee == "__fs_listar"
                                    || callee == "__fs_listdir"
                                    || callee == "__env_listar"
                                    || callee == "__env_list"
                                    || callee == "__lector_buffer"
                                    || callee == "__buf_reader"
                                    || callee == "__stream_trozos"
                                    || callee == "__stream_chunks"
                                    || callee == "__regex_capturar"
                                    || callee == "__regex_captures"
                                {
                                    TypeInfo::Lista(Box::new(TypeInfo::Texto))
                                } else if callee == "__str_a_entero" || callee == "__texto_a_entero"
                                {
                                    TypeInfo::Entero
                                } else if callee.starts_with("__") {
                                    // Fallback dinámico: builtins sin typing específico
                                    // retornan Numero (acepta asignación a cualquier tipo)
                                    // antes era Decimal que es estricto y causaba falsos E031
                                    TypeInfo::Numero
                                } else {
                                    let var_type = self.lookup(&callee).map(|s| s.var_type.clone());
                                    match var_type {
                                        Some(TypeInfo::Func {
                                            param_types,
                                            return_type,
                                        }) => {
                                            if args.len() != param_types.len() {
                                                self.errors.push(SemError {
                                                    code: "E040".to_string(),
                                                    message: format!("La función '{}' espera {} argumentos, pero se pasaron {}", callee, param_types.len(), args.len()),
                                                    span: *span,
                                                    suggestion: format!("Pasa {} argumentos a '{}'", param_types.len(), callee),
                                                });
                                            } else {
                                                for (i, (got, expected)) in arg_types
                                                    .iter()
                                                    .zip(param_types.iter())
                                                    .enumerate()
                                                {
                                                    if !can_assign(expected, got) {
                                                        self.errors.push(SemError {
                                                            code: "E041".to_string(),
                                                            message: format!("El argumento {} de '{}' debe ser '{:?}', no '{:?}'", i + 1, callee, expected, got),
                                                            span: *span,
                                                            suggestion: format!("Pasa un valor de tipo '{:?}' en el argumento {}", expected, i + 1),
                                                        });
                                                    }
                                                }
                                            }
                                            *return_type
                                        }
                                        Some(TypeInfo::TypeVar(_)) | Some(TypeInfo::Numero) => {
                                            TypeInfo::Numero
                                        }
                                        Some(other) => {
                                            self.errors.push(SemError {
                                                code: "E058".to_string(),
                                                message: format!("'{}' no es una función, es de tipo '{:?}'", callee, other),
                                                span: *span,
                                                suggestion: format!("'{}' no se puede llamar porque no es una función", callee),
                                            });
                                            TypeInfo::Void
                                        }
                                        None => {
                                            self.errors.push(SemError {
                                                code: "E042".to_string(),
                                                message: format!(
                                                    "La función '{}' no está definida",
                                                    callee
                                                ),
                                                span: *span,
                                                suggestion: format!(
                                                    "Define la función '{}' antes de llamarla",
                                                    callee
                                                ),
                                            });
                                            TypeInfo::Void
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Expr::Lambda { params, body, .. } => {
                        self.scopes.push(Scope::new());
                        for p in params {
                            let pt = self.type_to_info(p.param_type.clone());
                            if let Err(e) = self.current_scope().define(&p.name, pt, p.span) {
                                self.errors.push(e);
                            }
                        }
                        let mut ret_type = TypeInfo::Void;
                        for node in body {
                            match self.analyze_decl_or_stmt(node) {
                                TypeInfo::Void => {}
                                t => ret_type = t,
                            }
                        }
                        self.scopes.pop();
                        ret_type
                    }
                    _ => {
                        let callee_type = self.analyze_expr(callee);
                        match callee_type {
                            TypeInfo::Func {
                                param_types,
                                return_type,
                            } => {
                                if args.len() != param_types.len() {
                                    self.errors.push(SemError {
                                        code: "E040".to_string(),
                                        message: format!(
                                            "La función espera {} argumentos, pero se pasaron {}",
                                            param_types.len(),
                                            args.len()
                                        ),
                                        span: *span,
                                        suggestion: format!(
                                            "Pasa {} argumentos",
                                            param_types.len()
                                        ),
                                    });
                                } else {
                                    for (i, (got, expected)) in
                                        arg_types.iter().zip(param_types.iter()).enumerate()
                                    {
                                        if !can_assign(expected, got) {
                                            self.errors.push(SemError {
                                                code: "E041".to_string(),
                                                message: format!("El argumento {} debe ser '{:?}', no '{:?}'", i + 1, expected, got),
                                                span: *span,
                                                suggestion: format!("Pasa un valor de tipo '{:?}' en el argumento {}", expected, i + 1),
                                            });
                                        }
                                    }
                                }
                                *return_type
                            }
                            _ => {
                                self.errors.push(SemError {
                                    code: "E058".to_string(),
                                    message: format!(
                                        "Solo puedes llamar funciones, no valores de tipo '{:?}'",
                                        callee_type
                                    ),
                                    span: *span,
                                    suggestion: "Usa un identificador de función".to_string(),
                                });
                                TypeInfo::Void
                            }
                        }
                    }
                }
            }
            Expr::List { items, span: _ } => {
                if items.is_empty() {
                    TypeInfo::Lista(Box::new(TypeInfo::Numero))
                } else {
                    let item_type = self.analyze_expr(&items[0]);
                    for item in items[1..].iter() {
                        let t = self.analyze_expr(item);
                        if t != item_type
                            && !(item_type == TypeInfo::Decimal && t == TypeInfo::Entero
                                || item_type == TypeInfo::Entero && t == TypeInfo::Decimal)
                        {
                        }
                    }
                    TypeInfo::Lista(Box::new(item_type))
                }
            }
            Expr::Range {
                start, end, span, ..
            } => {
                let start_type = self.analyze_expr(start);
                let end_type = self.analyze_expr(end);
                for t in [&start_type, &end_type] {
                    if !matches!(t, TypeInfo::Entero | TypeInfo::Decimal) {
                        self.errors.push(SemError {
                            code: "E044".to_string(),
                            message: format!(
                                "Los límites de un rango deben ser numéricos, no '{:?}'",
                                t
                            ),
                            span: *span,
                            suggestion: "Usa números enteros o decimales como límites del rango"
                                .to_string(),
                        });
                    }
                }
                TypeInfo::Lista(Box::new(TypeInfo::Entero))
            }
            Expr::Index { expr, index, span } => {
                let raw_type = self.analyze_expr(expr);
                // Auto-deref Prestado<T> → T para indexación (QA bug #6)
                let expr_type = match &raw_type {
                    TypeInfo::Prestado { inner, .. } => (**inner).clone(),
                    _ => raw_type.clone(),
                };
                let index_type = self.analyze_expr(index);
                let is_range_slice = matches!(index.as_ref(), Expr::Range { .. })
                    || matches!(index_type, TypeInfo::Lista(_));
                if !is_range_slice
                    && index_type != TypeInfo::Entero
                    && index_type != TypeInfo::Numero
                {
                    self.errors.push(SemError {
                        code: "E043".to_string(),
                        message: format!(
                            "El índice debe ser entero o rango, no '{:?}'",
                            index_type
                        ),
                        span: *span,
                        suggestion: "Usa un valor de tipo 'entero' o un rango como índice"
                            .to_string(),
                    });
                }
                if is_range_slice {
                    match expr_type {
                        TypeInfo::Lista(inner) => TypeInfo::Lista(inner),
                        TypeInfo::Texto => TypeInfo::Texto,
                        _ => expr_type,
                    }
                } else {
                    match expr_type {
                        TypeInfo::Lista(inner) => *inner,
                        TypeInfo::Texto => TypeInfo::Texto,
                        TypeInfo::Numero => TypeInfo::Numero,
                        _ => {
                            self.errors.push(SemError {
                                code: "E044".to_string(),
                                message: format!(
                                    "No puedes indexar un valor de tipo '{:?}'",
                                    expr_type
                                ),
                                span: *span,
                                suggestion: "La indexación solo funciona con listas y texto"
                                    .to_string(),
                            });
                            TypeInfo::Decimal
                        }
                    }
                }
            }
            Expr::MethodCall {
                expr,
                method,
                args,
                resolved_func: _,
                span,
            } => {
                let raw_type = self.analyze_expr(expr);
                // Auto-deref Prestado<T> → T para llamadas a métodos (QA bug #6)
                let expr_type = match &raw_type {
                    TypeInfo::Prestado { inner, .. } => (**inner).clone(),
                    _ => raw_type.clone(),
                };
                let mut arg_types = Vec::new();
                for arg in args {
                    arg_types.push(self.analyze_expr(arg));
                }
                match method.as_str() {
                    "agregar" | "push" => match expr_type {
                        TypeInfo::Lista(inner) => {
                            if args.len() != 1 {
                                self.errors.push(SemError {
                                    code: "E045".to_string(),
                                    message: format!(
                                        "'{}' requiere 1 argumento, se pasaron {}",
                                        method,
                                        args.len()
                                    ),
                                    span: *span,
                                    suggestion: format!(
                                        "Pasa exactamente 1 argumento a '{}'",
                                        method
                                    ),
                                });
                            } else if arg_types.len() == 1
                                && !can_assign(&inner, &arg_types[0])
                                && !(*inner == TypeInfo::Numero || arg_types[0] == TypeInfo::Numero)
                            {
                                self.errors.push(SemError {
                                    code: "E046".to_string(),
                                    message: format!(
                                        "'{}' espera un valor de tipo '{:?}', no '{:?}'",
                                        method, inner, arg_types[0]
                                    ),
                                    span: *span,
                                    suggestion: format!(
                                        "Pasa un valor de tipo '{:?}' a '{}'",
                                        inner, method
                                    ),
                                });
                            }
                            TypeInfo::Void
                        }
                        _ => {
                            self.errors.push(SemError {
                                code: "E047".to_string(),
                                message: format!(
                                    "No puedes llamar '{}' en un valor de tipo '{:?}'",
                                    method, expr_type
                                ),
                                span: *span,
                                suggestion: "'agregar' solo se puede llamar en listas".to_string(),
                            });
                            TypeInfo::Void
                        }
                    },
                    "largo" | "len" | "length" => {
                        let base_t = match &expr_type {
                            TypeInfo::Prestado { inner, .. } | TypeInfo::Dueno(inner) => {
                                inner.as_ref()
                            }
                            other => other,
                        };
                        match base_t {
                            TypeInfo::Lista(_) => TypeInfo::Entero,
                            TypeInfo::Texto => TypeInfo::Entero,
                            TypeInfo::Numero => TypeInfo::Numero,
                            _ => {
                                self.errors.push(SemError {
                                    code: "E047".to_string(),
                                    message: format!(
                                        "No puedes llamar '{}' en un valor de tipo '{:?}'",
                                        method, expr_type
                                    ),
                                    span: *span,
                                    suggestion: "'largo' solo se puede llamar en listas y texto"
                                        .to_string(),
                                });
                                TypeInfo::Entero
                            }
                        }
                    }
                    _ => {
                        // Try trait method resolution
                        let resolved =
                            self.resolve_trait_method(&expr_type, method, &arg_types, span);
                        if let Some(ret_type) = resolved {
                            return ret_type;
                        }
                        self.errors.push(SemError {
                            code: "E050".to_string(),
                            message: format!(
                                "El método '{}' no existe para el tipo '{:?}'",
                                method, expr_type
                            ),
                            span: *span,
                            suggestion: format!("Revisa si el método '{}' está disponible", method),
                        });
                        TypeInfo::Void
                    }
                }
            }
            Expr::Lambda { params, body, .. } => {
                self.scopes.push(Scope::new());
                for p in params {
                    let pt = self.type_to_info(p.param_type.clone());
                    if let Err(e) = self.current_scope().define(&p.name, pt, p.span) {
                        self.errors.push(e);
                    }
                }
                let mut ret_type = TypeInfo::Void;
                for node in body {
                    match self.analyze_decl_or_stmt(node) {
                        TypeInfo::Void => {}
                        t => ret_type = t,
                    }
                }
                self.scopes.pop();
                let param_types = params
                    .iter()
                    .map(|p| self.type_to_info(p.param_type.clone()))
                    .collect();
                TypeInfo::Func {
                    param_types,
                    return_type: Box::new(ret_type),
                }
            }
            Expr::StructInit {
                struct_name,
                fields,
                type_args,
                span,
            } => {
                let struct_info = self.structs.get(struct_name).cloned();
                match struct_info {
                    Some((expected_fields, st_type_params)) => {
                        // Build substitution map if type_args provided
                        let subst = if !type_args.is_empty() && !st_type_params.is_empty() {
                            let mut map = HashMap::new();
                            for (tp, ta) in st_type_params.iter().zip(type_args.iter()) {
                                map.insert(tp.clone(), self.type_to_info(ta.clone()));
                            }
                            Some(map)
                        } else {
                            None
                        };
                        let resolved_fields: Vec<(String, TypeInfo)> = if let Some(ref s) = subst {
                            expected_fields
                                .iter()
                                .map(|(name, ft)| (name.clone(), substitute_typevars(ft, s)))
                                .collect()
                        } else {
                            expected_fields.clone()
                        };
                        for (fname, fval) in fields {
                            let val_type = self.analyze_expr(fval);
                            let field_def = resolved_fields.iter().find(|(name, _)| name == fname);
                            match field_def {
                                Some((_, ft)) => {
                                    if !can_assign(ft, &val_type) {
                                        self.errors.push(SemError {
                                            code: "E031".to_string(),
                                            message: format!("El campo '{}' espera un valor de tipo '{:?}', no '{:?}'", fname, ft, val_type),
                                            span: *span,
                                            suggestion: format!("Usa un valor de tipo '{:?}' para el campo '{}'", ft, fname),
                                        });
                                    }
                                }
                                None => {
                                    self.errors.push(SemError {
                                        code: "E059".to_string(),
                                        message: format!(
                                            "El struct '{}' no tiene un campo llamado '{}'",
                                            struct_name, fname
                                        ),
                                        span: *span,
                                        suggestion: format!(
                                            "Revisa los campos de '{}', '{}' no existe",
                                            struct_name, fname
                                        ),
                                    });
                                }
                            }
                        }
                        // Check all required fields are provided
                        for (expected_name, _) in &resolved_fields {
                            if !fields.iter().any(|(name, _)| name == expected_name) {
                                self.errors.push(SemError {
                                    code: "E061".to_string(),
                                    message: format!(
                                        "Falta el campo '{}' en la inicialización de '{}'",
                                        expected_name, struct_name
                                    ),
                                    span: *span,
                                    suggestion: format!(
                                        "Agrega el campo '{}' al inicializar '{}'",
                                        expected_name, struct_name
                                    ),
                                });
                            }
                        }
                        TypeInfo::Struct {
                            name: struct_name.clone(),
                            fields: resolved_fields,
                        }
                    }
                    None => {
                        self.errors.push(SemError {
                            code: "E062".to_string(),
                            message: format!("El struct '{}' no está definido", struct_name),
                            span: *span,
                            suggestion: format!(
                                "Define el struct '{}' antes de usarlo",
                                struct_name
                            ),
                        });
                        TypeInfo::Void
                    }
                }
            }
            Expr::FieldAccess { expr, field, span } => {
                let expr_type = self.analyze_expr(expr);
                // Auto-deref Prestado<T> → T para acceso a campos (QA bug #6)
                let resolved = match &expr_type {
                    TypeInfo::Prestado { inner, .. } => (**inner).clone(),
                    _ => expr_type.clone(),
                };
                match &resolved {
                    TypeInfo::Struct { fields, .. } => {
                        let field_type = fields.iter().find(|(name, _)| name == field);
                        match field_type {
                            Some((_, ft)) => ft.clone(),
                            None => {
                                self.errors.push(SemError {
                                    code: "E059".to_string(),
                                    message: format!(
                                        "El struct no tiene un campo llamado '{}'",
                                        field
                                    ),
                                    span: *span,
                                    suggestion: format!(
                                        "Revisa los campos del struct, '{}' no existe",
                                        field
                                    ),
                                });
                                TypeInfo::Void
                            }
                        }
                    }
                    _ => {
                        self.errors.push(SemError {
                            code: "E060".to_string(),
                            message: format!(
                                "No puedes acceder a un campo de un valor de tipo '{:?}'",
                                resolved
                            ),
                            span: *span,
                            suggestion: "Solo los structs tienen campos".to_string(),
                        });
                        TypeInfo::Void
                    }
                }
            }
            Expr::Grouping { expr, .. } => self.analyze_expr(expr),
            Expr::Cast {
                expr,
                cast_type,
                span,
            } => {
                self.analyze_expr(expr);
                let ti = match cast_type {
                    Type::Entero => TypeInfo::Entero,
                    Type::Decimal => TypeInfo::Decimal,
                    Type::Numero => TypeInfo::Numero,
                    Type::Texto => TypeInfo::Texto,
                    Type::Booleano => TypeInfo::Booleano,
                    Type::Lista(inner) => {
                        let it = self.type_to_info(inner.as_ref().clone());
                        TypeInfo::Lista(Box::new(it))
                    }
                    Type::Struct(name) => {
                        let t = self.type_to_info(Type::Struct(name.clone()));
                        if t == TypeInfo::Void {
                            self.errors.push(SemError {
                                code: "E060".to_string(),
                                message: format!("El tipo '{}' no existe para el cast", name),
                                span: *span,
                                suggestion: "Usa un tipo válido después de 'como'".to_string(),
                            });
                        }
                        t
                    }
                    _ => TypeInfo::Void,
                };
                ti
            }
            Expr::Exito { expr, span } => {
                let inner = self.analyze_expr(expr);
                if inner == TypeInfo::Void {
                    self.errors.push(SemError {
                        code: "E064".to_string(),
                        message: "No puedes crear un resultado exitoso con un valor vacío"
                            .to_string(),
                        span: *span,
                        suggestion: "Pasa un valor válido a 'exito()'.".to_string(),
                    });
                }
                TypeInfo::Resultado {
                    ok: Box::new(inner),
                    err: Box::new(TypeInfo::Void),
                }
            }
            Expr::Error { expr, span } => {
                let inner = self.analyze_expr(expr);
                if inner == TypeInfo::Void {
                    self.errors.push(SemError {
                        code: "E064".to_string(),
                        message: "No puedes crear un resultado de error con un valor vacío"
                            .to_string(),
                        span: *span,
                        suggestion: "Pasa un valor válido a 'error()'.".to_string(),
                    });
                }
                TypeInfo::Resultado {
                    ok: Box::new(TypeInfo::Void),
                    err: Box::new(inner),
                }
            }
            Expr::Intentar { expr, span } => {
                let inner = self.analyze_expr(expr);
                match inner {
                    TypeInfo::Resultado { ok, err: _ } => *ok,
                    _ => {
                        self.errors.push(SemError {
                            code: "E065".to_string(),
                            message: format!("'intentar' solo funciona con expresiones de tipo 'resultado', no '{:?}'", inner),
                            span: *span,
                            suggestion: "Usa 'intentar' solo con valores de tipo 'resultado'.".to_string(),
                        });
                        TypeInfo::Void
                    }
                }
            }
            Expr::Algun { expr, span: _ } => {
                let inner = self.analyze_expr(expr);
                if inner == TypeInfo::Void {
                    self.errors.push(SemError {
                        code: "E064".to_string(),
                        message: "No puedes crear un valor opcional con un valor vacío".to_string(),
                        span: expr.span(),
                        suggestion: "Pasa un valor válido a 'algun()'.".to_string(),
                    });
                }
                TypeInfo::Opcion(Box::new(inner))
            }
            Expr::Ninguno { .. } => TypeInfo::Opcion(Box::new(TypeInfo::Void)),
            Expr::Tuple { items, span: _ } => {
                let mut types = Vec::new();
                for item in items {
                    types.push(self.analyze_expr(item));
                }
                TypeInfo::Tuple(types)
            }
            Expr::TupleAccess { expr, index, span } => {
                let expr_type = self.analyze_expr(expr);
                match &expr_type {
                    TypeInfo::Tuple(types) => {
                        if *index >= types.len() {
                            self.errors.push(SemError {
                                code: "E067".to_string(),
                                message: format!(
                                    "Índice {} fuera de rango para tupla de {} elementos",
                                    index,
                                    types.len()
                                ),
                                span: *span,
                                suggestion: format!("Usa un índice entre 0 y {}", types.len() - 1),
                            });
                            TypeInfo::Void
                        } else {
                            types[*index].clone()
                        }
                    }
                    _ => {
                        self.errors.push(SemError {
                            code: "E060".to_string(),
                            message: format!(
                                "No puedes acceder por índice a un valor de tipo '{:?}'",
                                expr_type
                            ),
                            span: *span,
                            suggestion: "El acceso por índice numérico solo funciona con tuplas"
                                .to_string(),
                        });
                        TypeInfo::Void
                    }
                }
            }
            Expr::EnumCtor {
                enum_name,
                variant,
                args,
                span,
            } => {
                let enum_variants = self.enums.get(enum_name).cloned();
                match enum_variants {
                    Some(variants) => {
                        let var_info = variants.iter().find(|(name, _)| name == variant);
                        match var_info {
                            Some((_, expected_types)) => {
                                for (i, arg) in args.iter().enumerate() {
                                    let arg_type = self.analyze_expr(arg);
                                    if i < expected_types.len()
                                        && !can_assign(&expected_types[i], &arg_type)
                                    {
                                        self.errors.push(SemError {
                                                code: "E031".to_string(),
                                                message: format!(
                                                    "El argumento {} de la variante '{}' espera un tipo '{:?}', no '{:?}'",
                                                    i + 1, variant, expected_types[i], arg_type
                                                ),
                                                span: *span,
                                                suggestion: format!(
                                                    "Usa un valor de tipo '{:?}' en el argumento {}",
                                                    expected_types[i], i + 1
                                                ),
                                            });
                                    }
                                }
                                TypeInfo::Enum(enum_name.clone())
                            }
                            None => {
                                self.errors.push(SemError {
                                    code: "E066".to_string(),
                                    message: format!(
                                        "La enumeración '{}' no tiene una variante llamada '{}'",
                                        enum_name, variant
                                    ),
                                    span: *span,
                                    suggestion: format!(
                                        "Revisa las variantes de '{}', '{}' no existe",
                                        enum_name, variant
                                    ),
                                });
                                TypeInfo::Void
                            }
                        }
                    }
                    None => {
                        self.errors.push(SemError {
                            code: "E062".to_string(),
                            message: format!("La enumeración '{}' no está definida", enum_name),
                            span: *span,
                            suggestion: format!(
                                "Define la enumeración '{}' antes de usarla",
                                enum_name
                            ),
                        });
                        TypeInfo::Void
                    }
                }
            }
            Expr::Ternary {
                condition,
                true_branch,
                false_branch,
                span,
            } => {
                let cond_type = self.analyze_expr(condition);
                if cond_type != TypeInfo::Booleano {
                    self.errors.push(SemError {
                        code: "E034".to_string(),
                        message: format!(
                            "La condición del operador ternario debe ser booleano, no '{:?}'",
                            cond_type
                        ),
                        span: *span,
                        suggestion: "Usa una expresión booleana como condición".to_string(),
                    });
                }
                let true_type = self.analyze_expr(true_branch);
                let false_type = self.analyze_expr(false_branch);
                if true_type != false_type
                    && !can_assign(&true_type, &false_type)
                    && !can_assign(&false_type, &true_type)
                {
                    self.errors.push(SemError {
                        code: "E031".to_string(),
                        message: format!(
                            "El operador ternario requiere que ambas ramas tengan el mismo tipo, no '{:?}' y '{:?}'",
                            true_type, false_type
                        ),
                        span: *span,
                        suggestion: "Ambas ramas deben ser del mismo tipo".to_string(),
                    });
                }
                true_type
            }
            Expr::Esperar { expr, .. } => self.analyze_expr(expr),
            Expr::SafeFieldAccess { expr, field, span } => {
                let expr_type = self.analyze_expr(expr);
                match expr_type {
                    TypeInfo::Struct { ref fields, .. } => {
                        if let Some((_, ft)) = fields.iter().find(|(name, _)| name == field) {
                            ft.clone()
                        } else {
                            self.errors.push(SemError {
                                code: "E059".to_string(),
                                message: format!("El struct no tiene un campo llamado '{}'", field),
                                span: *span,
                                suggestion: format!(
                                    "Revisa los campos del struct, '{}' no existe",
                                    field
                                ),
                            });
                            TypeInfo::Decimal
                        }
                    }
                    _ => TypeInfo::Numero,
                }
            }
            Expr::Elvis { expr, default, .. } => {
                let expr_type = self.analyze_expr(expr);
                let default_type = self.analyze_expr(default);
                if can_assign(&default_type, &expr_type) {
                    default_type
                } else {
                    expr_type
                }
            }
            Expr::Comprehension {
                expr: inner_expr,
                var_name,
                iter,
                condition,
                span,
            } => {
                let iter_type = self.analyze_expr(iter);
                let elem_type = match iter_type {
                    TypeInfo::Lista(inner) => *inner,
                    _ => TypeInfo::Entero,
                };
                self.scopes.push(Scope::new());
                if let Err(e) = self.current_scope().define(var_name, elem_type, *span) {
                    self.errors.push(e);
                }
                if let Some(cond) = condition {
                    self.analyze_expr(cond);
                }
                let item_type = self.analyze_expr(inner_expr);
                self.scopes.pop();
                TypeInfo::Lista(Box::new(item_type))
            }
            Expr::Query {
                var_name,
                source,
                where_clause,
                order_by,
                select_expr,
                span,
                ..
            } => {
                let src_type = self.analyze_expr(source);
                let elem_type = match src_type {
                    TypeInfo::Lista(inner) => *inner,
                    _ => TypeInfo::Numero,
                };
                self.scopes.push(Scope::new());
                if let Err(e) = self.current_scope().define(var_name, elem_type, *span) {
                    self.errors.push(e);
                }
                if let Some(w) = where_clause {
                    self.analyze_expr(w);
                }
                if let Some(o) = order_by {
                    self.analyze_expr(o);
                }
                let res_type = self.analyze_expr(select_expr);
                self.scopes.pop();
                TypeInfo::Lista(Box::new(res_type))
            }
            Expr::Comptime { expr, span: _ } => self.analyze_expr(expr),
        }
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.lookup(name) {
                return Some(sym);
            }
        }
        None
    }

    fn resolve_trait_method(
        &self,
        receiver_type: &TypeInfo,
        method: &str,
        _arg_types: &[TypeInfo],
        span: &Span,
    ) -> Option<TypeInfo> {
        self.resolve_trait_method_mangled(receiver_type, method, _arg_types, span)
            .map(|(t, _)| t)
    }

    fn resolve_trait_method_mangled(
        &self,
        receiver_type: &TypeInfo,
        method: &str,
        _arg_types: &[TypeInfo],
        span: &Span,
    ) -> Option<(TypeInfo, String)> {
        let _ = span;

        // When receiver is a TypeVar, look up the bound trait and substitute defaults
        if let TypeInfo::TypeVar(tv) = receiver_type {
            let bound_trait = self.find_bound_for_typevar(tv)?;
            let trait_sig = self.traits.get(&bound_trait)?;
            let (methods, assoc_types) = trait_sig;
            let mut subst = HashMap::new();
            for at in assoc_types {
                if let Some(default) = &at.default {
                    subst.insert(at.name.clone(), default.clone());
                }
            }
            for (t_mname, _t_params, t_ret) in methods {
                if t_mname == method {
                    let mangled = format!("{}_{}_{}", tv, bound_trait, method);
                    let resolved_ret = substitute_typevars(t_ret, &subst);
                    return Some((resolved_ret, mangled));
                }
            }
            return None;
        }

        let type_name = match receiver_type {
            TypeInfo::Struct { name, .. } => name.clone(),
            TypeInfo::Enum(name) => name.clone(),
            _ => type_info_to_impl_name(receiver_type)?,
        };

        // Look through impls for the receiver type, using the impl's concrete return type
        for (impl_type, trait_name) in self.impls.keys() {
            if impl_type != &type_name {
                continue;
            }
            let mangled = if trait_name.is_empty() {
                format!("{}_{}", impl_type, method)
            } else {
                format!("{}_{}_{}", impl_type, trait_name, method)
            };
            if let Some((ret, _, _, _)) = self.functions.get(&mangled) {
                return Some((ret.clone(), mangled));
            }
        }
        None
    }

    fn find_bound_for_typevar(&self, tv: &str) -> Option<String> {
        for bounds in self.type_param_bounds.values() {
            for (name, bound) in bounds {
                if name == tv {
                    return Some(bound.clone());
                }
            }
        }
        None
    }

    fn current_scope(&mut self) -> &mut Scope {
        if self.scopes.is_empty() {
            self.scopes.push(Scope::new());
        }
        self.scopes
            .last_mut()
            .expect("current_scope: scopes empty after push")
    }

    // Vincula las variables capturadas por un patrón de if-let / arm de match
    // en el scope actual (tipo dinámico `Numero` — acepta cualquier valor).
    fn bind_pattern_vars(&mut self, pattern: &Expr, span: Span) {
        match pattern {
            Expr::Ident { name, .. } => {
                let _ = self.current_scope().define(name, TypeInfo::Numero, span);
            }
            Expr::Call { args, .. } => {
                for a in args.iter() {
                    self.bind_pattern_vars(a, span);
                }
            }
            Expr::Algun { expr, .. } => {
                self.bind_pattern_vars(expr, span);
            }
            Expr::Exito { expr, .. } => {
                self.bind_pattern_vars(expr, span);
            }
            Expr::Error { expr, .. } => {
                self.bind_pattern_vars(expr, span);
            }
            Expr::Ninguno { .. } => {}
            // QA bug #3: destructurar enums de usuario con datos
            // `Exitoso(valor)` / `Resultado::Exitoso(valor)` como patrón if-let
            Expr::EnumCtor { args, .. } => {
                for a in args.iter() {
                    self.bind_pattern_vars(a, span);
                }
            }
            Expr::Tuple { items, .. } => {
                for it in items.iter() {
                    self.bind_pattern_vars(it, span);
                }
            }
            Expr::StructInit { fields, .. } => {
                for (_name, val) in fields.iter() {
                    self.bind_pattern_vars(val, span);
                }
            }
            Expr::List { items, .. } => {
                for it in items.iter() {
                    self.bind_pattern_vars(it, span);
                }
            }
            _ => {}
        }
    }
}

impl SemanticAnalyzer {
    fn resolve_type(&self, t: Type, type_params: &[String]) -> TypeInfo {
        match t {
            Type::Struct(ref name) if type_params.contains(name) => TypeInfo::TypeVar(name.clone()),
            Type::Struct(ref name) if self.enums.contains_key(name) => TypeInfo::Enum(name.clone()),
            Type::GenericStruct { name, args } => {
                // Resolve type args too
                let resolved_args: Vec<TypeInfo> = args
                    .into_iter()
                    .map(|a| self.resolve_type(a, type_params))
                    .collect();
                if let Some((fields, st_type_params)) = self.structs.get(&name) {
                    let mut subst = HashMap::new();
                    for (tp, ta) in st_type_params.iter().zip(resolved_args.iter()) {
                        subst.insert(tp.clone(), ta.clone());
                    }
                    let resolved_fields: Vec<(String, TypeInfo)> = fields
                        .iter()
                        .map(|(fname, ft)| (fname.clone(), substitute_typevars(ft, &subst)))
                        .collect();
                    TypeInfo::Struct {
                        name,
                        fields: resolved_fields,
                    }
                } else if self.enums.contains_key(&name) {
                    TypeInfo::Enum(name)
                } else {
                    TypeInfo::Struct {
                        name,
                        fields: vec![],
                    }
                }
            }
            _ => self.type_to_info(t),
        }
    }

    fn type_to_info(&self, t: Type) -> TypeInfo {
        match t {
            Type::Numero => TypeInfo::Numero,
            Type::Entero => TypeInfo::Entero,
            Type::Decimal => TypeInfo::Decimal,
            Type::Texto => TypeInfo::Texto,
            Type::Booleano => TypeInfo::Booleano,
            Type::Lista(inner) => TypeInfo::Lista(Box::new(self.type_to_info(*inner))),
            Type::Func {
                param_types,
                return_type,
            } => TypeInfo::Func {
                param_types: param_types
                    .into_iter()
                    .map(|t| self.type_to_info(t))
                    .collect(),
                return_type: Box::new(self.type_to_info(*return_type)),
            },
            Type::GenericStruct { name, args } => {
                if let Some((fields, type_params)) = self.structs.get(&name) {
                    let mut subst = HashMap::new();
                    for (tp, ta) in type_params.iter().zip(args.iter()) {
                        subst.insert(tp.clone(), self.type_to_info(ta.clone()));
                    }
                    let resolved_fields: Vec<(String, TypeInfo)> = fields
                        .iter()
                        .map(|(fname, ft)| (fname.clone(), substitute_typevars(ft, &subst)))
                        .collect();
                    TypeInfo::Struct {
                        name,
                        fields: resolved_fields,
                    }
                } else if self.enums.contains_key(&name) {
                    TypeInfo::Enum(name)
                } else {
                    TypeInfo::Struct {
                        name,
                        fields: vec![],
                    }
                }
            }
            Type::Struct(name) => {
                if self.enums.contains_key(&name) {
                    TypeInfo::Enum(name)
                } else {
                    let fields = self
                        .structs
                        .get(&name)
                        .map(|(f, _)| f.clone())
                        .unwrap_or_default();
                    TypeInfo::Struct { name, fields }
                }
            }
            Type::Resultado { ok, err } => TypeInfo::Resultado {
                ok: Box::new(self.type_to_info(*ok)),
                err: Box::new(self.type_to_info(*err)),
            },
            Type::Opcion(inner) => TypeInfo::Opcion(Box::new(self.type_to_info(*inner))),
            Type::Tuple(types) => {
                TypeInfo::Tuple(types.into_iter().map(|t| self.type_to_info(t)).collect())
            }
            Type::ImplTrait(name) => TypeInfo::TypeVar(name), // opaque; resolved at call site
            Type::Prestado { inner, mutable } => TypeInfo::Prestado {
                inner: Box::new(self.type_to_info(*inner)),
                mutable,
            },
            Type::Dueno(inner) => TypeInfo::Dueno(Box::new(self.type_to_info(*inner))),
        }
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
        Type::Prestado { inner, .. } => type_to_impl_name(inner),
        Type::Dueno(inner) => type_to_impl_name(inner),
        Type::Func { .. } | Type::ImplTrait(_) => None,
    }
}

fn type_info_to_impl_name(t: &TypeInfo) -> Option<String> {
    match t {
        TypeInfo::Struct { name, .. } => Some(name.clone()),
        TypeInfo::Enum(name) => Some(name.clone()),
        TypeInfo::Entero => Some("entero".to_string()),
        TypeInfo::Decimal => Some("decimal".to_string()),
        TypeInfo::Texto => Some("texto".to_string()),
        TypeInfo::Booleano => Some("booleano".to_string()),
        TypeInfo::Numero => Some("numero".to_string()),
        TypeInfo::Lista(_) => Some("lista".to_string()),
        TypeInfo::Resultado { .. } => Some("resultado".to_string()),
        TypeInfo::Opcion(_) => Some("opcion".to_string()),
        TypeInfo::Tuple(_) => Some("tupla".to_string()),
        TypeInfo::Prestado { inner, .. } => type_info_to_impl_name(inner),
        TypeInfo::Dueno(inner) => type_info_to_impl_name(inner),
        TypeInfo::Void | TypeInfo::Func { .. } | TypeInfo::TypeVar(_) => None,
    }
}

fn substitute_typevars(typ: &TypeInfo, subst: &HashMap<String, TypeInfo>) -> TypeInfo {
    match typ {
        TypeInfo::TypeVar(name) => subst.get(name).cloned().unwrap_or(typ.clone()),
        TypeInfo::Lista(inner) => TypeInfo::Lista(Box::new(substitute_typevars(inner, subst))),
        TypeInfo::Func {
            param_types,
            return_type,
        } => TypeInfo::Func {
            param_types: param_types
                .iter()
                .map(|p| substitute_typevars(p, subst))
                .collect(),
            return_type: Box::new(substitute_typevars(return_type, subst)),
        },
        TypeInfo::Resultado { ok, err } => TypeInfo::Resultado {
            ok: Box::new(substitute_typevars(ok, subst)),
            err: Box::new(substitute_typevars(err, subst)),
        },
        TypeInfo::Opcion(inner) => TypeInfo::Opcion(Box::new(substitute_typevars(inner, subst))),
        TypeInfo::Tuple(types) => TypeInfo::Tuple(
            types
                .iter()
                .map(|t| substitute_typevars(t, subst))
                .collect(),
        ),
        TypeInfo::Struct { name, fields } if fields.is_empty() => {
            subst.get(name).cloned().unwrap_or(typ.clone())
        }
        TypeInfo::Struct { name, fields } => TypeInfo::Struct {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(n, t)| (n.clone(), substitute_typevars(t, subst)))
                .collect(),
        },
        _ => typ.clone(),
    }
}

fn is_numeric(t: &TypeInfo) -> bool {
    matches!(t, TypeInfo::Entero | TypeInfo::Decimal | TypeInfo::Numero)
}

fn can_assign(target: &TypeInfo, value: &TypeInfo) -> bool {
    if target == value {
        return true;
    }
    // Numero (dynamic type) accepts any value AND can be assigned from any type
    if *target == TypeInfo::Numero || *value == TypeInfo::Numero {
        return true;
    }
    // TypeVar matches any type
    if matches!(target, TypeInfo::TypeVar(_)) || matches!(value, TypeInfo::TypeVar(_)) {
        return true;
    }
    if *target == TypeInfo::Decimal && *value == TypeInfo::Entero {
        return true;
    }
    if let (TypeInfo::Lista(t_inner), TypeInfo::Lista(v_inner)) = (target, value) {
        if **v_inner == TypeInfo::Void {
            return true;
        }
        return can_assign(t_inner, v_inner);
    }
    if let (
        TypeInfo::Func {
            param_types: tp,
            return_type: tr,
        },
        TypeInfo::Func {
            param_types: vp,
            return_type: vr,
        },
    ) = (target, value)
    {
        if tp.len() != vp.len() {
            return false;
        }
        if !can_assign(tr, vr) {
            return false;
        }
        for (t, v) in tp.iter().zip(vp.iter()) {
            if !can_assign(t, v) {
                return false;
            }
        }
        return true;
    }
    if let (
        TypeInfo::Resultado { ok: tok, err: terr },
        TypeInfo::Resultado { ok: vok, err: verr },
    ) = (target, value)
    {
        let ok_compat = can_assign(tok, vok) || **vok == TypeInfo::Void || **tok == TypeInfo::Void;
        let err_compat =
            can_assign(terr, verr) || **verr == TypeInfo::Void || **terr == TypeInfo::Void;
        return ok_compat && err_compat;
    }
    if let (TypeInfo::Opcion(target_inner), TypeInfo::Opcion(value_inner)) = (target, value) {
        if **value_inner == TypeInfo::Void {
            return true;
        }
        return can_assign(target_inner, value_inner);
    }
    // Tipado nominal para structs de usuario: dos Struct con el mismo nombre son
    // el mismo tipo independientemente de los campos (fix bug #4 — structs
    // recursivos donde el placeholder tiene fields vacíos)
    if let (TypeInfo::Struct { name: tn, .. }, TypeInfo::Struct { name: vn, .. }) = (target, value)
    {
        if tn == vn {
            return true;
        }
    }
    if let (TypeInfo::Enum(a), TypeInfo::Enum(b)) = (target, value) {
        return a == b;
    }
    if let (TypeInfo::Tuple(t), TypeInfo::Tuple(v)) = (target, value) {
        if t.len() != v.len() {
            return false;
        }
        for (ta, va) in t.iter().zip(v.iter()) {
            if !can_assign(ta, va) {
                return false;
            }
        }
        return true;
    }
    if let (TypeInfo::Prestado { inner: t_inner, .. }, TypeInfo::Prestado { inner: v_inner, .. }) =
        (target, value)
    {
        return can_assign(t_inner, v_inner);
    }
    if let TypeInfo::Prestado { inner: t_inner, .. } = target {
        return can_assign(t_inner, value);
    }
    if let TypeInfo::Prestado { inner: v_inner, .. } = value {
        return can_assign(target, v_inner);
    }
    if let (TypeInfo::Dueno(t_inner), TypeInfo::Dueno(v_inner)) = (target, value) {
        return can_assign(t_inner, v_inner);
    }
    if let TypeInfo::Dueno(t_inner) = target {
        return can_assign(t_inner, value);
    }
    if let TypeInfo::Dueno(v_inner) = value {
        return can_assign(target, v_inner);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_lexer::Lexer;
    use lumen_parser::Parser;

    fn analyze(source: &str) -> Vec<SemError> {
        let lexer = Lexer::new(source);
        let (tokens, lex_errors) = lexer.tokenize();
        assert!(lex_errors.is_empty(), "Lexer errors: {:?}", lex_errors);
        let parser = Parser::new(tokens);
        let (mut program, parse_errors) = parser.parse();
        assert!(parse_errors.is_empty(), "Parse errors: {:?}", parse_errors);
        let sema = SemanticAnalyzer::new();
        sema.analyze(&mut program)
    }

    #[test]
    fn test_valid_program() {
        let errors = analyze("numero x = 42;");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_type_mismatch() {
        let errors = analyze(r#"entero x = "hola";"#);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E031");
    }

    #[test]
    fn test_undefined_variable() {
        let errors = analyze("x = 42;");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E033");
    }

    #[test]
    fn test_redeclaration() {
        let errors = analyze("numero x = 1; numero x = 2;");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E032");
    }

    #[test]
    fn test_boolean_condition() {
        let errors = analyze("numero x = 1; si (x) { }");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E034");
    }

    #[test]
    fn test_valid_if() {
        let errors = analyze("booleano flag = verdadero; si (flag) { }");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_arithmetic_type_error() {
        let errors = analyze(r#"entero x = 1 + "hola";"#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_sub_non_number_error() {
        let errors = analyze(r#"numero x = 1 - "hola";"#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_mul_non_number_error() {
        let errors = analyze(r#"numero x = 2 * "a";"#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_div_non_number_error() {
        let errors = analyze(r#"numero x = 4 / "b";"#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_string_concatenation_valid() {
        let errors = analyze(r#"texto s = "hola" + " mundo";"#);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_equal_different_types() {
        let errors = analyze(r#"booleano b = 1 =="hola";"#);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E036");
    }

    #[test]
    fn test_not_equal_different_types() {
        let errors = analyze(r#"booleano b = verdadero != 3;"#);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E036");
    }

    #[test]
    fn test_comparison_less_non_number() {
        // String comparison is now allowed (same type)
        let errors = analyze(r#"booleano b = "a" < "b";"#);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_comparison_numeric_any_type() {
        // Numeric comparisons accept any type on either side (runtime truthiness,
        // parity con el VM: `i < n` con tipos mixtos es válido).
        let errors = analyze(r#"booleano b = 1 < "hola";"#);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_logical_dynamic_truthiness() {
        // && / || aceptan cualquier valor (truthiness dinámica en runtime,
        // parity con el VM: `mientras i < n && cs[i] != "\n"`).
        let errors = analyze("booleano b = verdadero && 1;");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_function_call_arg_count() {
        let source = "funcion numero suma(numero a, numero b) { retornar a + b; } suma(1);";
        let errors = analyze(source);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E040");
    }

    #[test]
    fn test_function_call_arg_type() {
        let source =
            r#"funcion numero suma(entero a, entero b) { retornar a + b; } suma(1, "hola");"#;
        let errors = analyze(source);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E041");
    }

    #[test]
    fn test_undefined_function() {
        let errors = analyze("foo(1);");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E042");
    }

    #[test]
    fn test_scoping() {
        let source = "numero x = 1; { numero y = 2; } y = 3;";
        let errors = analyze(source);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E033");
    }

    #[test]
    fn test_valid_function_program() {
        let source = "funcion numero suma(numero a, numero b) { retornar a + b; }
numero x = suma(3, 4);
imprimir(x);";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_valid_complex_program() {
        let source = "numero contador = 0;
mientras (contador < 10) {
    contador = contador + 1;
}
imprimir(contador);";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_negate_non_number() {
        let errors = analyze(r#"x = -("hola");"#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_not_non_boolean() {
        let errors = analyze("x = !42;");
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_if_else() {
        let source = "booleano flag = verdadero;
si (flag) {
    numero x = 1;
} sino {
    numero y = 2;
}";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_for_loop() {
        let source = "para (numero i = 0; i < 10; i = i + 1) { imprimir(i); }";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_while_loop() {
        let source = "numero i = 0; mientras (i < 10) { i = i + 1; }";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_foreach_valid() {
        let source = "lista<entero> nums = [1, 2, 3];
para n en nums {
    imprimir(n);
}";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_foreach_type_error() {
        let source = "entero x = 42;
para n en x {
    imprimir(n);
}";
        let errors = analyze(source);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_foreach_strings() {
        let source = r#"lista<texto> nombres = ["Ana", "Luis"];
para nombre en nombres {
    imprimir(nombre);
}"#;
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_foreach_english() {
        let source = "array<integer> nums = [1, 2, 3];
for n in nums {
    print(n);
}";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_foreach_nested() {
        let source = "lista<entero> nums = [1, 2];
para a en nums {
    para b en nums {
        imprimir(a * b);
    }
}";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_foreach_in_function() {
        let source = "funcion texto unir(lista<texto> palabras) {
    texto res = \"\";
    para p en palabras {
        res = res + p;
    }
    retornar res;
}
imprimir(unir([\"a\", \"b\"]));";
        let errors = analyze(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_opcion_valid_algun() {
        let errors = analyze("opcion<entero> x = algun(42);");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_opcion_valid_ninguno() {
        let errors = analyze("opcion<entero> x = ninguno;");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_opcion_assign_ninguno_to_any() {
        let errors = analyze("opcion<texto> x = ninguno;");
        assert!(errors.is_empty());
    }

    #[test]
    fn test_opcion_type_mismatch() {
        let errors = analyze("opcion<texto> x = algun(42);");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E031");
    }

    #[test]
    fn test_opcion_english_keywords() {
        let errors = analyze("option<integer> x = some(42); option<string> y = none;");
        assert!(errors.is_empty());
    }

    // --- Generics tests ---

    #[test]
    fn test_generic_function_valid() {
        let src = "funcion T identidad<T>(T valor) { retornar valor; }
entero x = identidad<entero>(42);
imprimir(x);";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_generic_function_type_mismatch() {
        let src = "funcion T identidad<T>(T valor) { retornar valor; }
entero x = identidad<entero>(\"hola\");";
        let errors = analyze(src);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E041");
    }

    #[test]
    fn test_generic_struct_valid() {
        let src = "estructura Par<T, U> { primero: T, segundo: U }
Par<entero, texto> p = Par<entero, texto> { primero: 1, segundo: \"hola\" };
imprimir(p.primero);
imprimir(p.segundo);";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_generic_struct_field_type_mismatch() {
        let src = "estructura Par<T, U> { primero: T, segundo: U }
Par<entero, texto> p = Par<entero, texto> { primero: \"mal\", segundo: \"hola\" };";
        let errors = analyze(src);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E031");
    }

    #[test]
    fn test_generic_identity_different_types() {
        let src = "funcion T id<T>(T v) { retornar v; }
entero x = id<entero>(42);
texto s = id<texto>(\"hola\");
decimal d = id<decimal>(3.5);";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_match_exhaustiveness_error() {
        let src = "enum Color { Rojo, Verde, Azul }
Color c = Color::Rojo;
elegir (c) {
    caso Color::Rojo: imprimir(\"rojo\");
    caso Color::Verde: imprimir(\"verde\");
}";
        let errors = analyze(src);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E080");
    }

    #[test]
    fn test_match_exhaustiveness_with_default() {
        let src = "enum Color { Rojo, Verde, Azul }
Color c = Color::Rojo;
elegir (c) {
    caso Color::Rojo: imprimir(\"rojo\");
    defecto: imprimir(\"otro\");
}";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_match_exhaustiveness_all_covered() {
        let src = "enum Color { Rojo, Verde, Azul }
Color c = Color::Rojo;
elegir (c) {
    caso Color::Rojo: imprimir(\"rojo\");
    caso Color::Verde: imprimir(\"verde\");
    caso Color::Azul: imprimir(\"azul\");
}";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_match_guard_boolean_type_error() {
        let src = "entero x = 1;
elegir (x) {
    caso 1 si 42: imprimir(\"mal\");
}";
        let errors = analyze(src);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E034");
    }

    #[test]
    fn test_match_guard_valid_boolean() {
        let src = "entero x = 5;
elegir (x) {
    caso 5 si x > 3: imprimir(\"valido\");
}";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_match_exhaustiveness_enum_with_data() {
        let src = "enum Forma { Circulo(decimal), Cuadrado, Triangulo }
Forma f = Forma::Circulo(5.0);
elegir (f) {
    caso Forma::Circulo(5.0): imprimir(\"circulo\");
    caso Forma::Cuadrado: imprimir(\"cuadrado\");
}";
        let errors = analyze(src);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E080");
    }

    #[test]
    fn test_match_range_pattern_numero() {
        let src = "entero x = 5;
elegir (x) {
    caso 0..10: imprimir(\"bajo\");
    caso 10..20: imprimir(\"medio\");
    defecto: imprimir(\"alto\");
}";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_match_range_pattern_inclusive() {
        let src = "entero x = 5;
elegir (x) {
    caso 0..=5: imprimir(\"bajo\");
    defecto: imprimir(\"alto\");
}";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_match_or_patterns_ok() {
        let src = "enum Color { Rojo, Verde, Azul }
Color c = Color::Rojo;
elegir (c) {
    caso Color::Rojo | Color::Verde: imprimir(\"calido\");
    caso Color::Azul: imprimir(\"frio\");
}";
        let errors = analyze(src);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_match_or_patterns_exhaustive_count() {
        let src = "enum Color { Rojo, Verde, Azul }
Color c = Color::Rojo;
elegir (c) {
    caso Color::Rojo | Color::Verde: imprimir(\"calido\");
}";
        let errors = analyze(src);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E080");
    }
}
