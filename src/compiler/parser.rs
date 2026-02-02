// src/compiler/parser.rs
use crate::compiler::ast::*;
use crate::compiler::lexer::{Token, TokenInfo};

pub struct Parser {
    tokens: Vec<TokenInfo>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenInfo>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> &TokenInfo {
        self.tokens.get(self.pos).unwrap_or(&TokenInfo { token: Token::Eof, line: 0, col: 0 })
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current().token, Token::Eof)
    }

    fn skip_newlines(&mut self) {
        while matches!(self.current().token, Token::Newline) {
            self.advance();
        }
    }

    fn token_label(tok: &Token) -> String {
        match tok {
            Token::Identifier(s) => format!("Identifier({})", s),
            Token::Number(n) => format!("Number({})", n),
            Token::String(s) => format!("String(\"{}\")", s),
            other => format!("{:?}", other),
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        self.skip_newlines();

        while !self.is_at_end() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.skip_newlines();
        }

        let mut p = Program::new();
        p.statements = statements;
        Ok(p)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        self.skip_newlines();

        let cur = self.current().clone();
        match cur.token {
            Token::Numero => {
                // Declaración: numero <id> = <expr>   (also 'number' mapped by lexer)
                self.advance(); // consume 'numero'
                let cur2 = self.current().clone();
                let name = match &cur2.token {
                    Token::Identifier(s) => s.clone(),
                    other => return Err(format!("[{}:{}] Se esperaba identificador después de 'numero', encontré: {}", cur2.line, cur2.col, Parser::token_label(&other))),
                };
                self.advance();

                // esperar '='
                let cur3 = self.current().clone();
                match cur3.token {
                    Token::Assign => { self.advance(); }
                    other => return Err(format!("[{}:{}] Se esperaba '=' en declaración de variable, encontré: {}", cur3.line, cur3.col, Parser::token_label(&other))),
                }

                let value = self.parse_expression()?;
                Ok(Statement::VarDecl { name, value })
            }

            Token::Identifier(_) => {
                // Identifier start -> assignment: id = expr
                if let Token::Identifier(ref name) = cur.token {
                    let id_name = name.clone();
                    self.advance();
                    let cur2 = self.current().clone();
                    match cur2.token {
                        Token::Assign => {
                            self.advance();
                            let value = self.parse_expression()?;
                            Ok(Statement::Assignment { target: id_name, value })
                        }
                        other => Err(format!("[{}:{}] Se esperaba '=' después del identificador, encontré: {}", cur2.line, cur2.col, Parser::token_label(&other))),
                    }
                } else {
                    unreachable!()
                }
            }

            Token::Imprimir => {
                // Support both: imprimir(expr)  OR  imprimir expr  OR print(...)
                self.advance(); // consume 'imprimir'

                // If next is '(' parse parenthesized expression
                if matches!(self.current().token, Token::LParen) {
                    self.advance();
                    let expr = self.parse_expression()?;
                    let cur3 = self.current().clone();
                    match cur3.token {
                        Token::RParen => { self.advance(); }
                        other => return Err(format!("[{}:{}] Se esperaba ')' después de expresión en 'imprimir', encontré: {}", cur3.line, cur3.col, Parser::token_label(&other))),
                    }
                    Ok(Statement::Print { expr })
                } else {
                    // parse expression directly
                    let expr = self.parse_expression()?;
                    Ok(Statement::Print { expr })
                }
            }

            Token::Si => {
                self.advance(); // consume 'si'
                let cur2 = self.current().clone();
                match cur2.token {
                    Token::LParen => self.advance(),
                    other => return Err(format!("[{}:{}] Se esperaba '(' después de 'si', encontré: {}", cur2.line, cur2.col, Parser::token_label(&other))),
                }
                let condition = self.parse_expression()?;
                let cur3 = self.current().clone();
                match cur3.token {
                    Token::RParen => self.advance(),
                    other => return Err(format!("[{}:{}] Se esperaba ')' después de condición en 'si', encontré: {}", cur3.line, cur3.col, Parser::token_label(&other))),
                }
                let then_body = self.parse_block()?;

                let else_body = if matches!(self.current().token, Token::Sino) {
                    self.advance(); // consume 'sino'
                    Some(self.parse_block()?)
                } else {
                    None
                };

                Ok(Statement::If { condition, then_body, else_body })
            }

            Token::Mientras => {
                self.advance(); // consume 'mientras'
                let cur2 = self.current().clone();
                match cur2.token {
                    Token::LParen => self.advance(),
                    other => return Err(format!("[{}:{}] Se esperaba '(' después de 'mientras', encontré: {}", cur2.line, cur2.col, Parser::token_label(&other))),
                }
                let condition = self.parse_expression()?;
                let cur3 = self.current().clone();
                match cur3.token {
                    Token::RParen => self.advance(),
                    other => return Err(format!("[{}:{}] Se esperaba ')' después de condición en 'mientras', encontré: {}", cur3.line, cur3.col, Parser::token_label(&other))),
                }
                let body = self.parse_block()?;
                Ok(Statement::While { condition, body })
            }

            Token::Import => {
                // import <identifier>  OR  import "module"
                self.advance(); // consume 'import'
                let cur2 = self.current().clone();
                let module = match cur2.token {
                    Token::Identifier(ref s) => s.clone(),
                    Token::String(ref s) => s.clone(),
                    other => return Err(format!("[{}:{}] Se esperaba nombre de módulo (identificador o string) después de 'import', encontré: {}", cur2.line, cur2.col, Parser::token_label(&other))),
                };
                self.advance();
                Ok(Statement::Import { module })
            }

            Token::Eof => Err(format!("[{}:{}] EOF inesperado al parsear sentencia", cur.line, cur.col)),

            other => Err(format!("[{}:{}] Token inesperado al inicio de la sentencia: {:?}", cur.line, cur.col, other)),
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        // Espera '{' <statements> '}'
        let cur = self.current().clone();
        match cur.token {
            Token::LBrace => { self.advance(); }
            other => return Err(format!("[{}:{}] Se esperaba '{{' para iniciar bloque, encontré: {}", cur.line, cur.col, Parser::token_label(&other))),
        }

        let mut stmts = Vec::new();
        self.skip_newlines();
        while !matches!(self.current().token, Token::RBrace | Token::Eof) {
            let s = self.parse_statement()?;
            stmts.push(s);
            self.skip_newlines();
        }

        let cur2 = self.current().clone();
        match cur2.token {
            Token::RBrace => { self.advance(); }
            other => return Err(format!("[{}:{}] Se esperaba '}}' para cerrar bloque, encontré: {}", cur2.line, cur2.col, Parser::token_label(&other))),
        }

        Ok(stmts)
    }

    // ---------- Expressions (precedence climbing / recursive descent) ----------

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_comparison()?;

        while matches!(self.current().token, Token::Equal) {
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expr::BinOp { left: Box::new(expr), op: BinOperator::Eq, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_term()?;

        while matches!(self.current().token, Token::Less | Token::Greater) {
            let op = match self.current().token {
                Token::Less => BinOperator::Lt,
                Token::Greater => BinOperator::Gt,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_term()?;
            expr = Expr::BinOp { left: Box::new(expr), op, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;

        while matches!(self.current().token, Token::Plus | Token::Minus) {
            let op = match self.current().token {
                Token::Plus => BinOperator::Add,
                Token::Minus => BinOperator::Sub,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_factor()?;
            expr = Expr::BinOp { left: Box::new(expr), op, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;

        while matches!(self.current().token, Token::Star | Token::Slash) {
            let op = match self.current().token {
                Token::Star => BinOperator::Mul,
                Token::Slash => BinOperator::Div,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expr::BinOp { left: Box::new(expr), op, right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if matches!(self.current().token, Token::Minus) {
            // -expr  => 0 - expr
            self.advance();
            let right = self.parse_unary()?;
            Ok(Expr::BinOp { left: Box::new(Expr::Number(0)), op: BinOperator::Sub, right: Box::new(right) })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let cur = self.current().clone();
        match cur.token {
            Token::Number(n) => {
                let val = n;
                self.advance();
                Ok(Expr::Number(val))
            }
            Token::String(ref s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::String(val))
            }
            Token::Identifier(ref s) => {
                let name = s.clone();
                self.advance();
                Ok(Expr::Variable(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                let cur2 = self.current().clone();
                match cur2.token {
                    Token::RParen => { self.advance(); }
                    other => return Err(format!("[{}:{}] Se esperaba ')' después de expresión entre paréntesis, encontré: {}", cur2.line, cur2.col, Parser::token_label(&other))),
                }
                Ok(expr)
            }
            _ => Err(format!("[{}:{}] Token inesperado al inicio de la expresión: {:?}", cur.line, cur.col, cur.token)),
        }
    }
}
