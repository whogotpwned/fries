use crate::token::{Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while self.pos < self.source.len() {
            if let Some(token) = self.skip_whitespace_and_comments()? {
                tokens.push(token);
            }
        }
        tokens.push(Token::new(TokenKind::Eof, self.line, self.col));
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.source.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<Option<Token>, String> {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.advance();
                    let line = self.line - 1;
                    let col = 1;
                    return Ok(Some(Token::new(TokenKind::Newline, line, col)));
                }
                Some('#') => {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }

        match self.peek() {
            Some(ch) => self.read_token(ch).map(Some),
            None => Ok(None),
        }
    }

    fn read_token(&mut self, ch: char) -> Result<Token, String> {
        let line = self.line;
        let col = self.col;

        match ch {
            '0'..='9' => self.read_number(line, col),
            '"' | '\'' => self.read_string(line, col),
            'a'..='z' | 'A'..='Z' | '_' => self.read_ident(line, col),
            _ => self.read_symbol(line, col),
        }
    }

    fn read_number(&mut self, line: usize, col: usize) -> Result<Token, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('.') && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
            let text: String = self.source[start..self.pos].iter().collect();
            let n: f64 = text.parse().map_err(|_| format!("Invalid float: {text}"))?;
            Ok(Token::new(TokenKind::Float(n), line, col))
        } else {
            let text: String = self.source[start..self.pos].iter().collect();
            let n: i64 = text.parse().map_err(|_| format!("Invalid integer: {text}"))?;
            Ok(Token::new(TokenKind::Int(n), line, col))
        }
    }

    fn read_string(&mut self, line: usize, col: usize) -> Result<Token, String> {
        let quote = self.advance().unwrap();
        let mut s = String::new();
        loop {
            let ch = self.advance().ok_or_else(|| {
                format!("Unterminated string starting at line {line}, col {col}")
            })?;
            if ch == quote {
                break;
            }
            if ch == '\\' {
                let escaped = self.advance().ok_or("Unexpected end of string escape")?;
                match escaped {
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    'r' => s.push('\r'),
                    '\\' => s.push('\\'),
                    '"' => s.push('"'),
                    '\'' => s.push('\''),
                    '0' => s.push('\0'),
                    _ => return Err(format!("Unknown escape: \\{escaped}")),
                }
            } else {
                s.push(ch);
            }
        }
        Ok(Token::new(TokenKind::String(s), line, col))
    }

    fn read_ident(&mut self, line: usize, col: usize) -> Result<Token, String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let text: String = self.source[start..self.pos].iter().collect();
        let kind = match text.as_str() {
            "let" => TokenKind::Let,
            "var" => TokenKind::Var,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elif" => TokenKind::Elif,
            "return" => TokenKind::Return,
            "match" => TokenKind::Match,
            "type" => TokenKind::Type,
            "do" => TokenKind::Do,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            "null" => TokenKind::Null,
            "_" => TokenKind::Underscore,
            _ => TokenKind::Ident(text),
        };
        Ok(Token::new(kind, line, col))
    }

    fn read_symbol(&mut self, line: usize, col: usize) -> Result<Token, String> {
        let ch = self.peek().unwrap();

        let token = match ch {
            '+' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::PlusAssign
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::MinusAssign
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StarAssign
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::SlashAssign
                } else {
                    TokenKind::Slash
                }
            }
            '%' => {
                self.advance();
                TokenKind::Percent
            }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Eq
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Assign
                }
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Neq
                } else {
                    TokenKind::Not
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Lte
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Gte
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                self.advance();
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::And
                } else {
                    return Err(format!("Unexpected character: '&' at line {line}, col {col}"));
                }
            }
            '|' => {
                self.advance();
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::Or
                } else {
                    TokenKind::Pipe
                }
            }
            '(' => { self.advance(); TokenKind::LParen }
            ')' => { self.advance(); TokenKind::RParen }
            '{' => { self.advance(); TokenKind::LBrace }
            '}' => { self.advance(); TokenKind::RBrace }
            '[' => { self.advance(); TokenKind::LBracket }
            ']' => { self.advance(); TokenKind::RBracket }
            ',' => { self.advance(); TokenKind::Comma }
            '.' => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('.') {
                        self.advance();
                        TokenKind::DotDotDot
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }
            _ => return Err(format!("Unexpected character: '{ch}' at line {line}, col {col}")),
        };

        Ok(Token::new(token, line, col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(source: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(source);
        lexer.tokenize().unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_integers() {
        let tokens = tokenize("42");
        assert_eq!(tokens, vec![TokenKind::Int(42), TokenKind::Eof]);
    }

    #[test]
    fn test_floats() {
        let tokens = tokenize("3.14");
        assert_eq!(tokens, vec![TokenKind::Float(3.14), TokenKind::Eof]);
    }

    #[test]
    fn test_string() {
        let tokens = tokenize("\"hello\"");
        assert_eq!(tokens, vec![TokenKind::String("hello".into()), TokenKind::Eof]);
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("1 + 2 * 3");
        assert_eq!(tokens, vec![
            TokenKind::Int(1), TokenKind::Plus,
            TokenKind::Int(2), TokenKind::Star,
            TokenKind::Int(3), TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize("let var fn if else elif return match type do while for in null");
        assert_eq!(tokens, vec![
            TokenKind::Let, TokenKind::Var, TokenKind::Fn,
            TokenKind::If, TokenKind::Else, TokenKind::Elif,
            TokenKind::Return, TokenKind::Match, TokenKind::Type,
            TokenKind::Do, TokenKind::While, TokenKind::For,
            TokenKind::In, TokenKind::Null, TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_comparison() {
        let tokens = tokenize("x == y != z");
        assert_eq!(tokens, vec![
            TokenKind::Ident("x".into()), TokenKind::Eq,
            TokenKind::Ident("y".into()), TokenKind::Neq,
            TokenKind::Ident("z".into()), TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_assignment_ops() {
        let tokens = tokenize("x = 1\nx += 2");
        assert_eq!(tokens, vec![
            TokenKind::Ident("x".into()), TokenKind::Assign,
            TokenKind::Int(1), TokenKind::Newline,
            TokenKind::Ident("x".into()), TokenKind::PlusAssign,
            TokenKind::Int(2), TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_arrow_and_fat_arrow() {
        let tokens = tokenize("-> =>");
        assert_eq!(tokens, vec![
            TokenKind::Arrow, TokenKind::FatArrow, TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_comment() {
        let tokens = tokenize("x # comment\n42");
        assert_eq!(tokens, vec![
            TokenKind::Ident("x".into()), TokenKind::Newline,
            TokenKind::Int(42), TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_bool_and_null() {
        let tokens = tokenize("true false null");
        assert_eq!(tokens, vec![
            TokenKind::Bool(true), TokenKind::Bool(false),
            TokenKind::Null, TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_brackets() {
        let tokens = tokenize("()");
        assert_eq!(tokens, vec![TokenKind::LParen, TokenKind::RParen, TokenKind::Eof]);
    }

    #[test]
    fn test_escape_in_string() {
        let tokens = tokenize("\"hello\\nworld\"");
        assert_eq!(tokens, vec![TokenKind::String("hello\nworld".into()), TokenKind::Eof]);
    }

    #[test]
    fn test_dot_dot() {
        let tokens = tokenize("1..10");
        assert_eq!(tokens, vec![
            TokenKind::Int(1), TokenKind::DotDot,
            TokenKind::Int(10), TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_dot_dot_dot() {
        let tokens = tokenize("..., 5");
        assert_eq!(tokens, vec![
            TokenKind::DotDotDot,
            TokenKind::Comma,
            TokenKind::Int(5), TokenKind::Eof,
        ]);
    }
}