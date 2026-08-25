use crate::ast::*;
use crate::error::ParseError;
use lumen_lexer::token::{Span, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    no_struct_init: bool,
    type_params_stack: Vec<Vec<String>>,
    pending_greater: bool,
    // Dentro de un arm de `elegir`, `|` separa patrones (OR) — nunca BinOp::BitOr.
    match_arm_pipe: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            no_struct_init: false,
            type_params_stack: Vec::new(),
            pending_greater: false,
            match_arm_pipe: false,
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
            ]) && self.check_next(&[TokenKind::LeftParen])
            {
                self.parse_expr_or_assign().map(DeclOrStmt::Stmt)
            } else {
                self.parse_declaration().map(DeclOrStmt::Decl)
            }
        } else if self.check(&[TokenKind::Async]) {
            self.parse_async_function().map(DeclOrStmt::Decl)
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
                TokenKind::Impl,
            ]) {
                self.parse_function().map(DeclOrStmt::Decl)
            } else if self.check_next(&[TokenKind::LeftParen]) {
                self.parse_expr_or_assign().map(DeclOrStmt::Stmt)
            } else {
                self.parse_function().map(DeclOrStmt::Decl)
            }
        } else if self.check(&[TokenKind::Sea, TokenKind::Let]) {
            if self.is_guard_let() {
                self.parse_guard_let().map(DeclOrStmt::Stmt)
            } else {
                self.parse_declaration().map(DeclOrStmt::Decl)
            }
        } else if self.check(&[TokenKind::Si, TokenKind::If]) {
            self.parse_if().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Mientras, TokenKind::While]) {
            self.parse_while().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Para, TokenKind::For]) {
            if self.check_next(&[TokenKind::LeftParen]) {
                self.parse_for().map(DeclOrStmt::Stmt)
            } else if self.is_foreach_like() {
                self.parse_foreach().map(DeclOrStmt::Stmt)
            } else {
                self.parse_for().map(DeclOrStmt::Stmt)
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
        } else if self.check(&[TokenKind::Const]) {
            self.parse_const().map(DeclOrStmt::Decl)
        } else if self.check(&[TokenKind::Rasgo, TokenKind::Trait]) {
            self.parse_rasgo().map(DeclOrStmt::Decl)
        } else if self.check(&[TokenKind::Impl]) {
            self.parse_impl_rasgo().map(DeclOrStmt::Decl)
        } else if self.check(&[TokenKind::Posponer, TokenKind::Defer]) {
            self.parse_posponer().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Intentar, TokenKind::Try]) && self.check_next_is_brace() {
            self.parse_try_catch().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::Ensamblador, TokenKind::Asm]) {
            self.parse_inline_asm().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::BloqueC, TokenKind::CBlock]) {
            self.parse_inline_c().map(DeclOrStmt::Stmt)
        } else if self.check(&[TokenKind::BloqueRust, TokenKind::RustBlock]) {
            self.parse_inline_rust().map(DeclOrStmt::Stmt)
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
        let mut var_type = self.parse_type()?;
        let name = self.expect_ident()?;

        // C-style array type: `entero paleta_r[] = [...]`
        if self.check(&[TokenKind::LeftBracket]) {
            self.advance();
            if !self.check(&[TokenKind::RightBracket]) {
                self.error(
                    "E022",
                    "Se esperaba ']' para cerrar el tipo arreglo",
                    self.peek().span,
                    "Agrega ']' después de '['",
                );
                return None;
            }
            self.advance();
            var_type = Type::Lista(Box::new(var_type));
        }

        if self.check(&[TokenKind::Comma]) {
            return self.parse_destructure_decl(var_type, name, start);
        }

        let init = if self.check(&[TokenKind::Equal]) {
            self.advance();
            // Permitir struct-init en el inicializador (p.ej. `sea c = Caja { valor: 1 };`)
            let saved_nsi = self.no_struct_init;
            self.no_struct_init = false;
            let e = self.parse_expression().map(Box::new);
            self.no_struct_init = saved_nsi;
            e
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

    fn parse_async_function(&mut self) -> Option<Decl> {
        self.advance(); // consume async
        let mut decl = self.parse_function()?;
        if let Decl::Function {
            ref mut is_async, ..
        } = &mut decl
        {
            *is_async = true;
        }
        Some(decl)
    }

    fn parse_function(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance();
        let return_type = self.parse_type()?;
        // Function name: ident or keyword (e.g. `funcion void texto(...)`)
        let name = match self.peek().kind {
            TokenKind::Ident(ref s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => {
                let kw = self.peek().kind.as_str();
                if kw.is_empty() {
                    self.advance();
                    self.error(
                        "E011",
                        "Se esperaba un nombre de función",
                        start,
                        "Escribe un identificador para la función",
                    );
                    return None;
                }
                let kw = kw.to_string();
                self.advance();
                kw
            }
        };

        if self.check(&[TokenKind::Equal]) {
            self.advance();
            let saved_nsi = self.no_struct_init;
            self.no_struct_init = false;
            let e = self.parse_expression().map(Box::new);
            self.no_struct_init = saved_nsi;
            self.expect_semicolon();
            return Some(Decl::Variable {
                var_type: return_type,
                name,
                init: e,
                span: Span::merge(&start, &self.previous().span),
            });
        }

        let (type_params, type_param_bounds) = self.parse_type_params();

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
            type_param_bounds,
            is_async: false,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_struct_decl(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance();
        let name = self.expect_ident()?;

        let (type_params, type_param_bounds) = self.parse_type_params();

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
            type_param_bounds,
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

    fn parse_const(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance(); // consume const
        let var_type = self.parse_type()?;
        let name = self.expect_ident()?;
        if !self.check(&[TokenKind::Equal]) {
            self.error(
                "E012",
                "Se esperaba '=' en declaración const",
                self.peek().span,
                "Agrega '=' después del nombre de la constante",
            );
            return None;
        }
        self.advance();
        let value = Box::new(self.parse_expression()?);
        self.expect_semicolon();
        Some(Decl::Const {
            var_type,
            name,
            value,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_rasgo(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance(); // consume rasgo/trait
        let name = self.expect_ident()?;
        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E017",
                "Se esperaba '{' para el rasgo",
                start,
                "Agrega '{' para definir los métodos",
            );
            return None;
        }
        self.advance();
        let mut methods = Vec::new();
        let mut associated_types = Vec::new();
        while !self.check(&[TokenKind::RightBrace]) && !self.is_at_end() {
            if self.check(&[TokenKind::Eof]) {
                break;
            }
            // Check for associated type: tipo Item;
            if self.check(&[TokenKind::Tipo]) {
                let type_start = self.peek().span;
                self.advance(); // consume tipo
                let assoc_name = self.expect_ident()?;
                let default = if self.check(&[TokenKind::Equal]) {
                    self.advance(); // consume =
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect_semicolon();
                associated_types.push(AssociatedType {
                    name: assoc_name,
                    default,
                    span: Span::merge(&type_start, &self.previous().span),
                });
            } else if self.check(&[TokenKind::Funcion, TokenKind::Function]) {
                // Parse method signature
                self.advance(); // consume funcion/function
                let ret_type = self.parse_type()?;
                let method_name = self.expect_ident()?;
                if !self.check(&[TokenKind::LeftParen]) {
                    self.error(
                        "E014",
                        "Se esperaba '('",
                        start,
                        "Agrega '(' para iniciar los parámetros",
                    );
                    return None;
                }
                self.advance();
                let mut params = Vec::new();
                if !self.check(&[TokenKind::RightParen]) {
                    // Check if first param is the receiver (just a name, no type)
                    if self.check_ident() && !self.is_type_keyword(&self.peek().kind) {
                        let receiver_name = self.expect_ident()?;
                        // Use a placeholder type for the receiver — resolved during sema
                        params.push(Param {
                            param_type: Type::Struct("Self".to_string()),
                            name: receiver_name,
                            default: None,
                            span: Span::merge(&start, &self.previous().span),
                        });
                        if self.check(&[TokenKind::Comma]) {
                            self.advance();
                        }
                    }
                    while !self.check(&[TokenKind::RightParen]) {
                        params.push(self.parse_param()?);
                        while self.check(&[TokenKind::Comma]) {
                            self.advance();
                            if self.check(&[TokenKind::RightParen]) {
                                break;
                            }
                            params.push(self.parse_param()?);
                        }
                    }
                }
                if !self.check(&[TokenKind::RightParen]) {
                    self.error(
                        "E015",
                        "Se esperaba ')'",
                        start,
                        "Agrega ')' para cerrar los parámetros",
                    );
                    return None;
                }
                self.advance();
                self.expect_semicolon();
                methods.push(TraitMethod {
                    name: method_name,
                    params,
                    return_type: ret_type,
                });
            } else {
                self.error(
                    "E072",
                    "Se esperaba 'funcion' o 'tipo' en el rasgo",
                    self.peek().span,
                    "Agrega 'funcion tipo nombre(...)' o 'tipo Nombre'",
                );
                return None;
            }
        }
        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E017",
                "Se esperaba '}' para cerrar el rasgo",
                start,
                "Agrega '}' al final del rasgo",
            );
            return None;
        }
        self.advance();
        Some(Decl::Rasgo {
            name,
            methods,
            associated_types,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_posponer(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance(); // consume posponer/defer
        let body = if self.check(&[TokenKind::LeftBrace]) {
            self.parse_block()?
        } else {
            let stmt = self.parse_decl_or_stmt()?;
            vec![stmt]
        };
        Some(Stmt::Posponer {
            body,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn check_next_is_brace(&self) -> bool {
        if self.pos + 1 >= self.tokens.len() {
            return false;
        }
        matches!(self.tokens[self.pos + 1].kind, TokenKind::LeftBrace)
    }

    fn parse_try_catch(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance(); // consume intentar/try
        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E017",
                "Se esperaba '{' después de 'intentar'",
                start,
                "Usa: intentar { ... } atrapar (e) { ... }",
            );
            return None;
        }
        let try_body = self.parse_block()?;
        if !self.check(&[TokenKind::Atrapar, TokenKind::Catch]) {
            self.error(
                "E012",
                "Se esperaba 'atrapar' / 'catch' después del bloque intentar",
                start,
                "Agrega 'atrapar (e) { ... }'",
            );
            return None;
        }
        self.advance(); // consume atrapar/catch
        let err_var = if self.check(&[TokenKind::LeftParen]) {
            self.advance();
            let v = self.expect_ident()?;
            if !self.check(&[TokenKind::RightParen]) {
                self.error(
                    "E015",
                    "Se esperaba ')' después del nombre de error",
                    start,
                    "Agrega ')'",
                );
                return None;
            }
            self.advance();
            v
        } else {
            "e".to_string()
        };
        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E017",
                "Se esperaba '{' para el bloque atrapar",
                start,
                "Agrega '{ ... }'",
            );
            return None;
        }
        let catch_body = self.parse_block()?;
        let end = self.previous().span;
        Some(Stmt::TryCatch {
            try_body,
            err_var,
            catch_body,
            span: Span::merge(&start, &end),
        })
    }

    fn parse_inline_asm(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E015",
                "Se esperaba '{' después de 'ensamblador' / 'asm'",
                start,
                "Agrega '{'",
            );
            return None;
        }
        self.advance();
        let _expr = self.parse_expression()?;
        let code = match &_expr {
            Expr::Str { value, .. } => value.clone(),
            _ => "/* inline asm */".to_string(),
        };
        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E015",
                "Se esperaba '}' para cerrar el bloque de ensamblador",
                start,
                "Agrega '}'",
            );
            return None;
        }
        self.advance();
        Some(Stmt::InlineAsm {
            code,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_inline_c(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E015",
                "Se esperaba '{' después de 'bloque_c'",
                start,
                "Agrega '{'",
            );
            return None;
        }
        self.advance();
        let _expr = self.parse_expression()?;
        let code = match &_expr {
            Expr::Str { value, .. } => value.clone(),
            _ => "/* inline c */".to_string(),
        };
        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E015",
                "Se esperaba '}' para cerrar el bloque C",
                start,
                "Agrega '}'",
            );
            return None;
        }
        self.advance();
        Some(Stmt::InlineC {
            code,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_inline_rust(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E015",
                "Se esperaba '{' después de 'bloque_rust'",
                start,
                "Agrega '{'",
            );
            return None;
        }
        self.advance();
        let _expr = self.parse_expression()?;
        let code = match &_expr {
            Expr::Str { value, .. } => value.clone(),
            _ => "/* inline rust */".to_string(),
        };
        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E015",
                "Se esperaba '}' para cerrar el bloque Rust",
                start,
                "Agrega '}'",
            );
            return None;
        }
        self.advance();
        Some(Stmt::InlineRust {
            code,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn is_type_keyword(&self, kind: &TokenKind) -> bool {
        matches!(
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
        )
    }

    fn parse_impl_rasgo(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance(); // consume impl
        let ident_name = self.expect_ident()?;
        let (trait_name, target_type) = if self.check(&[TokenKind::Para, TokenKind::For]) {
            self.advance(); // consume para/for
            let target_type = self.parse_type()?;
            (ident_name, target_type)
        } else if self.check(&[TokenKind::LeftBrace]) {
            (String::new(), Type::Struct(ident_name))
        } else {
            self.error(
                "E073",
                "Se esperaba 'para'/'for' después del nombre del rasgo o '{' para métodos",
                self.peek().span,
                "Agrega 'para <tipo>' o '{' directamente",
            );
            return None;
        };
        if !self.check(&[TokenKind::LeftBrace]) {
            self.error(
                "E017",
                "Se esperaba '{' para la implementación",
                start,
                "Agrega '{' para definir los métodos",
            );
            return None;
        }
        self.advance();
        let mut methods = Vec::new();
        let mut associated_types = Vec::new();
        while !self.check(&[TokenKind::RightBrace]) && !self.is_at_end() {
            if self.check(&[TokenKind::Eof]) {
                break;
            }
            if self.check(&[TokenKind::Tipo]) {
                let type_start = self.peek().span;
                self.advance(); // consume tipo
                let assoc_name = self.expect_ident()?;
                if !self.check(&[TokenKind::Equal]) {
                    self.error(
                        "E071",
                        "Se esperaba '=' en la especificación del tipo asociado",
                        self.peek().span,
                        "Agrega '=' seguido de un tipo",
                    );
                    return None;
                }
                self.advance(); // consume =
                let assoc_type = self.parse_type()?;
                self.expect_semicolon();
                associated_types.push(ImplAssociatedType {
                    name: assoc_name,
                    target_type: assoc_type,
                    span: Span::merge(&type_start, &self.previous().span),
                });
            } else if self.check(&[TokenKind::Funcion, TokenKind::Function]) {
                let func = self.parse_method_impl()?;
                methods.push(func);
            } else {
                self.error(
                    "E072",
                    "Se esperaba 'funcion' o 'tipo' en la implementación",
                    self.peek().span,
                    "Agrega 'funcion' o 'tipo Nombre = Tipo;'",
                );
                return None;
            }
        }
        if !self.check(&[TokenKind::RightBrace]) {
            self.error(
                "E017",
                "Se esperaba '}' para cerrar la implementación",
                start,
                "Agrega '}' al final",
            );
            return None;
        }
        self.advance();
        Some(Decl::ImplRasgo {
            trait_name,
            target_type,
            associated_types,
            methods,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_method_impl(&mut self) -> Option<Decl> {
        let start = self.peek().span;
        self.advance(); // consume funcion/function
        let return_type = self.parse_type()?;
        let name = self.expect_ident()?;

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
            // Check if first param is the receiver (just a name, no type)
            if self.check_ident() && !self.is_type_keyword(&self.peek().kind) {
                let receiver_name = self.expect_ident()?;
                params.push(Param {
                    param_type: Type::Struct("Self".to_string()),
                    name: receiver_name,
                    default: None,
                    span: Span::merge(&start, &self.previous().span),
                });
                if self.check(&[TokenKind::Comma]) {
                    self.advance();
                }
            }
            while !self.check(&[TokenKind::RightParen]) {
                params.push(self.parse_param()?);
                while self.check(&[TokenKind::Comma]) {
                    self.advance();
                    if self.check(&[TokenKind::RightParen]) {
                        break;
                    }
                    params.push(self.parse_param()?);
                }
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

        let body = self.parse_block()?;
        Some(Decl::Function {
            return_type,
            name,
            params,
            body,
            type_params: Vec::new(),
            type_param_bounds: Vec::new(),
            is_async: false,
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
        } else if self.check_ident() || Self::is_keyword(&self.peek().kind) {
            let token = self.advance()?;
            match &token.kind {
                TokenKind::Ident(s) => s.clone(),
                kind => kind.as_str().to_string(),
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
            if self.check_ident() || Self::is_keyword(&self.peek().kind) {
                let token = self.advance()?;
                match &token.kind {
                    TokenKind::Ident(s) => Some(s.clone()),
                    kind => Some(kind.as_str().to_string()),
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
        let saved = self.no_struct_init;
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
        self.no_struct_init = saved;
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
        // If-let: si sea Patron = Expr { ... }
        if self.check(&[TokenKind::Sea, TokenKind::Let]) {
            return self.parse_if_let(start);
        }
        let saved_no_struct = self.no_struct_init;
        self.no_struct_init = true;
        let condition = Box::new(self.parse_expression()?);
        self.no_struct_init = saved_no_struct;
        let then_body = self.parse_block()?;
        let else_body = if self.check(&[TokenKind::Sino, TokenKind::Else]) {
            self.advance();
            if self.check(&[TokenKind::Si, TokenKind::If]) {
                // sino si — chained if
                let nested_if = self.parse_if()?;
                Some(vec![DeclOrStmt::Stmt(nested_if)])
            } else {
                Some(self.parse_block()?)
            }
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

    fn parse_if_let(&mut self, start: Span) -> Option<Stmt> {
        self.advance(); // consume sea/let
        let pattern = self.parse_expression()?;
        if !self.check(&[TokenKind::Equal]) {
            self.error(
                "E071",
                "Se esperaba '=' en 'si sea'",
                start,
                "Agrega '=' y una expresión",
            );
            return None;
        }
        self.advance();
        // Avoid struct init ambiguity: parse value without treating `{` as struct fields
        let saved_no_struct = self.no_struct_init;
        self.no_struct_init = true;
        let value = Box::new(self.parse_expression()?);
        self.no_struct_init = saved_no_struct;
        let then_body = self.parse_block()?;
        let else_body = if self.check(&[TokenKind::Sino, TokenKind::Else]) {
            self.advance();
            if self.check(&[TokenKind::Si, TokenKind::If]) {
                let nested_if = self.parse_if()?;
                Some(vec![DeclOrStmt::Stmt(nested_if)])
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Some(Stmt::IfLet {
            pattern,
            value,
            then_body,
            else_body,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_guard_let(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance(); // consume sea/let
        let pattern = self.parse_expression()?;
        if !self.check(&[TokenKind::Equal]) {
            self.error(
                "E071",
                "Se esperaba '=' en 'sea' guard",
                start,
                "Agrega '=' y una expresión",
            );
            return None;
        }
        self.advance();
        let saved_no_struct = self.no_struct_init;
        self.no_struct_init = true;
        let value = Box::new(self.parse_expression()?);
        self.no_struct_init = saved_no_struct;
        if !self.check(&[TokenKind::Sino, TokenKind::Else]) {
            self.error(
                "E072",
                "Se esperaba 'sino' en 'sea' guard",
                start,
                "Agrega 'sino { }' con una instrucción divergente",
            );
            return None;
        }
        self.advance();
        let else_body = self.parse_block()?;
        Some(Stmt::GuardLet {
            pattern,
            value,
            else_body,
            span: Span::merge(&start, &self.previous().span),
        })
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        let start = self.peek().span;
        self.advance();
        let saved_no_struct = self.no_struct_init;
        self.no_struct_init = true;
        let condition = Box::new(self.parse_expression()?);
        self.no_struct_init = saved_no_struct;
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
        let has_paren = self.check(&[TokenKind::LeftParen]);
        if has_paren {
            self.advance();
        }
        let init = if self.is_for_init_decl() {
            Box::new(self.parse_declaration()?)
        } else {
            let start_i = self.peek().span;
            let name = self.expect_ident()?;
            if !self.check(&[TokenKind::Equal]) {
                return None;
            }
            self.advance();
            let value = Box::new(self.parse_expression()?);
            if !self.check(&[TokenKind::Semicolon]) {
                return None;
            }
            self.advance();
            Box::new(Decl::Variable {
                var_type: Type::Struct("Infer".to_string()),
                name,
                init: Some(value),
                span: Span::merge(&start_i, &self.previous().span),
            })
        };
        let condition = Box::new(self.parse_expression()?);
        if !self.check(&[TokenKind::Semicolon]) {
            return None;
        }
        self.advance();
        let update = Box::new(self.parse_assignment()?);
        if has_paren {
            if !self.check(&[TokenKind::RightParen]) {
                return None;
            }
            self.advance();
        }
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
        let label = if self.check_ident() {
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.expect_semicolon();
        Some(Stmt::Break { label, span })
    }

    fn parse_continue(&mut self) -> Option<Stmt> {
        let token = self.advance()?;
        let span = token.span;
        let label = if self.check_ident() {
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.expect_semicolon();
        Some(Stmt::Continue { label, span })
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

                // En patrones, `|` separa alternativas (OR) — nunca BitOr.
                let saved_pipe = self.match_arm_pipe;
                self.match_arm_pipe = true;
                let value = self.parse_expression();

                // OR patterns: A | B | C
                let mut alt_values = Vec::new();
                while self.check(&[TokenKind::Pipe]) {
                    self.advance();
                    if let Some(alt) = self.parse_expression() {
                        alt_values.push(alt);
                    }
                }
                self.match_arm_pipe = saved_pipe;
                let value = value?;

                let guard = if self.check(&[TokenKind::Si, TokenKind::If]) {
                    self.advance();
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };

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
                    guard,
                    body,
                    alt_values,
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
            let t = self.advance()?;
            let name = match t.kind {
                TokenKind::Ident(s) => s,
                _ => unreachable!(),
            };
            self.advance();
            let value = Box::new(self.parse_expression()?);
            self.expect_semicolon();
            Some(Stmt::Assignment {
                name,
                value,
                span: Span::merge(&start, &self.previous().span),
            })
        } else {
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
                    Expr::Index {
                        expr: target,
                        index,
                        ..
                    } => Some(Stmt::ArraySet {
                        arr: target,
                        index,
                        value,
                        span: Span::merge(&start, &self.previous().span),
                    }),
                    _ => {
                        self.error(
                            "E024",
                            "No se puede asignar a esta expresión",
                            start,
                            "Solo se puede asignar a variables, índices y campos de struct",
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
        }
    }

    // --- Pratt Parser for Expressions ---

    fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Option<Expr> {
        let mut left = self.parse_ternary()?;
        while self.check(&[TokenKind::PipeGreater]) {
            self.advance();
            let right = self.parse_ternary()?;
            let span = Span::merge(&left.span(), &right.span());
            match right {
                Expr::Call {
                    callee,
                    mut args,
                    type_args,
                    span: _,
                } => {
                    args.insert(0, left);
                    left = Expr::Call {
                        callee,
                        args,
                        type_args,
                        span,
                    };
                }
                Expr::Ident {
                    name,
                    span: id_span,
                } => {
                    left = Expr::Call {
                        callee: Box::new(Expr::Ident {
                            name,
                            span: id_span,
                        }),
                        args: vec![left],
                        type_args: Vec::new(),
                        span,
                    };
                }
                Expr::MethodCall {
                    expr: target,
                    method,
                    mut args,
                    resolved_func,
                    span: _,
                } => {
                    args.insert(0, left);
                    left = Expr::MethodCall {
                        expr: target,
                        method,
                        args,
                        resolved_func,
                        span,
                    };
                }
                other => {
                    left = Expr::Call {
                        callee: Box::new(other),
                        args: vec![left],
                        type_args: Vec::new(),
                        span,
                    };
                }
            }
        }
        Some(left)
    }

    fn parse_ternary(&mut self) -> Option<Expr> {
        let condition = self.parse_logical_or()?;
        if self.check(&[TokenKind::QuestionColon]) {
            let start = condition.span();
            self.advance();
            let default_branch = self.parse_expression()?;
            let span = Span::merge(&start, &default_branch.span());
            return Some(Expr::Elvis {
                expr: Box::new(condition),
                default: Box::new(default_branch),
                span,
            });
        }
        if !self.check(&[TokenKind::Question]) {
            return Some(condition);
        }
        let start = condition.span();
        self.advance();
        let true_branch = self.parse_expression()?;
        if !self.check(&[TokenKind::Colon]) {
            self.error(
                "E012",
                "Se esperaba ':' en el operador ternario",
                self.peek().span,
                "Agrega ':' para la rama falsa del ternario",
            );
            return None;
        }
        self.advance();
        let false_branch = self.parse_expression()?;
        let span = Span::merge(&start, &false_branch.span());
        Some(Expr::Ternary {
            condition: Box::new(condition),
            true_branch: Box::new(true_branch),
            false_branch: Box::new(false_branch),
            span,
        })
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
                resolved_method: None,
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
                resolved_method: None,
                span,
            };
        }
        Some(left)
    }

    fn parse_comparison(&mut self) -> Option<Expr> {
        // Range: `a..b` (exclusivo) / `a..=b` (inclusivo) — precedencia alta,
        // se evalua como un solo operando en las comparaciones de `elegir`.
        let mut left = self.parse_addition()?;
        if self.check(&[TokenKind::DotDot, TokenKind::DotDotEqual]) {
            let inclusive = self.check(&[TokenKind::DotDotEqual]);
            self.advance();
            let right = self.parse_addition()?;
            let span = Span::merge(&left.span(), &right.span());
            return Some(Expr::Range {
                start: Box::new(left),
                end: Box::new(right),
                inclusive,
                span,
            });
        }
        while self.check(&[
            TokenKind::EqualEqual,
            TokenKind::BangEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Ampersand,
        ]) {
            let op = match self.peek().kind {
                TokenKind::EqualEqual => BinOp::Equal,
                TokenKind::BangEqual => BinOp::NotEqual,
                TokenKind::Less => BinOp::Less,
                TokenKind::LessEqual => BinOp::LessEqual,
                TokenKind::Greater => BinOp::Greater,
                TokenKind::GreaterEqual => BinOp::GreaterEqual,
                TokenKind::Ampersand => BinOp::BitAnd,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_addition()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                resolved_method: None,
                span,
            };
        }
        Some(left)
    }

    fn parse_addition(&mut self) -> Option<Expr> {
        let mut left = self.parse_shift()?;
        while self.check(&[
            TokenKind::Plus,
            TokenKind::PlusPlus,
            TokenKind::Minus,
            TokenKind::Caret,
        ]) || (self.check(&[TokenKind::Pipe]) && !self.match_arm_pipe)
        {
            let op = match self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::PlusPlus => BinOp::Concat,
                TokenKind::Minus => BinOp::Sub,
                TokenKind::Pipe => BinOp::BitOr,
                TokenKind::Caret => BinOp::BitXor,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_shift()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                resolved_method: None,
                span,
            };
        }
        Some(left)
    }

    fn parse_shift(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplication()?;
        while self.check(&[TokenKind::ShiftLeft, TokenKind::ShiftRight]) {
            let op = match self.peek().kind {
                TokenKind::ShiftLeft => BinOp::ShiftLeft,
                TokenKind::ShiftRight => BinOp::ShiftRight,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplication()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                resolved_method: None,
                span,
            };
        }
        Some(left)
    }

    fn parse_multiplication(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;
        while self.check(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op = match self.peek().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            let span = Span::merge(&left.span(), &right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                resolved_method: None,
                span,
            };
        }
        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        if self.check(&[TokenKind::Esperar, TokenKind::Await]) {
            let start = self.peek().span;
            self.advance();
            let expr = self.parse_unary()?;
            let span = Span::merge(&start, &expr.span());
            Some(Expr::Esperar {
                expr: Box::new(expr),
                span,
            })
        } else if self.check(&[TokenKind::Minus, TokenKind::Bang, TokenKind::Tilde]) {
            let op = match self.peek().kind {
                TokenKind::Minus => UnOp::Negate,
                TokenKind::Bang => UnOp::Not,
                TokenKind::Tilde => UnOp::BitNot,
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
                    Some(t) => match t.kind {
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
                                    resolved_func: None,
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
                            let field_name = t.kind.as_str();
                            if field_name.is_empty() {
                                self.error("E024", "Se esperaba un nombre de campo o índice numérico después de '.'", t.span, "Escribe el nombre del campo o un número");
                                return Some(expr);
                            }
                            let span = Span::merge(&start, &self.previous().span);
                            expr = Expr::FieldAccess {
                                expr: Box::new(expr),
                                field: field_name.to_string(),
                                span,
                            };
                        }
                    },
                    None => return Some(expr),
                }
            } else if self.check(&[TokenKind::QuestionDot]) {
                let start = expr.span();
                self.advance();
                let field = self.expect_ident()?;
                let span = Span::merge(&start, &self.previous().span);
                expr = Expr::SafeFieldAccess {
                    expr: Box::new(expr),
                    field,
                    span,
                };
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
            } else if self.check(&[TokenKind::Como, TokenKind::As]) {
                // Cast `X como T` — no-op de bytecode (el valor pasa tal cual),
                // pero con tipado real en el análisis semántico.
                self.advance();
                let cast_type = self.parse_type()?;
                let cast_span = expr.span();
                let cast_merge = Span::merge(&cast_span, &self.previous().span);
                expr = Expr::Cast {
                    expr: Box::new(expr),
                    cast_type,
                    span: cast_merge,
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
                if s.contains('.') || s.contains('e') || s.contains('E') {
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
            TokenKind::FStrLiteral(s) => self.parse_fstring(s, span),
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
                if self.check(&[TokenKind::RightBracket]) {
                    self.advance();
                    return Some(Expr::List {
                        items: Vec::new(),
                        span: Span::merge(&span, &self.previous().span),
                    });
                }
                let first_expr = self.parse_expression()?;
                // Comprensión de lista: `[expr para x en iter]` o `[expr para x en iter si cond]`
                if self.check(&[TokenKind::Para, TokenKind::For]) {
                    self.advance();
                    let var_name = self.expect_ident()?;
                    if !self.check(&[TokenKind::En, TokenKind::In]) {
                        self.error(
                            "E012",
                            "Se esperaba 'en' / 'in' en la comprensión de lista",
                            span,
                            "Usa: [expr para variable en coleccion]",
                        );
                        return None;
                    }
                    self.advance();
                    let iter_expr = self.parse_expression()?;
                    let condition = if self.check(&[TokenKind::Si, TokenKind::If]) {
                        self.advance();
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    };
                    if !self.check(&[TokenKind::RightBracket]) {
                        self.error(
                            "E022",
                            "Se esperaba ']' para cerrar la comprensión de lista",
                            span,
                            "Agrega ']' al final de la comprensión",
                        );
                        return None;
                    }
                    self.advance();
                    return Some(Expr::Comprehension {
                        expr: Box::new(first_expr),
                        var_name,
                        iter: Box::new(iter_expr),
                        condition,
                        span: Span::merge(&span, &self.previous().span),
                    });
                }
                let mut items = vec![first_expr];
                while self.check(&[TokenKind::Comma]) {
                    self.advance();
                    if self.check(&[TokenKind::RightBracket]) {
                        break;
                    }
                    items.push(self.parse_expression()?);
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
            TokenKind::Consultar | TokenKind::Query => {
                let start = span;
                let var_name = self.expect_ident()?;
                if !self.check(&[TokenKind::En, TokenKind::In]) {
                    self.error(
                        "E012",
                        "Se esperaba 'en' / 'in' en la consulta",
                        start,
                        "Usa: consultar <variable> en <origen>",
                    );
                    return None;
                }
                self.advance();
                let source = self.parse_expression()?;
                let where_clause = if self.check(&[TokenKind::Donde, TokenKind::Where]) {
                    self.advance();
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                let (order_by, descending) =
                    if self.check(&[TokenKind::OrdenarPor, TokenKind::OrderBy]) {
                        self.advance();
                        let ord_expr = self.parse_expression()?;
                        let desc = if self.check(&[TokenKind::Descendente, TokenKind::Descending]) {
                            self.advance();
                            true
                        } else if self.check(&[TokenKind::Ascendente, TokenKind::Ascending]) {
                            self.advance();
                            false
                        } else {
                            false
                        };
                        (Some(Box::new(ord_expr)), desc)
                    } else {
                        (None, false)
                    };
                if !self.check(&[TokenKind::Seleccionar, TokenKind::Select]) {
                    self.error(
                        "E012",
                        "Se esperaba 'seleccionar' / 'select' en la consulta",
                        start,
                        "Usa: seleccionar <expresion>",
                    );
                    return None;
                }
                self.advance();
                let select_expr = self.parse_expression()?;
                let end = select_expr.span();
                Some(Expr::Query {
                    var_name,
                    source: Box::new(source),
                    where_clause,
                    order_by,
                    descending,
                    select_expr: Box::new(select_expr),
                    span: Span::merge(&start, &end),
                })
            }
            TokenKind::EnTiempoCompilacion | TokenKind::Comptime => {
                if !self.check(&[TokenKind::LeftBrace]) {
                    self.error(
                        "E015",
                        "Se esperaba '{' después de 'comptime' / 'en_tiempo_compilacion'",
                        span,
                        "Usa: comptime { expr } o en_tiempo_compilacion { expr }",
                    );
                    return None;
                }
                self.advance();
                let inner = self.parse_expression()?;
                if !self.check(&[TokenKind::RightBrace]) {
                    self.error(
                        "E015",
                        "Se esperaba '}' para cerrar el bloque comptime",
                        span,
                        "Agrega '}' al final de la expresión comptime",
                    );
                    return None;
                }
                self.advance();
                Some(Expr::Comptime {
                    expr: Box::new(inner),
                    span: Span::merge(&span, &self.previous().span),
                })
            }
            _ => {
                let kw = token.kind.as_str();
                if !kw.is_empty() {
                    return self.parse_call_or_ident(kw.to_string(), span);
                }
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

    fn parse_fstring(&mut self, s: &str, span: Span) -> Option<Expr> {
        let mut parts: Vec<Expr> = Vec::new();
        let mut current_lit = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut idx = 0;

        while idx < chars.len() {
            if chars[idx] == '{' {
                if idx + 1 < chars.len() && chars[idx + 1] == '{' {
                    current_lit.push('{');
                    idx += 2;
                    continue;
                }
                if !current_lit.is_empty() {
                    parts.push(Expr::Str {
                        value: current_lit.clone(),
                        span,
                    });
                    current_lit.clear();
                }
                idx += 1; // consume '{'
                let mut expr_str = String::new();
                let mut brace_depth = 1;
                while idx < chars.len() && brace_depth > 0 {
                    if chars[idx] == '{' {
                        brace_depth += 1;
                        expr_str.push('{');
                    } else if chars[idx] == '}' {
                        brace_depth -= 1;
                        if brace_depth > 0 {
                            expr_str.push('}');
                        }
                    } else {
                        expr_str.push(chars[idx]);
                    }
                    idx += 1;
                }

                let expr_trimmed = expr_str.trim();
                if !expr_trimmed.is_empty() {
                    let lexer = lumen_lexer::Lexer::new(expr_trimmed);
                    let (tokens, errors) = lexer.tokenize();
                    if errors.is_empty() {
                        let mut sub_parser = Parser::new(tokens);
                        if let Some(parsed_expr) = sub_parser.parse_expression() {
                            let to_str_call = Expr::Call {
                                callee: Box::new(Expr::Ident {
                                    name: "a_texto".to_string(),
                                    span,
                                }),
                                args: vec![parsed_expr],
                                type_args: Vec::new(),
                                span,
                            };
                            parts.push(to_str_call);
                        }
                    }
                }
            } else if chars[idx] == '}' && idx + 1 < chars.len() && chars[idx + 1] == '}' {
                current_lit.push('}');
                idx += 2;
            } else {
                current_lit.push(chars[idx]);
                idx += 1;
            }
        }

        if !current_lit.is_empty() || parts.is_empty() {
            parts.push(Expr::Str {
                value: current_lit,
                span,
            });
        }

        let mut iter = parts.into_iter();
        let mut result = iter.next()?;
        for p in iter {
            result = Expr::Binary {
                op: BinOp::Add,
                left: Box::new(result),
                right: Box::new(p),
                resolved_method: None,
                span,
            };
        }
        Some(result)
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

    fn parse_type_params(&mut self) -> (Vec<String>, Vec<(String, String)>) {
        if !self.check(&[TokenKind::Less]) {
            return (Vec::new(), Vec::new());
        }
        self.advance();
        let mut names = Vec::new();
        let mut bounds = Vec::new();
        let token = self.advance();
        match token {
            Some(t) => match t.kind {
                TokenKind::Ident(name) => {
                    let bound = self.parse_type_param_bound();
                    names.push(name.clone());
                    if let Some(b) = bound {
                        bounds.push((name, b));
                    }
                    while self.check(&[TokenKind::Comma]) {
                        self.advance();
                        let next = self.advance();
                        match next {
                            Some(t2) => match t2.kind {
                                TokenKind::Ident(s) => {
                                    let bound = self.parse_type_param_bound();
                                    names.push(s.clone());
                                    if let Some(b) = bound {
                                        bounds.push((s, b));
                                    }
                                }
                                _ => {
                                    self.error(
                                        "E011",
                                        "Se esperaba un identificador para el parámetro de tipo",
                                        t2.span,
                                        "Escribe un nombre de parámetro de tipo",
                                    );
                                    return (names, bounds);
                                }
                            },
                            None => return (names, bounds),
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
                    return (names, bounds);
                }
            },
            None => return (names, bounds),
        }
        if !self.eat_type_greater() {
            self.error(
                "E021",
                "Se esperaba '>' para cerrar los parámetros de tipo",
                self.peek().span,
                "Agrega '>' después de los parámetros de tipo",
            );
            return (names, bounds);
        }
        (names, bounds)
    }

    fn parse_type_param_bound(&mut self) -> Option<String> {
        if !self.check(&[TokenKind::Colon]) {
            return None;
        }
        self.advance();
        let name = match &self.peek().kind {
            TokenKind::Ident(s) => s.clone(),
            _ => return None,
        };
        self.advance();
        Some(name)
    }

    fn is_type_arg_start(&self) -> bool {
        if self.pos + 1 >= self.tokens.len() {
            return false;
        }
        let next = &self.tokens[self.pos + 1].kind;
        let is_type_keyword = matches!(
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
        );
        if is_type_keyword {
            return true;
        }
        if self.is_next_type_param() {
            // Also require a matching > followed by ( or { to avoid `x < T {` in conditions
            if let Some(tok) = self.find_token_after_type_args(self.pos) {
                return matches!(tok.kind, TokenKind::LeftParen | TokenKind::LeftBrace);
            }
            return false;
        }
        // Allow any Ident as a potential type arg (user-defined struct type)
        // but only if it's followed by > and then ( or {
        if matches!(next, TokenKind::Ident(_)) {
            let after_type = self.find_token_after_type_args(self.pos);
            if let Some(tok) = after_type {
                return matches!(tok.kind, TokenKind::LeftParen | TokenKind::LeftBrace);
            }
        }
        false
    }

    fn is_next_type_param(&self) -> bool {
        if self.pos + 1 >= self.tokens.len() {
            return false;
        }
        let next = &self.tokens[self.pos + 1];
        match &next.kind {
            TokenKind::Ident(name) => self
                .type_params_stack
                .iter()
                .any(|params| params.contains(name)),
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
        if !self.eat_type_greater() {
            self.error(
                "E021",
                "Se esperaba '>' para cerrar los argumentos de tipo",
                self.peek().span,
                "Agrega '>' después de los argumentos de tipo",
            );
            return None;
        }
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
            return self
                .type_params_stack
                .iter()
                .any(|params| params.contains(name));
        }
        false
    }

    fn is_next_type_in_type_context(&self) -> bool {
        self.is_type_at(self.pos + 1)
    }

    fn eat_type_greater(&mut self) -> bool {
        if self.pending_greater {
            self.pending_greater = false;
            return true;
        }
        if self.check(&[TokenKind::Greater]) {
            self.advance();
            return true;
        }
        if self.check(&[TokenKind::ShiftRight]) {
            self.advance();
            self.pending_greater = true;
            return true;
        }
        false
    }

    fn parse_type(&mut self) -> Option<Type> {
        let token = self.advance()?;
        let mut base_type = match token.kind {
            TokenKind::Numero | TokenKind::Number => Type::Numero,
            TokenKind::Sea | TokenKind::Let => Type::Struct("Infer".to_string()),
            TokenKind::Entero | TokenKind::Integer => Type::Entero,
            TokenKind::Decimal | TokenKind::Float => Type::Decimal,
            TokenKind::Texto | TokenKind::String => Type::Texto,
            TokenKind::Booleano | TokenKind::Boolean => Type::Booleano,
            TokenKind::Lista | TokenKind::Array => {
                if self.check(&[TokenKind::Less]) {
                    self.advance();
                    let inner = self.parse_type()?;
                    if !self.eat_type_greater() {
                        self.error(
                            "E021",
                            "Se esperaba '>' para cerrar el tipo lista",
                            token.span,
                            "Agrega '>' después del tipo interno",
                        );
                        return None;
                    }
                    Type::Lista(Box::new(inner))
                } else {
                    Type::Lista(Box::new(Type::Entero))
                }
            }
            TokenKind::Ident(name) => {
                if name == "list" || name == "Lista" {
                    if self.check(&[TokenKind::Less]) {
                        self.advance();
                        let inner = self.parse_type()?;
                        if !self.eat_type_greater() {
                            self.error(
                                "E021",
                                "Se esperaba '>' para cerrar el tipo lista",
                                token.span,
                                "Agrega '>' después del tipo interno",
                            );
                            return None;
                        }
                        Type::Lista(Box::new(inner))
                    } else {
                        Type::Lista(Box::new(Type::Entero))
                    }
                } else if name == "string" {
                    Type::Texto
                } else if self.check(&[TokenKind::Less]) && self.is_next_type_in_type_context() {
                    let args = self.parse_type_args()?;
                    Type::GenericStruct { name, args }
                } else if name == "cualquiera" || name == "any" {
                    Type::Numero
                } else {
                    Type::Struct(name)
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
                if !self.eat_type_greater() {
                    self.error(
                        "E021",
                        "Se esperaba '>' para cerrar el tipo resultado",
                        token.span,
                        "Agrega '>' después del tipo de error",
                    );
                    return None;
                }
                Type::Resultado {
                    ok: Box::new(ok),
                    err: Box::new(err),
                }
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
                    types.into_iter().next().unwrap()
                } else {
                    Type::Tuple(types)
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
                if !self.eat_type_greater() {
                    self.error(
                        "E021",
                        "Se esperaba '>' para cerrar el tipo opcional",
                        token.span,
                        "Agrega '>' después del tipo interno",
                    );
                    return None;
                }
                Type::Opcion(Box::new(inner))
            }
            TokenKind::Impl => {
                if self.check_ident() {
                    let trait_name = self.expect_ident()?;
                    Type::ImplTrait(trait_name)
                } else {
                    self.error(
                        "E011",
                        "Se esperaba un nombre de rasgo después de 'impl'",
                        token.span,
                        "Escribe el nombre del rasgo, ej: 'impl Comparable'",
                    );
                    return None;
                }
            }
            TokenKind::Prestado | TokenKind::Borrowed => {
                let mutable = if self.check(&[TokenKind::Mut, TokenKind::Mutable]) {
                    self.advance();
                    true
                } else {
                    false
                };
                let inner = self.parse_type()?;
                Type::Prestado {
                    inner: Box::new(inner),
                    mutable,
                }
            }
            TokenKind::Dueno | TokenKind::Owner => {
                let inner = self.parse_type()?;
                Type::Dueno(Box::new(inner))
            }
            _ => return None,
        };
        // Azúcar sintáctico para tipos opcionales: `texto?`, `entero?`, `Punto?` → `opcion<texto>`
        while self.check(&[TokenKind::Question]) {
            self.advance();
            base_type = Type::Opcion(Box::new(base_type));
        }
        Some(base_type)
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

    /// For-init is a typed declaration: `entero i = 0`, `lista<entero> l = []`, custom type `Punto p`.
    fn is_for_init_decl(&self) -> bool {
        if self.is_type_at(self.pos) {
            return true;
        }
        if self.check_ident() && self.check_ident_next() {
            return true;
        }
        self.check_ident_next_is_generic_type()
    }

    /// `para [tipo] ident (en|in) expr { ... }` — foreach without parens.
    fn is_foreach_like(&self) -> bool {
        let mut p = self.pos + 1;
        if p >= self.tokens.len() {
            return false;
        }
        if self.is_type_at(p) {
            p += 1;
        }
        if p >= self.tokens.len() {
            return false;
        }
        if !matches!(self.tokens[p].kind, TokenKind::Ident(_)) {
            return false;
        }
        p += 1;
        if p >= self.tokens.len() {
            return false;
        }
        matches!(self.tokens[p].kind, TokenKind::En | TokenKind::In)
    }

    /// Starting from an Ident at `start_pos` followed by `<`, find the token after the matching `>`.
    /// Aborts (returns None) if it hits a delimiter (`(` `)` `{` `}` `;`) before the matching `>`,
    /// so expressions like `i < veces {` or `i < largo(arr)` are not mistaken for generics.
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
                TokenKind::LeftParen
                | TokenKind::RightParen
                | TokenKind::LeftBrace
                | TokenKind::RightBrace
                | TokenKind::Semicolon => return None,
                _ => {}
            }
            i += 1;
        }
        None
    }

    // --- Helpers ---

    fn advance(&mut self) -> Option<Token> {
        if self.is_at_end() {
            return self.tokens.get(self.pos).cloned();
        }
        self.pos += 1;
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

    fn is_guard_let(&self) -> bool {
        let mut i = self.pos;
        let mut brace_depth = 0;
        let mut paren_depth = 0;
        let mut bracket_depth = 0;
        while i < self.tokens.len() {
            let kind = &self.tokens[i].kind;
            match kind {
                TokenKind::LeftBrace => brace_depth += 1,
                TokenKind::RightBrace => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                }
                TokenKind::LeftParen => paren_depth += 1,
                TokenKind::RightParen => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                TokenKind::LeftBracket => bracket_depth += 1,
                TokenKind::RightBracket => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    }
                }
                TokenKind::Semicolon => {
                    if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 {
                        return false;
                    }
                }
                _ if token_matches(kind, &TokenKind::Sino)
                    && brace_depth == 0
                    && paren_depth == 0
                    && bracket_depth == 0 =>
                {
                    return true;
                }
                _ => {}
            }
            i += 1;
        }
        false
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

    fn is_keyword(kind: &TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Si
                | TokenKind::Sino
                | TokenKind::Mientras
                | TokenKind::Para
                | TokenKind::Funcion
                | TokenKind::Retornar
                | TokenKind::Verdadero
                | TokenKind::Falso
                | TokenKind::Numero
                | TokenKind::Entero
                | TokenKind::Decimal
                | TokenKind::Texto
                | TokenKind::Booleano
                | TokenKind::Imprimir
                | TokenKind::Leer
                | TokenKind::Lista
                | TokenKind::Romper
                | TokenKind::Continuar
                | TokenKind::Elegir
                | TokenKind::Caso
                | TokenKind::Defecto
                | TokenKind::Estructura
                | TokenKind::Importar
                | TokenKind::Como
                | TokenKind::Resultado
                | TokenKind::Exito
                | TokenKind::ErrKeyword
                | TokenKind::Intentar
                | TokenKind::En
                | TokenKind::Enum
                | TokenKind::Opcion
                | TokenKind::Algun
                | TokenKind::Ninguno
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Function
                | TokenKind::Return
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Number
                | TokenKind::Integer
                | TokenKind::Float
                | TokenKind::String
                | TokenKind::Boolean
                | TokenKind::Print
                | TokenKind::Read
                | TokenKind::Array
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Match
                | TokenKind::Case
                | TokenKind::Default
                | TokenKind::Struct
                | TokenKind::Import
                | TokenKind::As
                | TokenKind::Result
                | TokenKind::Ok
                | TokenKind::Err
                | TokenKind::Try
                | TokenKind::In
                | TokenKind::Option
                | TokenKind::Some
                | TokenKind::None
                | TokenKind::Const
                | TokenKind::Rasgo
                | TokenKind::Trait
                | TokenKind::Impl
        )
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
            TokenKind::Rasgo | TokenKind::Trait => {
                self.advance();
                Some("rasgo".to_string())
            }
            _ => {
                let kw = token.kind.as_str();
                if !kw.is_empty() {
                    let kw = kw.to_string();
                    self.advance();
                    Some(kw)
                } else {
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
    }

    fn expect_ident(&mut self) -> Option<String> {
        let token = self.advance()?;
        match &token.kind {
            TokenKind::Ident(s) => Some(s.clone()),
            _ => {
                let kw = token.kind.as_str();
                let message = if !kw.is_empty() {
                    format!("La palabra '{}' es una palabra reservada del lenguaje y no puede usarse como identificador", kw)
                } else {
                    "Se esperaba un nombre de variable".to_string()
                };
                let suggestion = if !kw.is_empty() {
                    format!("Elige otro nombre para tu variable (p. ej. '{}_val', 'res', 'dato', 'valor')", kw)
                } else {
                    "Escribe un identificador".to_string()
                };
                self.error("E011", message, token.span, suggestion);
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
                | TokenKind::Result
                | TokenKind::Rasgo
                | TokenKind::Trait
                | TokenKind::Impl => return,
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
                | (TokenKind::Rasgo, TokenKind::Trait)
                | (TokenKind::Trait, TokenKind::Rasgo)
                | (TokenKind::Consultar, TokenKind::Query)
                | (TokenKind::Query, TokenKind::Consultar)
                | (TokenKind::Donde, TokenKind::Where)
                | (TokenKind::Where, TokenKind::Donde)
                | (TokenKind::OrdenarPor, TokenKind::OrderBy)
                | (TokenKind::OrderBy, TokenKind::OrdenarPor)
                | (TokenKind::Seleccionar, TokenKind::Select)
                | (TokenKind::Select, TokenKind::Seleccionar)
                | (TokenKind::Descendente, TokenKind::Descending)
                | (TokenKind::Descending, TokenKind::Descendente)
                | (TokenKind::Ascendente, TokenKind::Ascending)
                | (TokenKind::Ascending, TokenKind::Ascendente)
                | (TokenKind::Atrapar, TokenKind::Catch)
                | (TokenKind::Catch, TokenKind::Atrapar)
                | (TokenKind::Prestado, TokenKind::Borrowed)
                | (TokenKind::Borrowed, TokenKind::Prestado)
                | (TokenKind::Dueno, TokenKind::Owner)
                | (TokenKind::Owner, TokenKind::Dueno)
                | (TokenKind::Mut, TokenKind::Mutable)
                | (TokenKind::Mutable, TokenKind::Mut)
                | (TokenKind::EnTiempoCompilacion, TokenKind::Comptime)
                | (TokenKind::Comptime, TokenKind::EnTiempoCompilacion)
                | (TokenKind::Ensamblador, TokenKind::Asm)
                | (TokenKind::Asm, TokenKind::Ensamblador)
                | (TokenKind::BloqueC, TokenKind::CBlock)
                | (TokenKind::CBlock, TokenKind::BloqueC)
                | (TokenKind::BloqueRust, TokenKind::RustBlock)
                | (TokenKind::RustBlock, TokenKind::BloqueRust)
                | (TokenKind::Puro, TokenKind::Pure)
                | (TokenKind::Pure, TokenKind::Puro)
                | (TokenKind::GrupoTareas, TokenKind::TaskGroup)
                | (TokenKind::TaskGroup, TokenKind::GrupoTareas)
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
            | Expr::Cast { span, .. }
            | Expr::List { span, .. }
            | Expr::Range { span, .. }
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
            | Expr::TupleAccess { span, .. }
            | Expr::SafeFieldAccess { span, .. }
            | Expr::Elvis { span, .. }
            | Expr::Comprehension { span, .. }
            | Expr::Query { span, .. }
            | Expr::Comptime { span, .. }
            | Expr::Ternary { span, .. } => *span,
            Expr::Esperar { span, .. } => *span,
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
        if let DeclOrStmt::Decl(Decl::Function {
            name, type_params, ..
        }) = &program[0]
        {
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
        if let DeclOrStmt::Decl(Decl::Struct {
            name, type_params, ..
        }) = &program[0]
        {
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
    fn test_parse_range_pattern() {
        let source =
            "elegir (n) { caso 0..5: imprimir(\"bajo\"); caso 5..=10: imprimir(\"medio\"); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Stmt(Stmt::Match { arms, .. }) = &program[0] {
            assert_eq!(arms.len(), 2);
            assert!(matches!(
                &arms[0].value,
                Expr::Range {
                    inclusive: false,
                    ..
                }
            ));
            assert!(matches!(
                &arms[1].value,
                Expr::Range {
                    inclusive: true,
                    ..
                }
            ));
        } else {
            panic!("Expected Match statement");
        }
    }

    #[test]
    fn test_parse_or_patterns() {
        let source = "elegir (c) { caso Color::Rojo | Color::Verde: imprimir(\"calido\"); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Stmt(Stmt::Match { arms, .. }) = &program[0] {
            assert_eq!(arms.len(), 1);
            assert_eq!(arms[0].alt_values.len(), 1);
            assert!(matches!(&arms[0].alt_values[0], Expr::EnumCtor { .. }));
        } else {
            panic!("Expected Match statement");
        }
    }

    #[test]
    fn test_parse_range_expr() {
        let source = "lista<entero> r = 0..10;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable {
            init: Some(init), ..
        }) = &program[0]
        {
            assert!(matches!(init.as_ref(), Expr::Range { .. }));
        } else {
            panic!("Expected Variable declaration with init");
        }
    }

    #[test]
    fn test_parse_comparison_still_works() {
        let source = "x < y;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
        if let DeclOrStmt::Stmt(Stmt::Expr { expr, .. }) = &program[0] {
            assert!(matches!(
                expr.as_ref(),
                Expr::Binary {
                    op: BinOp::Less,
                    ..
                }
            ));
        } else {
            panic!("Expected Expr statement");
        }
    }

    #[test]
    fn test_parse_pipe_operator() {
        let source = "entero r = 10 |> duplicar() |> sumar(5);";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_parse_optional_type_sugar() {
        let source = "texto? nombre = algun(\"LUMEN\");";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable { var_type, .. }) = &program[0] {
            assert!(matches!(var_type, Type::Opcion(_)));
        } else {
            panic!("Expected Variable declaration");
        }
    }

    #[test]
    fn test_parse_list_comprehension() {
        let source = "lista<entero> pares = [x * 2 para x en nums si x % 2 == 0];";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable {
            init: Some(init), ..
        }) = &program[0]
        {
            assert!(matches!(init.as_ref(), Expr::Comprehension { .. }));
        } else {
            panic!("Expected Variable declaration with comprehension init");
        }
    }

    #[test]
    fn test_parse_linq_query_spanish() {
        let source = "lista<entero> r = consultar x en nums donde x > 5 seleccionar x * 2;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable {
            init: Some(init), ..
        }) = &program[0]
        {
            assert!(matches!(init.as_ref(), Expr::Query { .. }));
        } else {
            panic!("Expected Variable declaration with Query init");
        }
    }

    #[test]
    fn test_parse_linq_query_english() {
        let source = "array<integer> r = query x in nums where x > 5 select x * 2;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable {
            init: Some(init), ..
        }) = &program[0]
        {
            assert!(matches!(init.as_ref(), Expr::Query { .. }));
        } else {
            panic!("Expected Variable declaration with Query init");
        }
    }

    #[test]
    fn test_new_parser_for_with_type() {
        let source = "para (entero i = 0; i < 5; i = i + 1) { imprimir(i); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
        assert!(matches!(&program[0], DeclOrStmt::Stmt(Stmt::For { .. })));
    }

    #[test]
    fn test_new_parser_for_without_type() {
        let source = "para (i = 0; i < 10; i = i + 1) { imprimir(i); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert!(matches!(&program[0], DeclOrStmt::Stmt(Stmt::For { .. })));
    }

    #[test]
    fn test_new_parser_for_without_type_inferred() {
        let source = "para (i = 0; i < 5; i = i + 1) { }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Stmt(Stmt::For { init, .. }) = &program[0] {
            if let Decl::Variable { var_type, name, .. } = init.as_ref() {
                assert_eq!(name, "i");
                assert_eq!(*var_type, Type::Struct("Infer".to_string()));
            } else {
                panic!("Expected Infer variable");
            }
        }
    }

    #[test]
    fn test_new_parser_elegir_or_pattern_simple() {
        let source = "elegir (x) { caso 1 | 2: imprimir(\"a\"); defecto: imprimir(\"b\"); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Stmt(Stmt::Match { arms, .. }) = &program[0] {
            assert_eq!(arms.len(), 1);
            assert_eq!(arms[0].alt_values.len(), 1);
        } else {
            panic!("Expected Match");
        }
    }

    #[test]
    fn test_new_parser_elegir_or_pattern_enum() {
        let source = "elegir (c) { caso Color::Rojo | Color::Verde: imprimir(\"calido\"); caso Color::Azul: imprimir(\"frio\"); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Stmt(Stmt::Match { arms, .. }) = &program[0] {
            assert_eq!(arms.len(), 2);
            assert_eq!(arms[0].alt_values.len(), 1);
            assert!(matches!(&arms[0].alt_values[0], Expr::EnumCtor { .. }));
        } else {
            panic!("Expected Match");
        }
    }

    #[test]
    fn test_new_parser_elegir_range_exclusive() {
        let source = "elegir (n) { caso 0..5: imprimir(\"bajo\"); defecto: imprimir(\"alto\"); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Stmt(Stmt::Match { arms, .. }) = &program[0] {
            assert!(matches!(
                &arms[0].value,
                Expr::Range {
                    inclusive: false,
                    ..
                }
            ));
        } else {
            panic!("Expected Match");
        }
    }

    #[test]
    fn test_new_parser_elegir_range_inclusive() {
        let source = "elegir (n) { caso 0..=5: imprimir(\"bajo\"); defecto: imprimir(\"alto\"); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Stmt(Stmt::Match { arms, .. }) = &program[0] {
            assert!(matches!(
                &arms[0].value,
                Expr::Range {
                    inclusive: true,
                    ..
                }
            ));
        } else {
            panic!("Expected Match");
        }
    }

    #[test]
    fn test_new_parser_if_let_algun() {
        let source = "opcion<entero> opt = algun(1); si sea algun(x) = opt { imprimir(x); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 2);
        assert!(matches!(&program[1], DeclOrStmt::Stmt(Stmt::IfLet { .. })));
    }

    #[test]
    fn test_new_parser_if_let_exito() {
        let source = "resultado<entero, texto> r = exito(5); si sea exito(v) = r { imprimir(v); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert!(matches!(&program[1], DeclOrStmt::Stmt(Stmt::IfLet { .. })));
    }

    #[test]
    fn test_new_parser_funcion_vacio_principal() {
        let source = "funcion vacio principal() { imprimir(\"hola\"); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
        assert!(matches!(
            &program[0],
            DeclOrStmt::Decl(Decl::Function { .. })
        ));
    }

    #[test]
    fn test_new_parser_funcion_entero_principal() {
        let source = "funcion entero principal() { retornar 0; }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Function {
            name, return_type, ..
        }) = &program[0]
        {
            assert_eq!(name, "principal");
            assert_eq!(*return_type, Type::Entero);
        } else {
            panic!("Expected Function");
        }
    }

    #[test]
    fn test_new_parser_array_index_assign() {
        let source = "lista<entero> arr = [1,2,3]; arr[0] = 5;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 2);
        assert!(matches!(
            &program[1],
            DeclOrStmt::Stmt(Stmt::ArraySet { .. })
        ));
    }

    #[test]
    fn test_new_parser_cstyle_array_decl() {
        let source = "entero arr[] = [1,2,3];";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable { var_type, name, .. }) = &program[0] {
            assert_eq!(name, "arr");
            assert_eq!(*var_type, Type::Lista(Box::new(Type::Entero)));
        } else {
            panic!("Expected Variable");
        }
    }

    #[test]
    fn test_new_parser_como_cast_entero() {
        let source = "numero x = 5; entero y = x como entero;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable {
            init: Some(init), ..
        }) = &program[1]
        {
            assert!(matches!(init.as_ref(), Expr::Cast { .. }));
            if let Expr::Cast { cast_type, .. } = init.as_ref() {
                assert_eq!(*cast_type, Type::Entero);
            }
        } else {
            panic!("Expected Cast");
        }
    }

    #[test]
    fn test_new_parser_como_cast_texto() {
        let source = "numero x = 5; texto t = x como texto;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable {
            init: Some(init), ..
        }) = &program[1]
        {
            assert!(matches!(init.as_ref(), Expr::Cast { .. }));
        } else {
            panic!("Expected Cast");
        }
    }

    #[test]
    fn test_new_parser_pipe_simple() {
        let source = "entero r = 10 |> duplicar();";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_new_parser_pipe_chain() {
        let source = "entero r = 10 |> duplicar() |> sumar(5);";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable {
            init: Some(init), ..
        }) = &program[0]
        {
            assert!(matches!(init.as_ref(), Expr::Call { .. }));
        } else {
            panic!("Expected Call chain");
        }
    }

    #[test]
    fn test_new_parser_fstring_simple() {
        let source = r#"texto s = f"hola mundo";"#;
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_new_parser_fstring_with_expr() {
        let source = r#"texto nombre = "Lumen"; texto s = f"hola {nombre}";"#;
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 2);
        if let DeclOrStmt::Decl(Decl::Variable {
            init: Some(init), ..
        }) = &program[1]
        {
            assert!(matches!(
                init.as_ref(),
                Expr::Binary { .. } | Expr::Call { .. } | Expr::Str { .. }
            ));
        } else {
            panic!("Expected Variable with fstring");
        }
    }

    #[test]
    fn test_new_parser_generic_nested_lista() {
        let source = "lista<lista<entero>> matriz = [[1,2],[3,4]];";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable { var_type, .. }) = &program[0] {
            assert_eq!(
                *var_type,
                Type::Lista(Box::new(Type::Lista(Box::new(Type::Entero))))
            );
        } else {
            panic!("Expected nested lista");
        }
    }

    #[test]
    fn test_new_parser_generic_opcion_resultado_nested() {
        let source = "opcion<resultado<entero, texto>> r = algun(exito(42));";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable { var_type, .. }) = &program[0] {
            assert!(matches!(var_type, Type::Opcion(_)));
        } else {
            panic!("Expected opcion<resultado>");
        }
    }

    #[test]
    fn test_new_parser_generic_struct_nested() {
        let source = "Par<lista<entero>, texto> p = Par<lista<entero>, texto> { primero: [1,2], segundo: \"hola\" };";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn test_new_parser_elegir_or_multiple() {
        let source = "elegir (x) { caso 1 | 2 | 3: imprimir(\"a\"); caso 4 | 5: imprimir(\"b\"); defecto: imprimir(\"c\"); }";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Stmt(Stmt::Match { arms, default, .. }) = &program[0] {
            assert_eq!(arms.len(), 2);
            assert_eq!(arms[0].alt_values.len(), 2);
            assert_eq!(arms[1].alt_values.len(), 1);
            assert!(default.is_some());
        } else {
            panic!("Expected Match");
        }
    }

    #[test]
    fn test_new_parser_range_expr_simple() {
        let source = "lista<entero> r = 0..10; lista<entero> r2 = 0..=10;";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        assert_eq!(program.len(), 2);
        for prog in &program {
            if let DeclOrStmt::Decl(Decl::Variable {
                init: Some(init), ..
            }) = prog
            {
                assert!(matches!(init.as_ref(), Expr::Range { .. }));
            } else {
                panic!("Expected Range");
            }
        }
    }

    #[test]
    fn test_new_parser_cstyle_array_empty() {
        let source = "entero vacio[] = [];";
        let (program, errors) = parse(source);
        assert!(errors.is_empty(), "Parse errors: {:?}", errors);
        if let DeclOrStmt::Decl(Decl::Variable { var_type, .. }) = &program[0] {
            assert_eq!(*var_type, Type::Lista(Box::new(Type::Entero)));
        } else {
            panic!("Expected Variable");
        }
    }
}
