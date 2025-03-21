use std::mem;

use lexer::Lexer;
use token::{Span, Token, TokenType};

use crate::ast::{
    Account, AccountField, Accounts, AccountsField, Contract, Identifer, Instruction, Type,
};

pub mod lexer;
pub mod token;

pub struct Parser<'a> {
    lexer: TokenHandler<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Parser {
            lexer: TokenHandler::new(lexer),
        }
    }

    pub fn parse(mut self) -> (Vec<Account<'a>>, Vec<Accounts<'a>>, Vec<Contract<'a>>) {
        let mut account_defs = Vec::new();
        let mut accounts_defs = Vec::new();
        let mut contract_defs = Vec::new();

        while let Some(token) = self.lexer.curr() {
            match token.r#type {
                TokenType::Account => account_defs.push(self.parse_account()),
                TokenType::Accounts => accounts_defs.push(self.parse_accounts()),
                TokenType::Contract => contract_defs.push(self.parse_contract()),
                _ => panic!("expetect account, accounts, or contract"),
            }
        }

        (account_defs, accounts_defs, contract_defs)
    }

    fn parse_account(&mut self) -> Account<'a> {
        let account = self.lexer.consume_if(TokenType::Account).expect("bug");
        let name = self.parse_identifer();
        let _l_brace = self.lexer.consume_if(TokenType::LBrace).expect("bug");

        let mut fields = Vec::new();
        while let Some(token) = self.lexer.curr() {
            if token.r#type == TokenType::RBrace {
                break;
            }
            fields.push(self.parse_account_field());
        }

        let r_brace = self.lexer.consume_if(TokenType::RBrace).expect("bug");

        Account {
            span: Span {
                start: account.span.start,
                end: r_brace.span.end,
            },
            name,
            fields,
        }
    }

    fn parse_account_field(&mut self) -> AccountField<'a> {
        let name = self.parse_identifer();
        let _colon = self.lexer.consume_if(TokenType::Colon).expect("bug");
        let r#type = self.parse_type();
        let _assign = self
            .lexer
            .consume_if(TokenType::Assign)
            .expect(&format!("expected '=' found {:?}", self.lexer.curr()));
        let number = self.parse_int();
        let simi_colon = self.lexer.consume_if(TokenType::SimiColon).expect("bug");

        AccountField {
            span: Span {
                start: name.span.start,
                end: simi_colon.span.end,
            },
            number,
            name: name,
            r#type,
        }
    }

    fn parse_type(&mut self) -> Type {
        match self.lexer.bump().expect("bug").r#type {
            TokenType::U8 => Type::U8,
            TokenType::U64 => Type::U64,
            TokenType::Bool => Type::Bool,
            _ => panic!("bug"),
        }
    }

    fn parse_accounts(&mut self) -> Accounts<'a> {
        let accounts = self.lexer.consume_if(TokenType::Accounts).expect("bug");
        let name = self.parse_identifer();
        let _l_brace = self.lexer.consume_if(TokenType::LBrace).expect("bug");

        let mut fields = Vec::new();
        while let Some(token) = self.lexer.curr() {
            if token.r#type == TokenType::RBrace {
                break;
            }
            fields.push(self.parse_accounts_field());
        }

        let r_brace = self.lexer.consume_if(TokenType::RBrace).expect("bug");

        Accounts {
            span: Span {
                start: accounts.span.start,
                end: r_brace.span.end,
            },
            name,
            fields,
        }
    }

    fn parse_accounts_field(&mut self) -> AccountsField<'a> {
        let mutable = self.lexer.consume_if(TokenType::Mut).is_some();
        let init = self.lexer.consume_if(TokenType::Init).is_some();

        assert!(!(mutable & init));

        let name = self.parse_identifer();
        let _colon = self.lexer.consume_if(TokenType::Colon).expect("bug");
        let account = self.parse_identifer();
        let _assign = self.lexer.consume_if(TokenType::Assign).expect("bug");
        let number = self.parse_int();
        let simi_colon = self.lexer.consume_if(TokenType::SimiColon).expect("bug");

        AccountsField {
            span: Span {
                start: name.span.start,
                end: simi_colon.span.end,
            },
            number,
            name,
            account,
            init,
            mutable,
        }
    }

    fn parse_contract(&mut self) -> Contract<'a> {
        let contract = self.lexer.consume_if(TokenType::Contract).expect("bug");
        let name = self.parse_identifer();
        let _l_brace = self.lexer.consume_if(TokenType::LBrace).expect("bug");

        let mut instructions = Vec::new();
        while let Some(token) = self.lexer.curr() {
            if token.r#type == TokenType::RBrace {
                break;
            }
            instructions.push(self.parse_instruction());
        }

        let r_brace = self.lexer.consume_if(TokenType::RBrace).expect("bug");

        Contract {
            span: Span {
                start: contract.span.start,
                end: r_brace.span.end,
            },
            name,
            instructions,
        }
    }

    fn parse_instruction(&mut self) -> Instruction<'a> {
        let instruction = self.lexer.consume_if(TokenType::Instruction).expect("bug");
        let name = self.parse_identifer();
        let _l_param = self.lexer.consume_if(TokenType::LParam).expect("bug");
        let accounts = self.parse_identifer();
        let _r_param = self.lexer.consume_if(TokenType::RParam).expect("bug");
        let _assign = self.lexer.consume_if(TokenType::Assign).expect("bug");
        let number = self.parse_int();
        let simi_colon = self.lexer.consume_if(TokenType::SimiColon).expect("bug");

        Instruction {
            span: Span {
                start: instruction.span.start,
                end: simi_colon.span.end,
            },
            number,
            name,
            accounts,
        }
    }

    fn parse_identifer(&mut self) -> Identifer<'a> {
        let identifer = self
            .lexer
            .consume_if(TokenType::Identifer)
            .expect(&format!("expected ident found {:?}", self.lexer.curr));
        Identifer {
            span: identifer.span,
            value: self.lexer.src_span(identifer.span),
        }
    }

    fn parse_int(&mut self) -> u8 {
        let int = self.lexer.consume_if(TokenType::Intager).expect("bug");
        self.lexer.src_span(int.span).parse().expect("bug")
    }
}

pub struct TokenHandler<'a> {
    pub lexer: Lexer<'a>,
    pub curr: Option<Token>,
    pub next: Option<Token>,
}

impl<'a> TokenHandler<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let curr = lexer.next();
        let next = lexer.next();

        TokenHandler { lexer, curr, next }
    }

    pub fn curr(&mut self) -> Option<Token> {
        self.curr
    }

    pub fn peek(&self) -> Option<Token> {
        self.next
    }

    pub fn bump(&mut self) -> Option<Token> {
        mem::swap(&mut self.curr, &mut self.next);
        mem::replace(&mut self.next, self.lexer.next())
    }

    pub fn consume_if(&mut self, r#type: TokenType) -> Option<Token> {
        let token = self.curr.filter(|token| token.r#type == r#type);
        if token.is_some() {
            self.bump();
        }
        token
    }

    pub fn src_span(&self, span: Span) -> &'a str {
        self.lexer.src_span(span)
    }
}
