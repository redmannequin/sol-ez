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
            '[' => self.create_single_char_token(TokenType::LBracket),
            ']' => self.create_single_char_token(TokenType::RBracket),
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
            // keywords
            "account" => TokenType::Account,
            "accounts" => TokenType::Accounts,
            "contract" => TokenType::Contract,
            "init" => TokenType::Init,
            "instruction" => TokenType::Instruction,
            "message" => TokenType::Message,
            "mutable" => TokenType::Mutable,
            // types
            "bool" => TokenType::Bool,
            "option" => TokenType::Option,
            "Singer" => TokenType::Signer,
            "u64" => TokenType::U64,
            "u8" => TokenType::U8,
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

#[cfg(test)]
mod lexer_tests {
    use crate::parse::token::{Span, Token, TokenType};

    use super::Lexer;

    #[test]
    fn test_lexer() {
        let src = " message test { flag: bool = 1; }";
        let mut lexer = Lexer::new(src);

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 1, end: 8 },
                r#type: TokenType::Message
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 9, end: 13 },
                r#type: TokenType::Identifer
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 14, end: 15 },
                r#type: TokenType::LBrace
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 16, end: 20 },
                r#type: TokenType::Identifer
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 20, end: 21 },
                r#type: TokenType::Colon
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 22, end: 26 },
                r#type: TokenType::Bool
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 27, end: 28 },
                r#type: TokenType::Assign
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 29, end: 30 },
                r#type: TokenType::Intager
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 30, end: 31 },
                r#type: TokenType::SimiColon
            }),
            message_token
        );

        let message_token = lexer.next();
        assert_eq!(
            Some(Token {
                span: Span { start: 32, end: 33 },
                r#type: TokenType::RBrace
            }),
            message_token
        );

        let none = lexer.next();
        assert!(none.is_none());
    }

    #[test]
    fn test_single_char_tokens() {
        let tests = [
            ('{', TokenType::LBrace),
            ('}', TokenType::RBrace),
            ('(', TokenType::LParam),
            (')', TokenType::RParam),
            ('[', TokenType::LBracket),
            (']', TokenType::RBracket),
            (',', TokenType::Comma),
            ('=', TokenType::Assign),
            (':', TokenType::Colon),
            (';', TokenType::SimiColon),
        ];

        for (ch, tok) in tests {
            let src = &format!(" {} ", ch);
            let token = Lexer::new(src).next();
            assert_eq!(
                Some(Token {
                    span: Span { start: 1, end: 2 },
                    r#type: tok
                }),
                token
            )
        }
    }

    #[test]
    fn test_word_tokens() {
        let tests = [
            // keywords
            ("account", TokenType::Account),
            ("accounts", TokenType::Accounts),
            ("contract", TokenType::Contract),
            ("init", TokenType::Init),
            ("instruction", TokenType::Instruction),
            ("message", TokenType::Message),
            ("mutable", TokenType::Mutable),
            // types
            ("bool", TokenType::Bool),
            ("option", TokenType::Option),
            ("Singer", TokenType::Signer),
            ("u64", TokenType::U64),
            ("u8", TokenType::U8),
            // identifer
            ("test", TokenType::Identifer),
        ];

        for (keyword, tok) in tests {
            let src = &format!(" {} ", keyword);
            let token = Lexer::new(src).next();
            assert_eq!(
                Some(Token {
                    span: Span {
                        start: 1,
                        end: 1 + keyword.len()
                    },
                    r#type: tok
                }),
                token
            )
        }
    }

    #[test]
    fn test_intager_token() {
        let src = " 12345 ";
        let token = Lexer::new(src).next();
        assert_eq!(
            Some(Token {
                span: Span { start: 1, end: 6 },
                r#type: TokenType::Intager
            }),
            token
        )
    }

    #[test]
    fn test_invalid_single_char() {
        let tests = ".\"!@#$%^&*+~`\\/|?><";

        for ch in tests.chars() {
            let src = &format!(" {} ", ch);
            let token = Lexer::new(src).next();
            assert_eq!(
                Some(Token {
                    span: Span { start: 1, end: 2 },
                    r#type: TokenType::InvalidChar(ch)
                }),
                token
            )
        }
    }
}
