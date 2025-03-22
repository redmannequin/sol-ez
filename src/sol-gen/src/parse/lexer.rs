use super::token::{Span, Token, TokenType};

pub struct Lexer<'a> {
    src: CharHandler<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src: CharHandler::new(src),
        }
    }

    pub fn src_span(&self, span: Span) -> &'a str {
        self.src.span(span.start, span.end)
    }

    pub fn next(&mut self) -> Option<Token> {
        self.skip_whitespace();
        self.src.curr().map(|ch| match ch {
            '{' => self.create_single_char_token(TokenType::LBrace),
            '}' => self.create_single_char_token(TokenType::RBrace),
            '(' => self.create_single_char_token(TokenType::LParam),
            ')' => self.create_single_char_token(TokenType::RParam),
            ',' => self.create_single_char_token(TokenType::Comma),
            '=' => self.create_single_char_token(TokenType::Assign),
            ':' => self.create_single_char_token(TokenType::Colon),
            ';' => self.create_single_char_token(TokenType::SimiColon),
            ch => {
                if ch.is_ascii_alphabetic() {
                    self.parse_identifer()
                } else if ch.is_ascii_digit() {
                    self.parse_integer()
                } else {
                    self.create_single_char_token(TokenType::InvalidChar(ch))
                }
            }
        })
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.src.curr() {
            if !ch.is_whitespace() {
                break;
            }
            self.src.next();
        }
    }

    fn parse_identifer(&mut self) -> Token {
        let s = self.src.pos;
        while let Some(c) = self.src.next() {
            if !(c.is_ascii_alphanumeric() || c == '_') {
                break;
            }
        }
        let e = self.src.pos;

        let ty = match self.src.span(s, e) {
            "account" => TokenType::Account,
            "accounts" => TokenType::Accounts,
            "contract" => TokenType::Contract,
            "instruction" => TokenType::Instruction,
            "option" => TokenType::Option,
            "u64" => TokenType::U64,
            "u8" => TokenType::U8,
            "bool" => TokenType::Bool,
            "mut" => TokenType::Mut,
            "init" => TokenType::Init,
            _ => TokenType::Identifer,
        };

        self.craete_span_token(s, e, ty)
    }

    fn parse_integer(&mut self) -> Token {
        let s = self.src.pos;

        while let Some(ch) = self.src.next() {
            if !ch.is_ascii_digit() {
                break;
            }
        }

        self.craete_span_token(s, self.src.pos, TokenType::Intager)
    }

    fn create_single_char_token(&mut self, ty: TokenType) -> Token {
        let s = self.src.pos;
        let e = s + 1;
        self.src.next();
        self.craete_span_token(s, e, ty)
    }

    fn craete_span_token(&self, start: usize, end: usize, ty: TokenType) -> Token {
        Token {
            span: Span { start, end },
            r#type: ty,
        }
    }
}

struct CharHandler<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> CharHandler<'a> {
    pub fn new(src: &'a str) -> Self {
        CharHandler { src, pos: 0 }
    }

    pub fn next(&mut self) -> Option<char> {
        self.pos += 1;
        self.src.chars().nth(self.pos)
    }

    pub fn curr(&self) -> Option<char> {
        self.src.chars().nth(self.pos)
    }

    pub fn span(&self, s: usize, e: usize) -> &'a str {
        &self.src[s..e]
    }
}
