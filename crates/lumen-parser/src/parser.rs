use crate::ast::*;
use crate::error::ParseError;
use lumen_lexer::token::{Span, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    no_struct_init: bool,
    type_params_stack: Vec<Vec<String>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            no_struct_init: false,
            type_params_stack: Vec::new(),
        }
    }

    pub fn parse(mut self) -> (Program, Vec<ParseError>) {
        let mut program = Vec::new();
        while !self.is_at_end() {
            if self.check(&[TokenKind::Eof]) {
                break;
            }
            match self.parse_decl_or_stmt() {
                Some(node) => program.push(node),
                None => {
                    if !self.is_at_end() && !self.check(&[TokenKind::Eof]) {
                        self.synchronize();
                    }
                }
            }
        }
        (program, self.errors)
    }

    fn parse_decl_or_stmt(&mut self) -> Option<DeclOrStmt> {
        if self.check(&[
            TokenKind::Numero,
            TokenKind::Entero,
            TokenKind::Decimal,
            TokenKind::Texto,
            TokenKind::Booleano,
            TokenKind::Lista,
            TokenKind::Array,
            TokenKind::Resultado,
            TokenKind::Result,
            TokenKind::Opcion,
            TokenKind::Option,
            TokenKind::Number,
            TokenKind::Integer,
            TokenKind::Float,
            TokenKind::String,
            TokenKind::Boolean,
        ]) || (self.check_ident() && self.check_ident_next())
            || self.check_next_is_tuple_type()
            || self.check_ident_next_is_generic_type()
        {
            self.parse_declaration().map(DeclOrStmt::Decl)
        } else if self.check(&[TokenKind::Funcion, TokenKind::Function]) {
            if self.check_next(&[
                TokenKind::Numero,
                TokenKind::Entero,
                TokenKind::Decimal,
                TokenKind::Texto,
                TokenKind::Booleano,
                TokenKind::Lista,
                TokenKind::Array,
                TokenKind::Resultado,
                TokenKind::Result,
                TokenKind::Opcion,
                TokenKind::Option,
                TokenKind::Number,
                TokenKind::Integer,
                TokenKind::Float,
                TokenKind::String,
                TokenKind::Boolean,
            ]) {
                self.parse_function().map(DeclOrStmt::Decl)
            } else if self.check_next(&[TokenKind::LeftParen]) {
                self.parse_expr_or_assign().map(DeclOrStmt::Stmt)
            } else {
                self.parse_function().map(DeclOrStmt::Decl)
            }
        } else if self.check(&[TokenKind::Si, TokenKind::If]) {
            self.parse_if().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Mientras, TokenKind::While]) {
            self.parse_while().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Para, TokenKind::For]) {
            if self.check_next(&[TokenKind::LeftParen]) {
                self.parse_for().map(DeclOrStmt::Stmt)
            } else {
                self.parse_foreach().map(DeclOrStmt::Stmt)
            }
        } else if self.check(&[TokenKind::Retornar, TokenKind::Return]) {
            self.parse_return().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Romper, TokenKind::Break]) {
            self.parse_break().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Continuar, TokenKind::Continue]) {
            self.parse_continue().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Elegir, TokenKind::Match]) {
            self.parse_match().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Estructura, TokenKind::Struct]) {
            self.parse_struct_decl().map(DeclOrStmt::Decl)
        } else if self.check(&[TokenKind::Enum]) {
            self.parse_enum().map(DeclOrStmt::Decl)
        } else if self.check(&[TokenKind::Importar, TokenKind::Import]) {
            self.parse_import().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::LeftBrace]) {
            self.parse_block_stmt().map(DeclOrStmt::Stmt)
        } else {
            self.parse_expr_or_assign().map(DeclOrStmt::Stmt)
        }
    }

    fn parse_declaration(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        let var_type = self.parse_type()?;
        let name = self.expect_ident()?;

        if self.check(&[TokenKind::Comma]) {
            return self.parse_destructure_decl(var_type, name, start);
        }

        let init = if self.check(&[TokenKind::Equal]) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.expect_semicolon();
        Some(Decl::Variable {
            var_type,
            name,
            init,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_destructure_decl(
        &mut self,
        first_type: Type,
        first_name: String,
        start: Span,
    ) -> Option<Decl> {
        let mut targets = Vec::new();
        targets.push(DestructureTarget {
            var_type: Some(first_type),
            name: first_name,
            span: Span::merge(&start, &start),
        });

        loop {
            self.advance();
            let t_start = self.peek().span;
            if self.check_ident() && self.peek_ident_is("_") {
                self.advance();
                targets.push(DestructureTarget {
                    var_type: None,
                    name: "_".to_string(),
                    span: Span::merge(&t_start, &self.previous().span),
                });
            } else {
                let t_type = self.parse_type()?;
                let t_name = self.expect_ident()?;
                targets.push(DestructureTarget {
                    var_type: Some(t_type),
                    name: t_name,
                    span: Span::merge(&t_start, &self.previous().span),
                });
            }
            if !self.check(&[TokenKind::Comma]) {
                break;
            }
        }

        if !self.check(&[TokenKind::Equal]) {
            self.error(
                "E012",
                "Se esperaba '=' para la destructuración",
                start,
                "Agrega '=' después de las variables",
            );
            return None;
        }
        self.advance();
        let init = Box::new(self.parse_expression()?);
        self.expect_semicolon();
        Some(Decl::Destructure {
            targets,
            init,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_function(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance();
        let return_type = self.parse_type()?;
        let name = self.expect_ident()?;

        let type_params = self.parse_type_params();

        if !self.check(&[TokenKind::LeftParen]) {
            self.error(
                "E014",
                "Se esperaba '('",
                start,
                "Agrega '(' para iniciar la lista de parámetros",
            );
            return None;
        }
        self.advance();

        let mut params = Vec::new();
        if !self.check(&[TokenKind::RightParen]) {
            params.push(self.parse_param()?);
            while self.check(&[TokenKind::Comma]) {
                self.advance();
                params.push(self.parse_param()?);
            }
        }
        if !self.check(&[TokenKind::RightParen]) {
            self.error(
                "E015",
                "Se esperaba ')'",
                start,
                "Agrega ')' para cerrar la lista de parámetros",
            );
            return None;
        }
        self.advance();

        // Push type params into stack for body parsing
        let saved_type_params = self.type_params_stack.clone();
        if !type_params.is_empty() {
            self.type_params_stack.push(type_params.clone());
        }
        let body = self.parse_block()?;
        if !type_params.is_empty() {
            self.type_params_stack.pop();
        }
        self.type_params_stack = saved_type_params;
        Some(Decl::Function {
            return_type,
            name,
            params,
            body,
            type_params,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_struct_decl(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance();
        let name = self.expect_ident()?;

        let type_params = self.parse_type_params();

        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E017",
                "Se esperaba '{' para la estructura",
                start,
                "Agrega '{' para definir los campos",
            );
            return None;
        }
        self.advance();

        let mut fields = Vec::new();
        while !self.check(&[TokenKind::RightBrace]) && !self.is_at_end() {
            if self.check(&[TokenKind::Eof]) {
                break;
            }

            let field_start = self.peek().span;
            let field_name = self.expect_field_name()?;

            if !self.check(&[TokenKind::Colon]) {
                self.error(
                    "E052",
                    "Se esperaba ':' después del nombre del campo",
                    self.peek().span,
                    "Agrega ':' después del nombre del campo",
                );
                return None;
            }
            self.advance();

            let field_type = self.parse_type()?;

            fields.push(StructField {
                field_type,
                name: field_name,
                span: Span::merge(&field_start, &self.previous().span),
            });

            if self.check(&[TokenKind::Comma]) {
                self.advance();
            } else if !self.check(&[TokenKind::RightBrace]) {
                self.error(
                    "E012",
                    "Se esperaba ',' o '}' para cerrar la estructura",
                    self.peek().span,
                    "Agrega ',' entre campos o '}' para cerrar",
                );
                return None;
            }
        }

        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E017",
                "Se esperaba '}' para cerrar la estructura",
                start,
                "Agrega '}' al final de la estructura",
            );
            return None;
        }
        self.advance();

        Some(Decl::Struct {
            name,
            fields,
            type_params,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_enum(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance();
        let name = self.expect_ident()?;

        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E017",
                "Se esperaba '{' para la enumeración",
                start,
                "Agrega '{' para definir las variantes",
            );
            return None;
        }
        self.advance();

        let mut variants = Vec::new();
        while !self.check(&[TokenKind::RightBrace]) && !self.is_at_end() {
            if self.check(&[TokenKind::Eof]) {
                break;
            }
            let var_start = self.peek().span;
            let var_name = self.expect_ident()?;
            let var_types = if self.check(&[TokenKind::LeftParen]) {
                self.advance();
                let mut types = Vec::new();
                if !self.check(&[TokenKind::RightParen]) {
                    types.push(self.parse_type()?);
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        types.push(self.parse_type()?);
                    }
                }
                if !self.check(&[TokenKind::RightParen]) {
                    self.error(
                        "E015",
                        "Se esperaba ')'",
                        var_start,
                        "Agrega ')' para cerrar los tipos de la variante",
                    );
                    return None;
                }
                self.advance();
                types
            } else {
                Vec::new()
            };
            variants.push(EnumVariant {
                name: var_name,
                types: var_types,
                span: Span::merge(&var_start, &self.previous().span),
            });
            if self.check(&[TokenKind::Comma]) {
                self.advance();
            } else if !self.check(&[TokenKind::RightBrace]) {
                self.error(
                    "E012",
                    "Se esperaba ',' o '}' para cerrar la enumeración",
                    self.peek().span,
                    "Agrega ',' entre variantes o '}' para cerrar",
                );
                return None;
            }
        }
        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E017",
                "Se esperaba '}' para cerrar la enumeración",
                start,
                "Agrega '}' al final de la enumeración",
            );
            return None;
        }
        self.advance();
        Some(Decl::Enum {
            name,
            variants,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_import(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance(); // consume importar/import
        let path = if let TokenKind::StrLiteral(s) = &self.peek().kind {
            let s = s.clone();
            self.advance();
            s
        } else if self.check_ident() {
            let token = self.advance()?;
            match token.kind {
                TokenKind::Ident(s) => s,
                _ => unreachable!(),
            }
        } else {
            self.error(
                "E011",
                "Se esperaba una ruta de archivo o nombre de módulo",
                self.peek().span,
                "Escribe \"archivo.nv\" o nombre_del_modulo",
            );
            return None;
        };
        let alias = if self.check(&[TokenKind::Como, TokenKind::As]) {
            self.advance();
            if self.check_ident() {
                let token = self.advance()?;
                match token.kind {
                    TokenKind::Ident(s) => Some(s),
                    _ => unreachable!(),
                }
            } else {
                self.error(
                    "E011",
                    "Se esperaba un nombre de alias",
                    self.peek().span,
                    "Escribe un identificador como alias",
                );
                None
            }
        } else {
            None
        };
        self.expect_semicolon();
        Some(Stmt::Import {
            path,
            alias,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_destructure_assign_stmt(&mut self, start: Span) -> Option<Stmt> {
        let mut targets = Vec::new();
        loop {
            let t_start = self.peek().span;
            if self.check_ident() {
                let name = match &self.peek().kind {
                    TokenKind::Ident(s) => s.clone(),
                    _ => unreachable!(),
                };
                self.advance();
                targets.push(DestructureTarget {
                    var_type: None,
                    name,
                    span: Span::merge(&t_start, &self.previous().span),
                });
            } else {
                self.error(
                    "E011",
                    "Se esperaba un identificador en la destructuración",
                    self.peek().span,
                    "Escribe un nombre de variable",
                );
                return None;
            }
            if !self.check(&[TokenKind::Comma]) {
                break;
            }
            self.advance();
        }

        if !self.check(&[TokenKind::Equal]) {
            self.error(
                "E012",
                "Se esperaba '=' para la destructuración",
                start,
                "Agrega '=' después de las variables",
            );
            return None;
        }
        self.advance();
        let value = Box::new(self.parse_expression()?);
        self.expect_semicolon();
        Some(Stmt::Destructure {
            targets,
            value,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_param(&mut self) -> Option<Param> {
        let start = self.peek().span;
        let param_type = self.parse_type()?;
        let name = self.expect_ident()?;
        let default = if self.check(&[TokenKind::Equal]) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Some(Param {
            param_type,
            name,
            default,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_block(&mut self) -> Option<Vec<DeclOrStmt>> {
        let mut stmts = Vec::new();
        if !self.check(&[TokenKind::LeftBrace]) {
            return None;
        }
        self.advance();
        while !self.check(&[TokenKind::RightBrace]) && !self.is_at_end() {
            if self.check(&[TokenKind::Eof]) {
                break;
            }
            match self.parse_decl_or_stmt() {
                Some(node) => stmts.push(node),
                None => {
                    self.synchronize();
                }
            }
        }
        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E017",
                "Se esperaba '}'",
                self.previous().span,
                "Agrega '}' para cerrar el bloque",
            );
            return Some(stmts);
        }
        self.advance();
        Some(stmts)
    }

    fn parse_block_stmt(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        let stmts = self.parse_block()?;
        Some(Stmt::Block {
            stmts,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_if(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        if !self.check(&[TokenKind::LeftParen]) {
            self.error(
                "E018",
                "Se esperaba '('",
                start,
                "Agrega '(' antes de la condición",
            );
            return None;
        }
        self.advance();
        let condition = Box::new(self.parse_expression()?);
        if !self.check(&[TokenKind::RightParen]) {
            self.error(
                "E019",
                "Se esperaba ')'",
                start,
                "Agrega ')' después de la condición",
            );
            return None;
        }
        self.advance();
        let then_body = self.parse_block()?;
        let else_body = if self.check(&[TokenKind::Sino, TokenKind::Else]) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Some(Stmt::If {
            condition,
            then_body,
            else_body,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        if !self.check(&[TokenKind::LeftParen]) {
            return None;
        }
        self.advance();
        let condition = Box::new(self.parse_expression()?);
        if !self.check(&[TokenKind::RightParen]) {
            return None;
        }
        self.advance();
        let body = self.parse_block()?;
        Some(Stmt::While {
            condition,
            body,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_for(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        if !self.check(&[TokenKind::LeftParen]) {
            return None;
        }
        self.advance();
        let init = Box::new(self.parse_declaration()?);
        let condition = Box::new(self.parse_expression()?);
        if !self.check(&[TokenKind::Semicolon]) {
            return None;
        }
        self.advance();
        let update = Box::new(self.parse_assignment()?);
        if !self.check(&[TokenKind::RightParen]) {
            return None;
        }
        self.advance();
        let body = self.parse_block()?;
        Some(Stmt::For {
            init,
            condition,
            update,
            body,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_foreach(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        let var_name = self.expect_ident()?;
        if !self.check(&[TokenKind::En, TokenKind::In]) {
            self.error(
                "E025",
                "Se esperaba 'en'/'in' después del nombre de variable en el ciclo para-cada",
                self.peek().span,
                "Agrega 'en' después del nombre de la variable",
            );
            return None;
        }
        self.advance();
        let saved = self.no_struct_init;
        self.no_struct_init = true;
        let expr = Box::new(self.parse_expression()?);
        self.no_struct_init = saved;
        let body = self.parse_block()?;
        Some(Stmt::ForEach {
            var_name,
            expr,
            body,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_return(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        let value = if !self.check(&[TokenKind::Semicolon]) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.expect_semicolon();
        Some(Stmt::Return {
            value,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_break(&mut self) -> Option<Stmt> {
        let token = self.advance()?;
        let span = token.span;
        self.expect_semicolon();
        Some(Stmt::Break { span })
    }

    fn parse_continue(&mut self) -> Option<Stmt> {
        let token = self.advance()?;
        let span = token.span;
        self.expect_semicolon();
        Some(Stmt::Continue { span })
    }

    fn parse_match(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();

        if !self.check(&[TokenKind::LeftParen]) {
            self.error(
                "E051",
                "Se esperaba '(' después de 'elegir'",
                start,
                "Agrega '(' para iniciar la expresión",
            );
            return None;
        }
        self.advance();

        let expr = self.parse_expression()?;

        if !self.check(&[TokenKind::RightParen]) {
            self.error(
                "E015",
                "Se esperaba ')'",
                start,
                "Agrega ')' después de la expresión",
            );
            return None;
        }
        self.advance();

        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E017",
                "Se esperaba '{' para el bloque de elegir",
                start,
                "Agrega '{' para iniciar los casos",
            );
            return None;
        }
        self.advance();

        let mut arms = Vec::new();
        let mut default = None;

        while !self.check(&[TokenKind::RightBrace]) && !self.is_at_end() {
            if self.check(&[TokenKind::Eof]) {
                break;
            }

            if self.check(&[TokenKind::Defecto, TokenKind::Default]) {
                let def_start = self.peek().span;
                self.advance();

                if !self.check(&[TokenKind::Colon]) {
                    self.error(
                        "E052",
                        "Se esperaba ':' después de 'defecto'",
                        def_start,
                        "Agrega ':' después de 'defecto'",
                    );
                    return Some(Stmt::Match {
                        expr: Box::new(expr),
                        arms,
                        default,
                        span: Span::merge(&start, &def_start),
                    });
                }
                self.advance();

                let mut body = Vec::new();
                while !self.check(&[
                    TokenKind::RightBrace,
                    TokenKind::Caso,
                    TokenKind::Case,
                    TokenKind::Defecto,
                    TokenKind::Default,
                ]) && !self.is_at_end()
                {
                    if self.check(&[TokenKind::Eof]) {
                        break;
                    }
                    match self.parse_decl_or_stmt() {
                        Some(node) => body.push(node),
                        None => {
                            self.synchronize();
                        }
                    }
                }
                default = Some(body);
                break;
            } else if self.check(&[TokenKind::Caso, TokenKind::Case]) {
                let arm_start = self.peek().span;
                self.advance();

                let value = self.parse_expression()?;

                if !self.check(&[TokenKind::Colon]) {
                    self.error(
                        "E052",
                        "Se esperaba ':' después del valor del caso",
                        arm_start,
                        "Agrega ':' después del valor",
                    );
                    return Some(Stmt::Match {
                        expr: Box::new(expr),
                        arms,
                        default,
                        span: Span::merge(&start, &arm_start),
                    });
                }
                self.advance();

                let mut body = Vec::new();
                while !self.check(&[
                    TokenKind::RightBrace,
                    TokenKind::Caso,
                    TokenKind::Case,
                    TokenKind::Defecto,
                    TokenKind::Default,
                ]) && !self.is_at_end()
                {
                    if self.check(&[TokenKind::Eof]) {
                        break;
                    }
                    match self.parse_decl_or_stmt() {
                        Some(node) => body.push(node),
                        None => {
                            self.synchronize();
                        }
                    }
                }
                arms.push(MatchArm {
                    value,
                    body,
                    span: Span::merge(&arm_start, &self.previous().span),
                });
            } else {
                self.error(
                    "E053",
                    "Se esperaba 'caso' o 'defecto' dentro de elegir",
                    self.peek().span,
                    "Usa 'caso' seguido de un valor y ':'",
                );
                self.advance();
            }
        }

        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E017",
                "Se esperaba '}' para cerrar elegir",
                start,
                "Agrega '}' al final",
            );
            return Some(Stmt::Match {
                expr: Box::new(expr),
                arms,
                default,
                span: Span::merge(&start, &self.previous().span),
            });
        }
        self.advance();

        Some(Stmt::Match {
            expr: Box::new(expr),
            arms,
            default,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_assignment(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        let name = match &self.peek().kind {
            TokenKind::Ident(s) => s.clone(),
            _ => return None,
        };
        self.advance();
        if !self.check(&[TokenKind::Equal]) {
            return None;
        }
        self.advance();
        let value = Box::new(self.parse_expression()?);
        Some(Stmt::Assignment {
            name,
            value,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_expr_or_assign(&mut self) -> Option<Stmt> {
        let start = self.peek().span;

        if self.check_next_comma_and_ident() {
            return self.parse_destructure_assign_stmt(start);
        }

        if self.check_ident() && self.check_next(&[TokenKind::Equal]) {
            let name = match self.advance() {
                Some(t) => match t.kind {
                    TokenKind::Ident(s) => s,
                    _ => unreachable!(),
                },
                None => return None,
            };
            self.advance();
            let value = Box::new(self.parse_expression()?);
            self.expect_semicolon();
            Some(Stmt::Assignment {
                name,
                value,
                span: Span::merge(&start, &self.previous().span),
            })
        } else if self.check_ident() && self.check_next(&[TokenKind::Dot]) {
            let expr = self.parse_expression()?;
            if self.check(&[TokenKind::Equal]) {
                self.advance();
                let value = Box::new(self.parse_expression()?);
                self.expect_semicolon();
                match expr {
                    Expr::FieldAccess {
                        expr: target,
                        field,
                        ..
                    } => Some(Stmt::FieldAssign {
                        expr: target,
                        field,
                        value,
                        span: Span::merge(&start, &self.previous().span),
                    }),
                    _ => {
                        self.error(
                            "E024",
                            "No se puede asignar a esta expresión",
                            start,
                            "Solo se puede asignar a campos de struct",
                        );
                        None
                    }
                }
            } else {
                self.expect_semicolon();
                Some(Stmt::Expr {
                    expr: Box::new(expr),
                    span: Span::merge(&start, &self.previous().span),
                })
            }
        } else {
            let expr = self.parse_expression()?;
            self.expect_semicolon();
            Some(Stmt::Expr {
                expr: Box::new(expr),
                span: Span::merge(&start, &self.previous().span),
            })
        }
    }

    // --- Pratt Parser for Expressions ---

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Option<Expr> {
        let mut left = self.parse_logical_and()?;
        while self.check(&[TokenKind::OrOr]) {
            self.advance();
            let right = self.parse_logical_and()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Some(left)
    }

    fn parse_logical_and(&mut self) -> Option<Expr> {
        let mut left = self.parse_comparison()?;
        while self.check(&[TokenKind::AndAnd]) {
            self.advance();
            let right = self.parse_comparison()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Some(left)
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        let mut left = self.parse_addition()?;
        while self.check(&[
            TokenKind::EqualEqual,
            TokenKind::BangEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let op = match self.peek().kind {
                TokenKind::EqualEqual => BinOp::Equal,
                TokenKind::BangEqual => BinOp::NotEqual,
                TokenKind::Less => BinOp::Less,
                TokenKind::LessEqual => BinOp::LessEqual,
                TokenKind::Greater => BinOp::Greater,
                TokenKind::GreaterEqual => BinOp::GreaterEqual,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_addition()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Some(left)
    }

    fn parse_addition(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplication()?;
        while self.check(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = match self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplication()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Some(left)
    }

    fn parse_multiplication(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;
        while self.check(&[TokenKind::Star, TokenKind::Slash]) {
            let op = match self.peek().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        if self.check(&[TokenKind::Minus, TokenKind::Bang]) {
            let op = match self.peek().kind {
                TokenKind::Minus => UnOp::Negate,
                TokenKind::Bang => UnOp::Not,
                _ => unreachable!(),
            };
            let op_span = self.peek().span;
            self.advance();
            let operand = self.parse_unary()?;
            let span = Span::merge(&op_span, &operand.span());
            Some(Expr::Unary {
                op,
                operand: Box::new(operand),
                span,
            })
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&[TokenKind::LeftBracket]) {
                let start = expr.span();
                self.advance();
                let index = self.parse_expression()?;
                if !self.check(&[TokenKind::RightBracket]) {
                    self.error(
                        "E023",
                        "Se esperaba ']' para cerrar el índice",
                        start,
                        "Agrega ']' después del índice",
                    );
                    return Some(expr);
                }
                self.advance();
                let span = Span::merge(&start, &self.previous().span);
                expr = Expr::Index {
                    expr: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else if self.check(&[TokenKind::Dot]) {
                let start = expr.span();
                self.advance();
                let token = self.advance();
                match token {
                    Some(t) => {
                        match t.kind {
                            TokenKind::Ident(s) => {
                                if self.check(&[TokenKind::LeftParen]) {
                                    self.advance();
                                    let mut args = Vec::new();
                                    if !self.check(&[TokenKind::RightParen]) {
                                        args.push(self.parse_expression()?);
                                        while self.check(&[TokenKind::Comma]) {
                                            self.advance();
                                            args.push(self.parse_expression()?);
                                        }
                                    }
                                    if !self.check(&[TokenKind::RightParen]) {
                                        self.error(
                                            "E015",
                                            "Se esperaba ')'",
                                            start,
                                            "Agrega ')' para cerrar la llamada al método",
                                        );
                                        return Some(expr);
                                    }
                                    self.advance();
                                    let span = Span::merge(&start, &self.previous().span);
                                    expr = Expr::MethodCall {
                                        expr: Box::new(expr),
                                        method: s,
                                        args,
                                        span,
                                    };
                                } else {
                                    let span = Span::merge(&start, &self.previous().span);
                                    expr = Expr::FieldAccess {
                                        expr: Box::new(expr),
                                        field: s,
                                        span,
                                    };
                                }
                            }
                            TokenKind::NumLiteral(n) => {
                                let span = Span::merge(&start, &self.previous().span);
                                if let Some(dot_pos) = n.find('.') {
                                    let int_part: usize = n[..dot_pos].parse().unwrap_or(0);
                                    let frac_str = &n[dot_pos + 1..];
                                    expr = Expr::TupleAccess {
                                        expr: Box::new(expr),
                                        index: int_part,
                                        span,
                                    };
                                    if !frac_str.is_empty() {
                                        let frac_val: usize = frac_str.parse().unwrap_or(0);
                                        let frac_span = Span::merge(&span, &span);
                                        expr = Expr::TupleAccess {
                                            expr: Box::new(expr),
                                            index: frac_val,
                                            span: frac_span,
                                        };
                                    }
                                } else {
                                    let index: usize = n.parse().unwrap_or(0);
                                    expr = Expr::TupleAccess {
                                        expr: Box::new(expr),
                                        index,
                                        span,
                                    };
                                }
                            }
                            _ => {
                                self.error("E024", "Se esperaba un nombre de campo o índice numérico después de '.'", t.span, "Escribe el nombre del campo o un número");
                                return Some(expr);
                            }
                        }
                    }
                    None => return Some(expr),
                }
            } else if self.check(&[TokenKind::LeftParen]) {
                let start = expr.span();
                self.advance();
                let mut args = Vec::new();
                if !self.check(&[TokenKind::RightParen]) {
                    args.push(self.parse_expression()?);
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                }
                if !self.check(&[TokenKind::RightParen]) {
                    self.error(
                        "E015",
                        "Se esperaba ')'",
                        start,
                        "Agrega ')' para cerrar la llamada",
                    );
                    return Some(expr);
                }
                self.advance();
                let span = Span::merge(&start, &self.previous().span);
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    type_args: Vec::new(),
                    span,
                };
            } else {
                break;
            }
        }
        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let token = self.advance()?;
        let span = token.span;
        match &token.kind {
            TokenKind::NumLiteral(s) => {
                if s.contains('.') {
                    let value: f64 = s.parse().unwrap_or(0.0);
                    Some(Expr::Float { value, span })
                } else {
                    let value: i64 = s.parse().unwrap_or(0);
                    Some(Expr::Int { value, span })
                }
            }
            TokenKind::StrLiteral(s) => Some(Expr::Str {
                value: s.clone(),
                span,
            }),
            TokenKind::Verdadero | TokenKind::True => Some(Expr::Bool { value: true, span }),
            TokenKind::Falso | TokenKind::False => Some(Expr::Bool { value: false, span }),
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.parse_call_or_ident(name, span)
            }
            TokenKind::Funcion | TokenKind::Function => self.parse_lambda(span),
            TokenKind::Imprimir | TokenKind::Print | TokenKind::Leer | TokenKind::Read => {
                let name = match token.kind {
                    TokenKind::Imprimir => "imprimir",
                    TokenKind::Print => "print",
                    TokenKind::Leer => "leer",
                    TokenKind::Read => "read",
                    _ => unreachable!(),
                };
                self.parse_call_or_ident(name.to_string(), span)
            }
            TokenKind::Exito | TokenKind::Ok => {
                if !self.check(&[TokenKind::LeftParen]) {
                    self.error(
                        "E014",
                        "Se esperaba '(' después de 'exito'",
                        span,
                        "Agrega '(expr)' para el valor de éxito",
                    );
                    return None;
                }
                self.advance();
                let expr = self.parse_expression()?;
                if !self.check(&[TokenKind::RightParen]) {
                    self.error(
                        "E015",
                        "Se esperaba ')'",
                        span,
                        "Agrega ')' para cerrar el valor de éxito",
                    );
                    return None;
                }
                self.advance();
                Some(Expr::Exito {
                    expr: Box::new(expr),
                    span: Span::merge(&span, &self.previous().span),
                })
            }
            TokenKind::ErrKeyword | TokenKind::Err => {
                if !self.check(&[TokenKind::LeftParen]) {
                    self.error(
                        "E014",
                        "Se esperaba '(' después de 'error'",
                        span,
                        "Agrega '(expr)' para el valor de error",
                    );
                    return None;
                }
                self.advance();
                let expr = self.parse_expression()?;
                if !self.check(&[TokenKind::RightParen]) {
                    self.error(
                        "E015",
                        "Se esperaba ')'",
                        span,
                        "Agrega ')' para cerrar el valor de error",
                    );
                    return None;
                }
                self.advance();
                Some(Expr::Error {
                    expr: Box::new(expr),
                    span: Span::merge(&span, &self.previous().span),
                })
            }
            TokenKind::Intentar | TokenKind::Try => {
                let expr_val = self.parse_expression()?;
                let end_span = expr_val.span();
                Some(Expr::Intentar {
                    expr: Box::new(expr_val),
                    span: Span::merge(&span, &end_span),
                })
            }
            TokenKind::Algun | TokenKind::Some => {
                if !self.check(&[TokenKind::LeftParen]) {
                    self.error(
                        "E014",
                        "Se esperaba '(' después de 'algun'",
                        span,
                        "Agrega '(expr)' para el valor",
                    );
                    return None;
                }
                self.advance();
                let expr = self.parse_expression()?;
                if !self.check(&[TokenKind::RightParen]) {
                    self.error("E015", "Se esperaba ')'", span, "Agrega ')' para cerrar");
                    return None;
                }
                self.advance();
                Some(Expr::Algun {
                    expr: Box::new(expr),
                    span: Span::merge(&span, &self.previous().span),
                })
            }
            TokenKind::Ninguno | TokenKind::None => Some(Expr::Ninguno { span }),
            TokenKind::LeftParen => {
                let first = self.parse_expression()?;
                if self.check(&[TokenKind::Comma]) {
                    let mut items = vec![first];
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        if self.check(&[TokenKind::RightParen]) {
                            break;
                        }
                        items.push(self.parse_expression()?);
                    }
                    if !self.check(&[TokenKind::RightParen]) {
                        self.error(
                            "E015",
                            "Se esperaba ')' para cerrar la tupla",
                            span,
                            "Agrega ')' después de los elementos",
                        );
                        return None;
                    }
                    self.advance();
                    Some(Expr::Tuple {
                        items,
                        span: Span::merge(&span, &self.previous().span),
                    })
                } else {
                    if !self.check(&[TokenKind::RightParen]) {
                        self.error(
                            "E015",
                            "Se esperaba ')'",
                            span,
                            "Agrega ')' para cerrar el paréntesis",
                        );
                        return None;
                    }
                    self.advance();
                    Some(Expr::Grouping {
                        expr: Box::new(first),
                        span: Span::merge(&span, &self.previous().span),
                    })
                }
            }
            TokenKind::LeftBracket => {
                let mut items = Vec::new();
                if !self.check(&[TokenKind::RightBracket]) {
                    items.push(self.parse_expression()?);
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        items.push(self.parse_expression()?);
                    }
                }
                if !self.check(&[TokenKind::RightBracket]) {
                    self.error(
                        "E022",
                        "Se esperaba ']' para cerrar la lista",
                        span,
                        "Agrega ']' al final de la lista",
                    );
                    return None;
                }
                self.advance();
                Some(Expr::List {
                    items,
                    span: Span::merge(&span, &self.previous().span),
                })
            }
            _ => {
                self.error(
                    "E020",
                    format!("Expresión inesperada: {:?}", token.kind),
                    span,
                    "Revisa la sintaxis de la expresión",
                );
                None
            }
        }
    }

    fn parse_lambda(&mut self, span: Span) -> Option<Expr> {
        if !self.check(&[TokenKind::LeftParen]) {
            self.error(
                "E014",
                "Se esperaba '(' en la función anónima",
                span,
                "Agrega '(' para iniciar los parámetros",
            );
            return None;
        }
        self.advance();
        let mut params = Vec::new();
        if !self.check(&[TokenKind::RightParen]) {
            params.push(self.parse_param()?);
            while self.check(&[TokenKind::Comma]) {
                self.advance();
                params.push(self.parse_param()?);
            }
        }
        if !self.check(&[TokenKind::RightParen]) {
            self.error(
                "E015",
                "Se esperaba ')'",
                span,
                "Agrega ')' para cerrar los parámetros",
            );
            return None;
        }
        self.advance();
        let body = self.parse_block()?;
        Some(Expr::Lambda {
            params,
            body,
            span: Span::merge(&span, &self.previous().span),
        })
    }

    fn parse_call_or_ident(&mut self, name: String, span: Span) -> Option<Expr> {
        if self.check(&[TokenKind::DoubleColon]) {
            self.advance();
            let variant = self.expect_ident()?;
            let args = if self.check(&[TokenKind::LeftParen]) {
                self.advance();
                let mut args = Vec::new();
                if !self.check(&[TokenKind::RightParen]) {
                    args.push(self.parse_expression()?);
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                }
                if !self.check(&[TokenKind::RightParen]) {
                    self.error(
                        "E015",
                        "Se esperaba ')'",
                        span,
                        "Agrega ')' para cerrar los argumentos",
                    );
                    return None;
                }
                self.advance();
                args
            } else {
                Vec::new()
            };
            Some(Expr::EnumCtor {
                enum_name: name,
                variant,
                args,
                span: Span::merge(&span, &self.previous().span),
            })
        } else if self.check(&[TokenKind::LeftParen]) {
            self.advance();
            let mut args = Vec::new();
            if !self.check(&[TokenKind::RightParen]) {
                args.push(self.parse_expression()?);
                while self.check(&[TokenKind::Comma]) {
                    self.advance();
                    args.push(self.parse_expression()?);
                }
            }
            if !self.check(&[TokenKind::RightParen]) {
                self.error(
                    "E015",
                    "Se esperaba ')'",
                    span,
                    "Agrega ')' para cerrar la llamada",
                );
                return None;
            }
            self.advance();
            Some(Expr::Call {
                callee: Box::new(Expr::Ident { name, span }),
                args,
                type_args: Vec::new(),
                span: Span::merge(&span, &self.previous().span),
            })
        } else if self.check(&[TokenKind::Less]) && self.is_type_arg_start() {
            let type_args = self.parse_type_args()?;
            if self.check(&[TokenKind::LeftParen]) {
                self.advance();
                let mut args = Vec::new();
                if !self.check(&[TokenKind::RightParen]) {
                    args.push(self.parse_expression()?);
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        args.push(self.parse_expression()?);
                    }
                }
                if !self.check(&[TokenKind::RightParen]) {
                    self.error(
                        "E015",
                        "Se esperaba ')'",
                        span,
                        "Agrega ')' para cerrar la llamada",
                    );
                    return None;
                }
                self.advance();
                Some(Expr::Call {
                    callee: Box::new(Expr::Ident { name, span }),
                    args,
                    type_args,
                    span: Span::merge(&span, &self.previous().span),
                })
            } else if self.check(&[TokenKind::LeftBrace]) && !self.no_struct_init {
                self.advance();
                let mut fields = Vec::new();
                while !self.check(&[TokenKind::RightBrace]) && !self.is_at_end() {
                    if self.check(&[TokenKind::Eof]) {
                        break;
                    }
                    let field_name = self.expect_field_name()?;
                    if !self.check(&[TokenKind::Colon]) {
                        self.error(
                            "E052",
                            "Se esperaba ':' después del nombre del campo",
                            self.peek().span,
                            "Agrega ':' después del nombre del campo",
                        );
                        return None;
                    }
                    self.advance();
                    let value = self.parse_expression()?;
                    fields.push((field_name, value));
                    if self.check(&[TokenKind::Comma]) {
                        self.advance();
                    } else if !self.check(&[TokenKind::RightBrace]) {
                        self.error(
                            "E012",
                            "Se esperaba ',' o '}'",
                            self.peek().span,
                            "Agrega ',' entre campos o '}' para cerrar",
                        );
                        return None;
                    }
                }
                if !self.check(&[TokenKind::RightBrace]) {
                    self.error(
                        "E022",
                        "Se esperaba '}' para cerrar la estructura",
                        span,
                        "Agrega '}' al final",
                    );
                    return None;
                }
                self.advance();
                Some(Expr::StructInit {
                    struct_name: name,
                    fields,
                    type_args,
                    span: Span::merge(&span, &self.previous().span),
                })
            } else {
                Some(Expr::Ident { name, span })
            }
        } else if self.check(&[TokenKind::LeftBrace]) && !self.no_struct_init {
            self.advance();
            let mut fields = Vec::new();
            while !self.check(&[TokenKind::RightBrace]) && !self.is_at_end() {
                if self.check(&[TokenKind::Eof]) {
                    break;
                }
                let field_name = self.expect_field_name()?;

                if !self.check(&[TokenKind::Colon]) {
                    self.error(
                        "E052",
                        "Se esperaba ':' después del nombre del campo",
                        self.peek().span,
                        "Agrega ':' después del nombre del campo",
                    );
                    return None;
                }
                self.advance();

                let value = self.parse_expression()?;
                fields.push((field_name, value));

                if self.check(&[TokenKind::Comma]) {
                    self.advance();
                } else if !self.check(&[TokenKind::RightBrace]) {
                    self.error(
                        "E012",
                        "Se esperaba ',' o '}'",
                        self.peek().span,
                        "Agrega ',' entre campos o '}' para cerrar",
                    );
                    return None;
                }
            }

            if !self.check(&[TokenKind::RightBrace]) {
                self.error(
                    "E022",
                    "Se esperaba '}' para cerrar la estructura",
                    span,
                    "Agrega '}' al final",
                );
                return None;
            }
            self.advance();

            Some(Expr::StructInit {
                struct_name: name,
                fields,
                type_args: Vec::new(),
                span: Span::merge(&span, &self.previous().span),
            })
        } else {
            Some(Expr::Ident { name, span })
        }
    }

    fn parse_type_params(&mut self) -> Vec<String> {
        if !self.check(&[TokenKind::Less]) {
            return Vec::new();
        }
        self.advance();
        let mut params = Vec::new();
        let token = self.advance();
        match token {
            Some(t) => match t.kind {
                TokenKind::Ident(name) => {
                    params.push(name);
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        let next = self.advance();
                        match next {
                            Some(t2) => match t2.kind {
                                TokenKind::Ident(s) => params.push(s),
                                _ => {
                                    self.error(
                                        "E011",
                                        "Se esperaba un identificador para el parámetro de tipo",
                                        t2.span,
                                        "Escribe un nombre de parámetro de tipo",
                                    );
                                    return params;
                                }
                            },
                            None => return params,
                        }
                    }
                }
                _ => {
                    self.error(
                        "E011",
                        "Se esperaba un identificador para el parámetro de tipo",
                        t.span,
                        "Escribe un nombre de parámetro de tipo",
                    );
                    return params;
                }
            },
            None => return params,
        }
        if !self.check(&[TokenKind::Greater]) {
            self.error(
                "E021",
                "Se esperaba '>' para cerrar los parámetros de tipo",
                self.peek().span,
                "Agrega '>' después de los parámetros de tipo",
            );
            return params;
        }
        self.advance();
        params
    }

    fn is_type_arg_start(&self) -> bool {
        if self.pos + 1 >= self.tokens.len() {
            return false;
        }
        let next = &self.tokens[self.pos + 1].kind;
        matches!(
            next,
            TokenKind::Numero
                | TokenKind::Number
                | TokenKind::Entero
                | TokenKind::Integer
                | TokenKind::Decimal
                | TokenKind::Float
                | TokenKind::Texto
                | TokenKind::String
                | TokenKind::Booleano
                | TokenKind::Boolean
                | TokenKind::Lista
                | TokenKind::Array
                | TokenKind::Resultado
                | TokenKind::Result
                | TokenKind::Opcion
                | TokenKind::Option
                | TokenKind::LeftParen
        ) || self.is_next_type_param()
    }

    fn is_next_type_param(&self) -> bool {
        if self.pos + 1 >= self.tokens.len() {
            return false;
        }
        let next = &self.tokens[self.pos + 1];
        match &next.kind {
            TokenKind::Ident(name) => {
                self.type_params_stack.iter().any(|params| params.contains(name))
            }
            _ => false,
        }
    }

    fn parse_type_args(&mut self) -> Option<Vec<Type>> {
        if !self.check(&[TokenKind::Less]) {
            return Some(Vec::new());
        }
        self.advance();
        let mut args = Vec::new();
        args.push(self.parse_type()?);
        while self.check(&[TokenKind::Comma]) {
            self.advance();
            args.push(self.parse_type()?);
        }
        if !self.check(&[TokenKind::Greater]) {
            self.error(
                "E021",
                "Se esperaba '>' para cerrar los argumentos de tipo",
                self.peek().span,
                "Agrega '>' después de los argumentos de tipo",
            );
            return None;
        }
        self.advance();
        Some(args)
    }

    /// Check if the token after next (at pos + 2) is a type keyword or known type param
    fn is_type_at(&self, idx: usize) -> bool {
        if idx >= self.tokens.len() {
            return false;
        }
        let kind = &self.tokens[idx].kind;
        let is_type_keyword = matches!(
            kind,
            TokenKind::Numero
                | TokenKind::Number
                | TokenKind::Entero
                | TokenKind::Integer
                | TokenKind::Decimal
                | TokenKind::Float
                | TokenKind::Texto
                | TokenKind::String
                | TokenKind::Booleano
                | TokenKind::Boolean
                | TokenKind::Lista
                | TokenKind::Array
                | TokenKind::Resultado
                | TokenKind::Result
                | TokenKind::Opcion
                | TokenKind::Option
                | TokenKind::LeftParen
        );
        if is_type_keyword {
            return true;
        }
        if let TokenKind::Ident(name) = kind {
            return self.type_params_stack.iter().any(|params| params.contains(name));
        }
        false
    }

    fn is_next_type_in_type_context(&self) -> bool {
        self.is_type_at(self.pos + 1)
    }

    fn parse_type(&mut self) -> Option<Type> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Numero | TokenKind::Number => Some(Type::Numero),
            TokenKind::Entero | TokenKind::Integer => Some(Type::Entero),
            TokenKind::Decimal | TokenKind::Float => Some(Type::Decimal),
            TokenKind::Texto | TokenKind::String => Some(Type::Texto),
            TokenKind::Booleano | TokenKind::Boolean => Some(Type::Booleano),
            TokenKind::Lista | TokenKind::Array => {
                if self.check(&[TokenKind::Less]) {
                    self.advance();
                    let inner = self.parse_type()?;
                    if !self.check(&[TokenKind::Greater]) {
                        self.error(
                            "E021",
                            "Se esperaba '>' para cerrar el tipo lista",
                            token.span,
                            "Agrega '>' después del tipo interno",
                        );
                        return None;
                    }
                    self.advance();
                    Some(Type::Lista(Box::new(inner)))
                } else {
                    Some(Type::Lista(Box::new(Type::Decimal)))
                }
            }
            TokenKind::Ident(name) => {
                if self.check(&[TokenKind::Less]) && self.is_next_type_in_type_context() {
                    let args = self.parse_type_args()?;
                    Some(Type::GenericStruct { name, args })
                } else {
                    Some(Type::Struct(name))
                }
            }
            TokenKind::Resultado | TokenKind::Result => {
                if !self.check(&[TokenKind::Less]) {
                    self.error(
                        "E021",
                        "Se esperaba '<' para el tipo resultado",
                        token.span,
                        "Agrega '<tipo_ok, tipo_err>' después de 'resultado'",
                    );
                    return None;
                }
                self.advance();
                let ok = self.parse_type()?;
                if !self.check(&[TokenKind::Comma]) {
                    self.error(
                        "E012",
                        "Se esperaba ',' entre tipos de resultado",
                        token.span,
                        "Agrega ',' para separar el tipo de éxito y error",
                    );
                    return None;
                }
                self.advance();
                let err = self.parse_type()?;
                if !self.check(&[TokenKind::Greater]) {
                    self.error(
                        "E021",
                        "Se esperaba '>' para cerrar el tipo resultado",
                        token.span,
                        "Agrega '>' después del tipo de error",
                    );
                    return None;
                }
                self.advance();
                Some(Type::Resultado {
                    ok: Box::new(ok),
                    err: Box::new(err),
                })
            }
            TokenKind::LeftParen => {
                let start = token.span;
                let mut types = Vec::new();
                if !self.check(&[TokenKind::RightParen]) {
                    types.push(self.parse_type()?);
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        if self.check(&[TokenKind::RightParen]) {
                            break;
                        }
                        types.push(self.parse_type()?);
                    }
                }
                if !self.check(&[TokenKind::RightParen]) {
                    self.error(
                        "E015",
                        "Se esperaba ')' para cerrar el tipo tupla",
                        start,
                        "Agrega ')' después de los tipos",
                    );
                    return None;
                }
                self.advance();
                if types.len() == 1 {
                    Some(types.into_iter().next().unwrap())
                } else {
                    Some(Type::Tuple(types))
                }
            }
            TokenKind::Opcion | TokenKind::Option => {
                if !self.check(&[TokenKind::Less]) {
                    self.error(
                        "E021",
                        "Se esperaba '<' para el tipo opcional",
                        token.span,
                        "Agrega '<tipo>' después de 'opcion'",
                    );
                    return None;
                }
                self.advance();
                let inner = self.parse_type()?;
                if !self.check(&[TokenKind::Greater]) {
                    self.error(
                        "E021",
                        "Se esperaba '>' para cerrar el tipo opcional",
                        token.span,
                        "Agrega '>' después del tipo interno",
                    );
                    return None;
                }
                self.advance();
                Some(Type::Opcion(Box::new(inner)))
            }
            _ => None,
        }
    }

    fn check_next_is_tuple_type(&self) -> bool {
        if !self.check(&[TokenKind::LeftParen]) {
            return false;
        }
        if self.pos + 1 >= self.tokens.len() {
            return false;
        }
        let next = &self.tokens[self.pos + 1].kind;
        matches!(
            next,
            TokenKind::Numero
                | TokenKind::Entero
                | TokenKind::Decimal
                | TokenKind::Texto
                | TokenKind::Booleano
                | TokenKind::Lista
                | TokenKind::Array
                | TokenKind::Resultado
                | TokenKind::Result
                | TokenKind::Opcion
                | TokenKind::Option
                | TokenKind::Number
                | TokenKind::Integer
                | TokenKind::Float
                | TokenKind::String
                | TokenKind::Boolean
                | TokenKind::LeftParen
        )
    }

    fn check_ident_next_is_generic_type(&self) -> bool {
        if !self.check_ident() {
            return false;
        }
        if self.pos + 2 >= self.tokens.len() {
            return false;
        }
        if !matches!(&self.tokens[self.pos + 1].kind, TokenKind::Less) {
            return false;
        }
        if !self.is_type_at(self.pos + 2) {
            return false;
        }
        // Peek past the <...> to ensure what follows is a variable name, not ( or {
        let after_gt = self.find_token_after_type_args(self.pos);
        if let Some(tok) = after_gt {
            matches!(tok.kind, TokenKind::Ident(_))
        } else {
            false
        }
    }

    /// Starting from an Ident at `start_pos` followed by `<`, find the token after the matching `>`.
    fn find_token_after_type_args(&self, start_pos: usize) -> Option<&Token> {
        let mut depth = 0u32;
        let mut i = start_pos + 1; // start at <
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::Less => depth += 1,
                TokenKind::Greater => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return self.tokens.get(i + 1);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        None
    }

    // --- Helpers ---

    fn advance(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous_token()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    fn previous_token(&self) -> Option<Token> {
        if self.pos > 0 {
            Some(self.tokens[self.pos - 1].clone())
        } else {
            None
        }
    }

    fn check(&self, kinds: &[TokenKind]) -> bool {
        if self.is_at_end() {
            return false;
        }
        let kind = &self.peek().kind;
        kinds.iter().any(|k| token_matches(kind, k))
    }

    fn check_ident(&self) -> bool {
        if self.is_at_end() {
            return false;
        }
        matches!(self.peek().kind, TokenKind::Ident(_))
    }

    fn check_ident_next(&self) -> bool {
        if self.is_at_end() {
            return false;
        }
        matches!(self.peek().kind, TokenKind::Ident(_))
            && self.pos + 1 < self.tokens.len()
            && matches!(self.tokens[self.pos + 1].kind, TokenKind::Ident(_))
    }

    fn check_next(&self, kinds: &[TokenKind]) -> bool {
        if self.pos + 1 >= self.tokens.len() {
            return false;
        }
        let kind = &self.tokens[self.pos + 1].kind;
        kinds.iter().any(|k| token_matches(kind, k))
    }

    fn check_next_comma_and_ident(&self) -> bool {
        if self.pos + 2 >= self.tokens.len() {
            return false;
        }
        matches!(&self.tokens[self.pos].kind, TokenKind::Ident(_))
            && matches!(&self.tokens[self.pos + 1].kind, TokenKind::Comma)
            && matches!(&self.tokens[self.pos + 2].kind, TokenKind::Ident(_))
    }

    fn peek_ident_is(&self, s: &str) -> bool {
        if let TokenKind::Ident(ref name) = &self.peek().kind {
            name == s
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.peek().kind, TokenKind::Eof)
    }

    fn expect_field_name(&mut self) -> Option<String> {
        let token = self.peek();
        match &token.kind {
            TokenKind::Ident(s) => {
                let s = s.clone();
                self.advance();
                Some(s)
            }
            TokenKind::Numero | TokenKind::Number => {
                self.advance();
                Some("numero".to_string())
            }
            TokenKind::Entero | TokenKind::Integer => {
                self.advance();
                Some("entero".to_string())
            }
            TokenKind::Decimal | TokenKind::Float => {
                self.advance();
                Some("decimal".to_string())
            }
            TokenKind::Texto | TokenKind::String => {
                self.advance();
                Some("texto".to_string())
            }
            TokenKind::Booleano | TokenKind::Boolean => {
                self.advance();
                Some("booleano".to_string())
            }
            TokenKind::Lista | TokenKind::Array => {
                self.advance();
                Some("lista".to_string())
            }
            TokenKind::Verdadero | TokenKind::True => {
                self.advance();
                Some("verdadero".to_string())
            }
            TokenKind::Falso | TokenKind::False => {
                self.advance();
                Some("falso".to_string())
            }
            TokenKind::Funcion | TokenKind::Function => {
                self.advance();
                Some("funcion".to_string())
            }
            TokenKind::Retornar | TokenKind::Return => {
                self.advance();
                Some("retornar".to_string())
            }
            TokenKind::Si | TokenKind::If => {
                self.advance();
                Some("si".to_string())
            }
            TokenKind::Sino | TokenKind::Else => {
                self.advance();
                Some("sino".to_string())
            }
            TokenKind::Mientras | TokenKind::While => {
                self.advance();
                Some("mientras".to_string())
            }
            TokenKind::Para | TokenKind::For => {
                self.advance();
                Some("para".to_string())
            }
            TokenKind::Imprimir | TokenKind::Print => {
                self.advance();
                Some("imprimir".to_string())
            }
            TokenKind::Leer | TokenKind::Read => {
                self.advance();
                Some("leer".to_string())
            }
            TokenKind::Romper | TokenKind::Break => {
                self.advance();
                Some("romper".to_string())
            }
            TokenKind::Continuar | TokenKind::Continue => {
                self.advance();
                Some("continuar".to_string())
            }
            TokenKind::Elegir | TokenKind::Match => {
                self.advance();
                Some("elegir".to_string())
            }
            TokenKind::Caso | TokenKind::Case => {
                self.advance();
                Some("caso".to_string())
            }
            TokenKind::Defecto | TokenKind::Default => {
                self.advance();
                Some("defecto".to_string())
            }
            TokenKind::Estructura | TokenKind::Struct => {
                self.advance();
                Some("estructura".to_string())
            }
            TokenKind::Importar | TokenKind::Import => {
                self.advance();
                Some("importar".to_string())
            }
            TokenKind::Como | TokenKind::As => {
                self.advance();
                Some("como".to_string())
            }
            TokenKind::En | TokenKind::In => {
                self.advance();
                Some("en".to_string())
            }
            _ => {
                self.error(
                    "E011",
                    "Se esperaba un nombre de campo",
                    token.span,
                    "Escribe un identificador",
                );
                None
            }
        }
    }

    fn expect_ident(&mut self) -> Option<String> {
        let token = self.advance()?;
        match token.kind {
            TokenKind::Ident(s) => Some(s),
            _ => {
                self.error(
                    "E011",
                    "Se esperaba un nombre de variable",
                    token.span,
                    "Escribe un identificador",
                );
                None
            }
        }
    }

    fn expect_semicolon(&mut self) {
        if !self.check(&[TokenKind::Semicolon]) {
            self.error(
                "E012",
                "Se esperaba ';'",
                self.previous().span,
                "Agrega ';' al final de la declaración",
            );
        } else {
            self.advance();
        }
    }

    fn error(
        &mut self,
        code: &str,
        message: impl Into<String>,
        span: Span,
        suggestion: impl Into<String>,
    ) {
        self.errors.push(ParseError {
            code: code.to_string(),
            message: message.into(),
            span,
            suggestion: suggestion.into(),
        });
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            match self.peek().kind {
                TokenKind::Funcion
                | TokenKind::Function
                | TokenKind::Numero
                | TokenKind::Number
                | TokenKind::Entero
                | TokenKind::Integer
                | TokenKind::Decimal
                | TokenKind::Float
                | TokenKind::Texto
                | TokenKind::String
                | TokenKind::Booleano
                | TokenKind::Boolean
                | TokenKind::Lista
                | TokenKind::Array
                | TokenKind::Si
                | TokenKind::If
                | TokenKind::Mientras
                | TokenKind::While
                | TokenKind::Para
                | TokenKind::For
                | TokenKind::Retornar
                | TokenKind::Return
                | TokenKind::Romper
                | TokenKind::Break
                | TokenKind::Continuar
                | TokenKind::Continue
                | TokenKind::Elegir
                | TokenKind::Match
                | TokenKind::LeftBrace
                | TokenKind::LeftBracket
                | TokenKind::Importar
                | TokenKind::Import
                | TokenKind::Resultado
                | TokenKind::Result => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}

fn token_matches(kind: &TokenKind, expected: &TokenKind) -> bool {
    std::mem::discriminant(kind) == std::mem::discriminant(expected)
        || matches!(
            (kind, expected),
            (TokenKind::Numero, TokenKind::Number)
                | (TokenKind::Number, TokenKind::Numero)
                | (TokenKind::Entero, TokenKind::Integer)
                | (TokenKind::Integer, TokenKind::Entero)
                | (TokenKind::Decimal, TokenKind::Float)
                | (TokenKind::Float, TokenKind::Decimal)
                | (TokenKind::Texto, TokenKind::String)
                | (TokenKind::String, TokenKind::Texto)
                | (TokenKind::Booleano, TokenKind::Boolean)
                | (TokenKind::Boolean, TokenKind::Booleano)
                | (TokenKind::Si, TokenKind::If)
                | (TokenKind::If, TokenKind::Si)
                | (TokenKind::Sino, TokenKind::Else)
                | (TokenKind::Else, TokenKind::Sino)
                | (TokenKind::Mientras, TokenKind::While)
                | (TokenKind::While, TokenKind::Mientras)
                | (TokenKind::Para, TokenKind::For)
                | (TokenKind::For, TokenKind::Para)
                | (TokenKind::Funcion, TokenKind::Function)
                | (TokenKind::Function, TokenKind::Funcion)
                | (TokenKind::Retornar, TokenKind::Return)
                | (TokenKind::Return, TokenKind::Retornar)
                | (TokenKind::Verdadero, TokenKind::True)
                | (TokenKind::True, TokenKind::Verdadero)
                | (TokenKind::Falso, TokenKind::False)
                | (TokenKind::False, TokenKind::Falso)
                | (TokenKind::Imprimir, TokenKind::Print)
                | (TokenKind::Print, TokenKind::Imprimir)
                | (TokenKind::Leer, TokenKind::Read)
                | (TokenKind::Read, TokenKind::Leer)
                | (TokenKind::Lista, TokenKind::Array)
                | (TokenKind::Array, TokenKind::Lista)
                | (TokenKind::Romper, TokenKind::Break)
                | (TokenKind::Break, TokenKind::Romper)
                | (TokenKind::Continuar, TokenKind::Continue)
                | (TokenKind::Continue, TokenKind::Continuar)
                | (TokenKind::Elegir, TokenKind::Match)
                | (TokenKind::Match, TokenKind::Elegir)
                | (TokenKind::Caso, TokenKind::Case)
                | (TokenKind::Case, TokenKind::Caso)
                | (TokenKind::Defecto, TokenKind::Default)
                | (TokenKind::Default, TokenKind::Defecto)
                | (TokenKind::Estructura, TokenKind::Struct)
                | (TokenKind::Struct, TokenKind::Estructura)
                | (TokenKind::Importar, TokenKind::Import)
                | (TokenKind::Import, TokenKind::Importar)
                | (TokenKind::Como, TokenKind::As)
                | (TokenKind::As, TokenKind::Como)
                | (TokenKind::Resultado, TokenKind::Result)
                | (TokenKind::Result, TokenKind::Resultado)
                | (TokenKind::Exito, TokenKind::Ok)
                | (TokenKind::Ok, TokenKind::Exito)
                | (TokenKind::ErrKeyword, TokenKind::Err)
                | (TokenKind::Err, TokenKind::ErrKeyword)
                | (TokenKind::Intentar, TokenKind::Try)
                | (TokenKind::Try, TokenKind::Intentar)
                | (TokenKind::En, TokenKind::In)
                | (TokenKind::In, TokenKind::En)
        )
}

#[allow(dead_code)]
trait Spannable {
    fn span(&self) -> Span;
}

#[allow(dead_code)]
impl Spannable for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Int { span, .. }
            | Expr::Float { span, .. }
            | Expr::Str { span, .. }
            | Expr::Bool { span, .. }
            | Expr::Ident { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Call { span, .. }
            | Expr::Grouping { span, .. }
            | Expr::List { span, .. }
            | Expr::Index { span, .. }
            | Expr::MethodCall { span, .. }
            | Expr::Lambda { span, .. }
            | Expr::StructInit { span, .. }
            | Expr::FieldAccess { span, .. }
            | Expr::Exito { span, .. }
            | Expr::Error { span, .. }
            | Expr::Intentar { span, .. }
            | Expr::Algun { span, .. }
            | Expr::Ninguno { span, .. }
            | Expr::EnumCtor { span, .. }
            | Expr::Tuple { span, .. }
            | Expr::TupleAccess { span, .. } => *span,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use lumen_lexer::Lexer;

    fn parse(source: &str) -> (Program, Vec<ParseError>) {
        let lexer = Lexer::new(source);
        let (tokens, lex_errors) = lexer.tokenize();
        assert!(lex_errors.is_empty(), "Lexer errors: {:?}", lex_errors);
        let parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_empty_program() {
        let (program, errors) = parse("");
        assert!(errors.is_empty());
        assert!(program.is_empty());
    }

    #[test]
    fn test_variable_declaration() {
        let (program, errors) = parse("numero x = 42;");
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_function_declaration() {
        let source = "funcion numero suma(numero a, numero b) { retornar a + b; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_if_statement() {
        let source = "si (x > 0) { imprimir(x); } sino { imprimir(0); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_while_loop() {
        let source = "mientras (x < 10) { x = x + 1; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_nested_block() {
        let source = "numero x = 1; { numero y = 2; x = x + y; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 2);
        if let DeclOrStmt::Stmt(Stmt::Block { stmts, .. }) = &program[1] {
            assert_eq!(stmts.len(), 2);
        } else {
            panic!("Expected block statement");
        }
    }

    #[test]
    fn test_function_call() {
        let source = "suma(3, 7);";
        let (_program, errors) = parse(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_hello_world() {
        let source = r#"imprimir("¡Hola, LÚMEN!");"#;
        let (_program, errors) = parse(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_loop_program() {
        let source = "numero contador = 0;
mientras (contador < 5) {
    imprimir(contador);
    contador = contador + 1;
}";
        let (_program, errors) = parse(source);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_func_program() {
        let source = "funcion numero suma(numero a, numero b) { retornar a + b; }
numero res = suma(3, 7);
imprimir(res);";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 3);
    }

    #[test]
    fn test_error_missing_semicolon() {
        let source = "numero x = 42";
        let (_program, errors) = parse(source);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E012");
    }

    #[test]
    fn test_error_missing_identifier_after_type() {
        let source = "numero 42;";
        let (_program, errors) = parse(source);
        assert!(!errors.is_empty());
        // Should produce E011 for expected identifier
    }

    #[test]
    fn test_error_invalid_type() {
        let source = "123 x = 42;";
        let (_program, errors) = parse(source);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_parse_block_stmt() {
        let source = "{ numero x = 1; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
        match &program[0] {
            DeclOrStmt::Stmt(Stmt::Block { .. }) => {}
            _ => panic!("Expected block statement"),
        }
    }

    #[test]
    fn test_parse_expr_stmt() {
        let source = "42;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
        match &program[0] {
            DeclOrStmt::Stmt(Stmt::Expr { .. }) => {}
            _ => panic!("Expected expr statement"),
        }
    }

    #[test]
    fn test_parse_grouping() {
        let source = "x = (1 + 2) * 3;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_logical_operators() {
        let source = "booleano b = verdadero && falso || verdadero;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_comparison_chain() {
        let source = "booleano b = x < y && y > z;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_unary_negate() {
        let source = "numero x = -42;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_unary_not() {
        let source = "booleano b = !verdadero;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_callable_keyword() {
        let source = r#"imprimir("hola");"#;
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_read_call() {
        let source = "leer();";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_boolean_literals() {
        let source = "booleano b1 = verdadero; booleano b2 = falso;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 2);
    }

    #[test]
    fn test_parse_empty_return() {
        let source = "funcion void nada() { retornar; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_type_texto() {
        let source = r#"texto s = "hola";"#;
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_english_keywords() {
        let source = "number x = 42; boolean b = true; string s = \"hello\"; while (x > 0) { x = x - 1; } if (b) { print(x); } for (number i = 0; i < 5; i = i + 1) { } function number foo(number a) { return a; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert!(program.len() >= 6);
    }

    #[test]
    fn test_parse_type_booleano() {
        let source = "booleano b = verdadero;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty());
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_error_synchronize() {
        // Test synchronize: invalid declaration `numero ;` triggers recovery
        use lumen_lexer::token::Pos;
        let p = |l: usize, c: usize| Pos { line: l, col: c };
        let t = |kind: TokenKind, l: usize, c: usize| Token {
            kind,
            span: Span {
                start: p(l, c),
                end: p(l, c + 1),
            },
        };
        let tokens = vec![
            t(TokenKind::Numero, 1, 1),
            t(TokenKind::Semicolon, 1, 1),
            t(TokenKind::Numero, 1, 1),
            t(TokenKind::Ident("y".to_string()), 1, 1),
            t(TokenKind::Equal, 1, 1),
            t(TokenKind::NumLiteral("2".to_string()), 1, 1),
            t(TokenKind::Semicolon, 1, 1),
            t(TokenKind::Eof, 1, 1),
        ];
        let parser = Parser::new(tokens);
        let (_program, errors) = parser.parse();
        assert!(!errors.is_empty());
        // Synchronize skips to statement boundary, valid code may be consumed
        assert_eq!(errors[0].code, "E011");
    }

    #[test]
    fn test_parse_resultado_type() {
        let source = "resultado<entero, texto> r = exito(42);";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_resultado_error() {
        let source = r#"resultado<entero, texto> r = error("falló");"#;
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_intentar() {
        let source = r#"funcion entero foo() {
    resultado<entero, texto> r = exito(42);
    retornar intentar r;
}"#;
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_resultado_func_return() {
        let source = "funcion resultado<entero, texto> dividir(entero a, entero b) { }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_foreach_spanish() {
        let source = "lista<entero> nums = [1, 2, 3];
para n en nums {
    imprimir(n);
}";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 2);
        if let DeclOrStmt::Stmt(Stmt::ForEach { var_name, .. }) = &program[1] {
            assert_eq!(var_name, "n");
        } else {
            panic!("Expected ForEach statement");
        }
    }

    #[test]
    fn test_parse_foreach_english() {
        let source = "array<integer> nums = [1, 2, 3];
for n in nums {
    print(n);
}";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 2);
        if let DeclOrStmt::Stmt(Stmt::ForEach { var_name, .. }) = &program[1] {
            assert_eq!(var_name, "n");
        } else {
            panic!("Expected ForEach statement");
        }
    }

    #[test]
    fn test_parse_foreach_nested() {
        let source = "lista<entero> nums = [1, 2];
para a en nums {
    para b en nums {
        imprimir(a * b);
    }
}";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 2);
    }

    #[test]
    fn test_cstyle_for_still_works() {
        let source = "para (entero i = 0; i < 5; i = i + 1) {
    imprimir(i);
}";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
        assert!(matches!(&program[0], DeclOrStmt::Stmt(Stmt::For { .. })));
    }

    // --- Generics parser tests ---

    #[test]
    fn test_parse_generic_function() {
        let source = "funcion T identidad<T>(T valor) { retornar valor; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
        if let DeclOrStmt::Decl(Decl::Function { name, type_params, .. }) = &program[0] {
            assert_eq!(name, "identidad");
            assert_eq!(type_params, &vec!["T".to_string()]);
        } else {
            panic!("Expected Function declaration");
        }
    }

    #[test]
    fn test_parse_generic_function_multi_param() {
        let source = "funcion T foo<T, U>(T a, U b) { retornar a; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Function { type_params, .. }) = &program[0] {
            assert_eq!(type_params, &vec!["T".to_string(), "U".to_string()]);
        } else {
            panic!("Expected Function declaration");
        }
    }

    #[test]
    fn test_parse_generic_struct() {
        let source = "estructura Par<T, U> { primero: T, segundo: U }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Struct { name, type_params, .. }) = &program[0] {
            assert_eq!(name, "Par");
            assert_eq!(type_params, &vec!["T".to_string(), "U".to_string()]);
        } else {
            panic!("Expected Struct declaration");
        }
    }

    #[test]
    fn test_parse_generic_call() {
        let source = "identidad<entero>(42);";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
        if let DeclOrStmt::Stmt(Stmt::Expr { expr, .. }) = &program[0] {
            if let Expr::Call { type_args, .. } = expr.as_ref() {
                assert_eq!(type_args.len(), 1);
                assert_eq!(type_args[0], Type::Entero);
            } else {
                panic!("Expected Call expression");
            }
        } else {
            panic!("Expected Expr statement");
        }
    }

    #[test]
    fn test_parse_generic_struct_init() {
        let source = "Par<entero, texto> p = Par<entero, texto> { primero: 1, segundo: \"hola\" };";
        let (program, errors) = parse(source);
        if !errors.is_empty() {
            let lexer = lumen_lexer::lexer::Lexer::new(source);
            let (tokens, _) = lexer.tokenize();
            for (i, t) in tokens.iter().enumerate() {
                println!("  {}: {:?} {:?}", i, t.kind, t.span);
            }
            panic!("Parse errors: {:?}", errors);
        }
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_comparison_still_works() {
        let source = "x < y;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
        if let DeclOrStmt::Stmt(Stmt::Expr { expr, .. }) = &program[0] {
            assert!(matches!(expr.as_ref(), Expr::Binary { op: BinOp::Less, .. }));
        } else {
            panic!("Expected Expr statement");
        }
    }
}

