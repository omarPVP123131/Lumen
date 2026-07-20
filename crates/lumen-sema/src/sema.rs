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

pub struct SemanticAnalyzer {
    scopes: Vec<Scope>,
    functions: HashMap<String, FuncSig>,
    structs: HashMap<String, StructDef>,
    enums: HashMap<String, Vec<(String, Vec<TypeInfo>)>>,
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
            errors: Vec::new(),
            loop_depth: 0,
        }
    }

    pub fn analyze(mut self, program: &mut Program) -> Vec<SemError> {
        self.collect_functions(program);
        self.collect_structs(program);
        self.collect_enums(program);
        self.analyze_program(program);
        self.errors
    }

    fn collect_functions(&mut self, program: &Program) {
        for node in program {
            if let DeclOrStmt::Decl(Decl::Function {
                return_type,
                name,
                params,
                type_params,
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
        for node in program {
            if let DeclOrStmt::Decl(Decl::Struct {
                name,
                fields,
                type_params,
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
                let declared_type = self.type_to_info(var_type.clone());
                let _init_type = init
                    .as_ref()
                    .map(|e| self.analyze_expr(e))
                    .unwrap_or(declared_type.clone());
                if let Some(ref init_expr) = init {
                    let init_type = self.analyze_expr(init_expr);
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
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> TypeInfo {
        match stmt {
            Stmt::Assignment { name, value, span } => {
                let value_type = self.analyze_expr(value);
                if let Some(sym) = self.lookup(name) {
                    if !can_assign(&sym.var_type, &value_type) {
                        self.errors.push(SemError {
                            code: "E031".to_string(),
                            message: format!("No puedes asignar un valor de tipo '{:?}' a la variable '{}' de tipo '{:?}'", value_type, name, sym.var_type),
                            span: *span,
                            suggestion: format!("Usa un valor de tipo '{:?}' para asignar a '{}'", sym.var_type, name),
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
            Stmt::Break { span } => {
                if self.loop_depth == 0 {
                    self.errors.push(SemError {
                        code: "E054".to_string(),
                        message: "'romper' solo se puede usar dentro de un ciclo".to_string(),
                        span: *span,
                        suggestion: "Usa 'romper' dentro de 'mientras' o 'para'".to_string(),
                    });
                }
                TypeInfo::Void
            }
            Stmt::Continue { span } => {
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
                span: _,
            } => {
                let expr_type = self.analyze_expr(expr);
                for arm in arms {
                    let arm_val_type = self.analyze_expr(&arm.value);
                    if arm_val_type != expr_type
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
                    self.scopes.push(Scope::new());
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
                let expr_type = self.analyze_expr(expr);
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
            Stmt::Expr { expr, .. } => self.analyze_expr(expr),
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
                span,
            } => {
                let lt = self.analyze_expr(left);
                let rt = self.analyze_expr(right);
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        if matches!(op, BinOp::Add)
                            && lt == TypeInfo::Texto
                            && rt == TypeInfo::Texto
                        {
                            TypeInfo::Texto
                        } else if lt == TypeInfo::Entero && rt == TypeInfo::Entero {
                            TypeInfo::Entero
                        } else if is_numeric(&lt) && is_numeric(&rt) {
                            TypeInfo::Decimal
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
                        if is_numeric(&lt) && is_numeric(&rt) {
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
                        if lt != TypeInfo::Booleano || rt != TypeInfo::Booleano {
                            self.errors.push(SemError {
                                code: "E037".to_string(),
                                message: format!(
                                    "Operador lógico requiere booleanos, no '{:?}' y '{:?}'",
                                    lt, rt
                                ),
                                span: *span,
                                suggestion: "Ambos operandos deben ser de tipo 'booleano'"
                                    .to_string(),
                            });
                        }
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
                    TypeInfo::Lista(Box::new(TypeInfo::Void))
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
            Expr::Index { expr, index, span } => {
                let expr_type = self.analyze_expr(expr);
                let index_type = self.analyze_expr(index);
                if index_type != TypeInfo::Entero {
                    self.errors.push(SemError {
                        code: "E043".to_string(),
                        message: format!("El índice debe ser entero, no '{:?}'", index_type),
                        span: *span,
                        suggestion: "Usa un valor de tipo 'entero' como índice".to_string(),
                    });
                }
                match expr_type {
                    TypeInfo::Lista(inner) => *inner,
                    _ => {
                        self.errors.push(SemError {
                            code: "E044".to_string(),
                            message: format!(
                                "No puedes indexar un valor de tipo '{:?}'",
                                expr_type
                            ),
                            span: *span,
                            suggestion: "La indexación solo funciona con listas".to_string(),
                        });
                        TypeInfo::Decimal
                    }
                }
            }
            Expr::MethodCall {
                expr,
                method,
                args,
                span,
            } => {
                let expr_type = self.analyze_expr(expr);
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
                                && *inner != arg_types[0]
                                && !(arg_types[0] == TypeInfo::Entero
                                    && *inner == TypeInfo::Decimal)
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
                    "largo" | "len" | "length" => match expr_type {
                        TypeInfo::Lista(_) => TypeInfo::Entero,
                        _ => {
                            self.errors.push(SemError {
                                code: "E047".to_string(),
                                message: format!(
                                    "No puedes llamar '{}' en un valor de tipo '{:?}'",
                                    method, expr_type
                                ),
                                span: *span,
                                suggestion: "'largo' solo se puede llamar en listas".to_string(),
                            });
                            TypeInfo::Entero
                        }
                    },
                    _ => {
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
                match &expr_type {
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
                                expr_type
                            ),
                            span: *span,
                            suggestion: "Solo los structs tienen campos".to_string(),
                        });
                        TypeInfo::Void
                    }
                }
            }
            Expr::Grouping { expr, .. } => self.analyze_expr(expr),
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

    fn current_scope(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }
}

impl SemanticAnalyzer {
    fn resolve_type(&self, t: Type, type_params: &[String]) -> TypeInfo {
        match t {
            Type::Struct(ref name) if type_params.contains(name) => TypeInfo::TypeVar(name.clone()),
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
            Type::Numero => TypeInfo::Decimal,
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
        }
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
        let errors = analyze(r#"numero x = "hola";"#);
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
        let errors = analyze(r#"numero x = 1 + "hola";"#);
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
        let errors = analyze(r#"booleano b = "a" < "b";"#);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E035");
    }

    #[test]
    fn test_comparison_type_error() {
        let errors = analyze(r#"booleano b = 1 < "hola";"#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_logical_type_error() {
        let errors = analyze("booleano b = verdadero && 1;");
        assert!(!errors.is_empty());
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
            r#"funcion numero suma(numero a, numero b) { retornar a + b; } suma(1, "hola");"#;
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
}
