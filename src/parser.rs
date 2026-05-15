use crate::ast::{Block, Expr, Pattern, TypeVariant};
use crate::token::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Block, String> {
        let mut block = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            let expr = self.parse_statement()?;
            block.push(expr);
            self.skip_newlines();
        }
        Ok(block)
    }

    fn skip_newlines(&mut self) {
        while self.peek_kind() == Some(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(self.tokens.last().unwrap())
    }

    fn peek_kind(&self) -> Option<&TokenKind> {
        if self.pos >= self.tokens.len() {
            None
        } else {
            Some(&self.tokens[self.pos].kind)
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or_else(|| {
            Token::new(TokenKind::Eof, 0, 0)
        });
        self.pos += 1;
        tok
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        let tok = self.advance();
        if tok.kind == kind {
            Ok(tok)
        } else {
            Err(format!("Expected {}, got {} at line {}", kind, tok.kind, tok.span.line))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Ident(name) => Ok(name),
            _ => Err(format!("Expected identifier, got {} at line {}", tok.kind, tok.span.line)),
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind() == Some(&TokenKind::Eof)
    }

    fn parse_statement(&mut self) -> Result<Expr, String> {
        match self.peek_kind() {
            Some(TokenKind::Let) => self.parse_let(),
            Some(TokenKind::Var) => self.parse_var(),
            Some(TokenKind::Return) => self.parse_return(),
            Some(TokenKind::While) => self.parse_while(),
            Some(TokenKind::For) => self.parse_for(),
            Some(TokenKind::Match) => self.parse_match(),
            Some(TokenKind::Type) => self.parse_type_decl(),
            Some(TokenKind::Load) => self.parse_load(),
            Some(TokenKind::Do) => self.parse_do(),
            Some(TokenKind::Fn) => {
                if matches!(self.tokens.get(self.pos + 1), Some(t) if matches!(t.kind, TokenKind::Ident(_))) {
                    self.parse_fn_decl()
                } else {
                    self.parse_expression()
                }
            }
            _ => self.parse_expression(),
        }
    }

    fn parse_let(&mut self) -> Result<Expr, String> {
        self.advance();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Assign)?;
        self.skip_newlines();
        let value = self.parse_expression()?;
        Ok(Expr::Let { name, value: Box::new(value) })
    }

    fn parse_var(&mut self) -> Result<Expr, String> {
        self.advance();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Assign)?;
        self.skip_newlines();
        let value = self.parse_expression()?;
        Ok(Expr::Var { name, value: Box::new(value) })
    }

    fn parse_return(&mut self) -> Result<Expr, String> {
        self.advance();
        self.skip_newlines();
        let value = self.parse_expression()?;
        Ok(Expr::Return(Box::new(value)))
    }

    fn parse_fn_decl(&mut self) -> Result<Expr, String> {
        self.advance();
        let name = self.expect_ident()?;
        self.expect(TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RParen)?;
        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Fn {
            name: Some(name),
            params,
            body,
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if self.peek_kind() == Some(&TokenKind::RParen) {
            return Ok(params);
        }
        params.push(self.expect_ident()?);
        loop {
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
            } else if self.peek_kind() == Some(&TokenKind::Newline) {
                self.skip_newlines();
                if self.peek_kind() == Some(&TokenKind::RParen) { break; }
            } else {
                break;
            }
            if self.peek_kind() == Some(&TokenKind::RParen) { break; }
            params.push(self.expect_ident()?);
        }
        Ok(params)
    }

    fn parse_while(&mut self) -> Result<Expr, String> {
        self.advance();
        let condition = self.parse_expression()?;
        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::While {
            condition: Box::new(condition),
            body,
        })
    }

    fn parse_for(&mut self) -> Result<Expr, String> {
        self.advance();
        let name = self.expect_ident()?;
        self.expect(TokenKind::In)?;
        let iterable = self.parse_expression()?;
        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::For {
            name,
            iterable: Box::new(iterable),
            body,
        })
    }

    fn parse_match(&mut self) -> Result<Expr, String> {
        self.advance();
        let subject = self.parse_expression()?;
        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;
        self.skip_newlines();
        let mut arms = Vec::new();
        while self.peek_kind() != Some(&TokenKind::RBrace) {
            let pattern = self.parse_pattern()?;
            self.expect(TokenKind::FatArrow)?;
            self.skip_newlines();
            let body = self.parse_match_arm_body()?;
            self.skip_newlines();
            arms.push((pattern, body));
            // commas are optional between arms
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Match {
            subject: Box::new(subject),
            arms,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        match self.peek_kind() {
            Some(TokenKind::Underscore) if self.peek().kind == TokenKind::Ident("_".into()) => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Some(TokenKind::Int(n)) => { let n = *n; self.advance(); Ok(Pattern::Int(n)) }
            Some(TokenKind::Float(n)) => { let n = *n; self.advance(); Ok(Pattern::Float(n)) }
            Some(TokenKind::String(s)) => { let s = s.clone(); self.advance(); Ok(Pattern::String(s)) }
            Some(TokenKind::Bool(b)) => { let b = *b; self.advance(); Ok(Pattern::Bool(b)) }
            Some(TokenKind::Null) => { self.advance(); Ok(Pattern::Null) }
            Some(TokenKind::Ident(name)) => {
                let name = name.clone();
                self.advance();
                if self.peek_kind() == Some(&TokenKind::LParen) {
                    self.advance();
                    let mut bindings = Vec::new();
                    if self.peek_kind() != Some(&TokenKind::RParen) {
                        bindings.push(self.expect_ident()?);
                        loop {
                            if self.peek_kind() == Some(&TokenKind::Comma) {
                                self.advance();
                            } else if self.peek_kind() == Some(&TokenKind::Newline) {
                                self.skip_newlines();
                                if self.peek_kind() == Some(&TokenKind::RParen) { break; }
                            } else {
                                break;
                            }
                            if self.peek_kind() == Some(&TokenKind::RParen) { break; }
                            bindings.push(self.expect_ident()?);
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    Ok(Pattern::Constructor { name, bindings })
                } else {
                    Ok(Pattern::Ident(name))
                }
            }
            Some(TokenKind::LBracket) => {
                self.advance();
                self.skip_newlines();
                let mut pats = Vec::new();
                if self.peek_kind() != Some(&TokenKind::RBracket) {
                    pats.push(self.parse_pattern()?);
                    loop {
                        self.skip_newlines();
                        if self.peek_kind() == Some(&TokenKind::Comma) {
                            self.advance();
                            self.skip_newlines();
                            if self.peek_kind() == Some(&TokenKind::RBracket) { break; }
                        } else {
                            break;
                        }
                        pats.push(self.parse_pattern()?);
                    }
                }
                self.skip_newlines();
                self.expect(TokenKind::RBracket)?;
                Ok(Pattern::List(pats))
            }
            Some(TokenKind::DotDotDot) => {
                self.advance();
                Ok(Pattern::Rest)
            }
            _ => Err(format!("Expected pattern, got {:?} at line {}", self.peek_kind(), self.peek().span.line)),
        }
    }

    fn parse_match_arm_body(&mut self) -> Result<Block, String> {
        if self.peek_kind() == Some(&TokenKind::LBrace) {
            self.advance();
            let body = self.parse_block()?;
            self.expect(TokenKind::RBrace)?;
            Ok(body)
        } else {
            let expr = self.parse_expression()?;
            Ok(vec![expr])
        }
    }

    fn parse_type_decl(&mut self) -> Result<Expr, String> {
        self.advance();
        let name = self.expect_ident()?;
        self.expect(TokenKind::Assign)?;
        self.skip_newlines();
        let mut variants = Vec::new();
        let first_name = self.expect_ident()?;
        let first_fields = self.parse_variant_fields()?;
        variants.push(TypeVariant { name: first_name, fields: first_fields });
        self.skip_newlines();
        while self.peek_kind() == Some(&TokenKind::Pipe) {
            self.advance();
            self.skip_newlines();
            let vname = self.expect_ident()?;
            let vfields = self.parse_variant_fields()?;
            variants.push(TypeVariant { name: vname, fields: vfields });
            self.skip_newlines();
        }
        Ok(Expr::TypeDecl { name, variants })
    }

    fn parse_variant_fields(&mut self) -> Result<Vec<String>, String> {
        if self.peek_kind() == Some(&TokenKind::LParen) {
            self.advance();
            let fields = self.parse_param_list()?;
            self.expect(TokenKind::RParen)?;
            Ok(fields)
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_load(&mut self) -> Result<Expr, String> {
        self.advance();
        let name = self.expect_ident()?;
        Ok(Expr::Load(name))
    }

    fn parse_do(&mut self) -> Result<Expr, String> {
        self.advance();
        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Do(body))
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.skip_newlines();
        let mut stmts = Vec::new();
        while self.peek_kind() != Some(&TokenKind::RBrace) && !self.is_at_end() {
            let stmt = self.parse_statement()?;
            stmts.push(stmt);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, String> {
        let left = self.parse_or()?;
        match self.peek_kind() {
            Some(TokenKind::Assign) => {
                self.advance();
                self.skip_newlines();
                let value = self.parse_assignment()?;
                Ok(Expr::Assign {
                    target: Box::new(left),
                    value: Box::new(value),
                })
            }
            Some(op @ TokenKind::PlusAssign)
            | Some(op @ TokenKind::MinusAssign)
            | Some(op @ TokenKind::StarAssign)
            | Some(op @ TokenKind::SlashAssign) => {
                let op = op.clone();
                self.advance();
                self.skip_newlines();
                let value = self.parse_assignment()?;
                Ok(Expr::CompoundAssign {
                    target: Box::new(left),
                    op,
                    value: Box::new(value),
                })
            }
            _ => Ok(left),
        }
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.peek_kind() == Some(&TokenKind::Or) {
            let op = self.advance().kind;
            self.skip_newlines();
            let right = self.parse_and()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.peek_kind() == Some(&TokenKind::And) {
            let op = self.advance().kind;
            self.skip_newlines();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while matches!(self.peek_kind(), Some(TokenKind::Eq) | Some(TokenKind::Neq)) {
            let op = self.advance().kind;
            self.skip_newlines();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_addition()?;
        while matches!(self.peek_kind(), Some(TokenKind::Lt) | Some(TokenKind::Gt) | Some(TokenKind::Lte) | Some(TokenKind::Gte)) {
            let op = self.advance().kind;
            self.skip_newlines();
            let right = self.parse_addition()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        while matches!(self.peek_kind(), Some(TokenKind::Plus) | Some(TokenKind::Minus)) {
            let op = self.advance().kind;
            self.skip_newlines();
            let right = self.parse_multiplication()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while matches!(self.peek_kind(), Some(TokenKind::Star) | Some(TokenKind::Slash) | Some(TokenKind::Percent)) {
            let op = self.advance().kind;
            self.skip_newlines();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek_kind(), Some(TokenKind::Not) | Some(TokenKind::Minus)) {
            let op = self.advance().kind;
            self.skip_newlines();
            let right = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op,
                right: Box::new(right),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_kind() {
                Some(TokenKind::LParen) => {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(TokenKind::RParen)?;
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                    };
                }
                Some(TokenKind::LBracket) => {
                    self.advance();
                    self.skip_newlines();
                    let index = self.parse_expression()?;
                    self.skip_newlines();
                    self.expect(TokenKind::RBracket)?;
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Some(TokenKind::Dot) => {
                    self.advance();
                    let field = self.expect_ident()?;
                    expr = Expr::Dot {
                        object: Box::new(expr),
                        field,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        self.skip_newlines();
        if self.peek_kind() == Some(&TokenKind::RParen) {
            return Ok(args);
        }
        args.push(self.parse_expression()?);
        loop {
            self.skip_newlines();
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
                if self.peek_kind() == Some(&TokenKind::RParen) { break; }
            } else {
                break;
            }
            args.push(self.parse_expression()?);
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Int(n) => { let n = *n; self.advance(); Ok(Expr::Int(n)) }
            TokenKind::Float(n) => { let n = *n; self.advance(); Ok(Expr::Float(n)) }
            TokenKind::String(s) => { let s = s.clone(); self.advance(); Ok(Expr::String(s)) }
            TokenKind::Bool(b) => { let b = *b; self.advance(); Ok(Expr::Bool(b)) }
            TokenKind::Null => { self.advance(); Ok(Expr::Null) }
            TokenKind::Ident(_) => { let name = self.expect_ident()?; Ok(Expr::Ident(name)) }

            TokenKind::LParen => {
                self.advance();
                self.skip_newlines();
                let expr = self.parse_expression()?;
                self.skip_newlines();
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }

            TokenKind::LBracket => {
                self.advance();
                self.skip_newlines();
                let result = self.parse_list_content()?;
                self.skip_newlines();
                self.expect(TokenKind::RBracket)?;
                Ok(result)
            }

            TokenKind::If => self.parse_if(),

            TokenKind::Fn => self.parse_fn_expr(),

            TokenKind::Do => self.parse_do_expr(),

            _ => Err(format!("Unexpected token: {} at line {}", tok.kind, tok.span.line)),
        }
    }

    fn parse_list_content(&mut self) -> Result<Expr, String> {
        // Empty list
        if self.peek_kind() == Some(&TokenKind::RBracket) {
            return Ok(Expr::List(Vec::new()));
        }

        let first = self.parse_expression()?;

        // Comprehension: [expr for name in iterable] or [expr for name in iterable if cond]
        if self.peek_kind() == Some(&TokenKind::For) {
            self.advance();
            let name = self.expect_ident()?;
            self.expect(TokenKind::In)?;
            let iterable = self.parse_expression()?;
            let filter = if self.peek_kind() == Some(&TokenKind::If) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            return Ok(Expr::Comprehension {
                expr: Box::new(first),
                name,
                iterable: Box::new(iterable),
                filter,
            });
        }

        // Range: [expr..end]
        if self.peek_kind() == Some(&TokenKind::DotDot) {
            self.advance();
            let end = self.parse_expression()?;
            return Ok(Expr::Range {
                start: Box::new(first),
                end: Box::new(end),
            });
        }

        // Not a range or comprehension — single element unless comma follows
        if self.peek_kind() != Some(&TokenKind::Comma) {
            return Ok(Expr::List(vec![first]));
        }

        // Consume first comma
        self.advance();
        self.skip_newlines();

        // Check for [first, ..., end] (DotDotDot right after comma)
        if self.peek_kind() == Some(&TokenKind::DotDotDot) {
            self.advance();
            self.skip_newlines();
            if self.peek_kind() == Some(&TokenKind::Comma) {
                self.advance();
                self.skip_newlines();
            }
            let end = self.parse_expression()?;
            // step = 1
            return Ok(Expr::RangeStep {
                start: Box::new(first),
                step: Box::new(Expr::Int(1)),
                end: Box::new(end),
            });
        }

        let second = self.parse_expression()?;

        // [first, second, ..., end] -- arithmetic sequence
        if self.peek_kind() == Some(&TokenKind::Comma) {
            self.advance();
            self.skip_newlines();
            if self.peek_kind() == Some(&TokenKind::DotDotDot) {
                self.advance();
                self.skip_newlines();
                if self.peek_kind() == Some(&TokenKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                }
                let end = self.parse_expression()?;
                return Ok(Expr::RangeStep {
                    start: Box::new(first),
                    step: Box::new(second),
                    end: Box::new(end),
                });
            }
            // More than 3 elements — plain list
            let mut elements = vec![first, second];
            elements.push(self.parse_expression()?);
            loop {
                self.skip_newlines();
                if self.peek_kind() == Some(&TokenKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                    if self.peek_kind() == Some(&TokenKind::RBracket) { break; }
                } else {
                    break;
                }
                elements.push(self.parse_expression()?);
            }
            return Ok(Expr::List(elements));
        }

        // Two-element list
        Ok(Expr::List(vec![first, second]))
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;
        let then_block = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;
        self.skip_newlines();

        let mut elif_clauses = Vec::new();

        // support both `elif` (deprecated but still valid) and `else if`
        loop {
            if self.peek_kind() == Some(&TokenKind::Elif) {
                self.advance();
                let cond = self.parse_expression()?;
                self.skip_newlines();
                self.expect(TokenKind::LBrace)?;
                let block = self.parse_block()?;
                self.expect(TokenKind::RBrace)?;
                self.skip_newlines();
                elif_clauses.push((cond, block));
            } else if self.peek_kind() == Some(&TokenKind::Else) &&
                       self.tokens.get(self.pos + 1).map_or(false, |t| t.kind == TokenKind::If) {
                self.advance(); // consume 'else'
                self.advance(); // consume 'if'
                let cond = self.parse_expression()?;
                self.skip_newlines();
                self.expect(TokenKind::LBrace)?;
                let block = self.parse_block()?;
                self.expect(TokenKind::RBrace)?;
                self.skip_newlines();
                elif_clauses.push((cond, block));
            } else {
                break;
            }
        }

        let else_block = if self.peek_kind() == Some(&TokenKind::Else) {
            self.advance();
            self.skip_newlines();
            self.expect(TokenKind::LBrace)?;
            let block = self.parse_block()?;
            self.expect(TokenKind::RBrace)?;
            Some(block)
        } else {
            None
        };

        Ok(Expr::If {
            condition: Box::new(condition),
            then_block,
            elif_clauses,
            else_block,
        })
    }

    fn parse_fn_expr(&mut self) -> Result<Expr, String> {
        self.advance();
        self.expect(TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RParen)?;
        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Fn {
            name: None,
            params,
            body,
        })
    }

    fn parse_do_expr(&mut self) -> Result<Expr, String> {
        self.advance();
        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Do(body))
    }
}