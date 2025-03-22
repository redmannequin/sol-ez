use std::mem;

use lexer::Lexer;
use token::{Span, Token, TokenType};

use crate::{
    ast::{Account, AccountField, Accounts, AccountsField, Contract, Identifer, Instruction, Type},
    error::SolGenError,
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

    pub fn parse(
        mut self,
    ) -> Result<(Vec<Account<'a>>, Vec<Accounts<'a>>, Vec<Contract<'a>>), SolGenError> {
        let mut account_defs = Vec::new();
        let mut accounts_defs = Vec::new();
        let mut contract_defs = Vec::new();

        while let Some(token) = self.lexer.curr() {
            match token.r#type {
                TokenType::Account => account_defs.push(self.parse_account()?),
                TokenType::Accounts => accounts_defs.push(self.parse_accounts()?),
                TokenType::Contract => contract_defs.push(self.parse_contract()?),
                TokenType::InvalidChar(ch) => return Err(SolGenError::InvalidChar(ch)),
                token_ty => {
                    return Err(SolGenError::ExpectedToken(
                        String::from("account, accounts, or contract"),
                        format!("{:?}", token_ty),
                    ));
                }
            }
        }

        Ok((account_defs, accounts_defs, contract_defs))
    }

    fn parse_account(&mut self) -> Result<Account<'a>, SolGenError> {
        let account = self.lexer.consume_if(TokenType::Account)?;
        let name = self.parse_identifer()?;
        let _l_brace = self.lexer.consume_if(TokenType::LBrace)?;

        let mut fields = Vec::new();
        while let Some(token) = self.lexer.curr() {
            if token.r#type == TokenType::RBrace {
                break;
            }
            fields.push(self.parse_account_field()?);
        }

        let r_brace = self.lexer.consume_if(TokenType::RBrace)?;

        Ok(Account {
            span: Span {
                start: account.span.start,
                end: r_brace.span.end,
            },
            name,
            fields,
        })
    }

    fn parse_account_field(&mut self) -> Result<AccountField<'a>, SolGenError> {
        let name = self.parse_identifer()?;
        let _colon = self.lexer.consume_if(TokenType::Colon)?;
        let r#type = self.parse_type()?;
        let _assign = self.lexer.consume_if(TokenType::Assign)?;
        let number = self.parse_int()?;
        let simi_colon = self.lexer.consume_if(TokenType::SimiColon)?;

        Ok(AccountField {
            span: Span {
                start: name.span.start,
                end: simi_colon.span.end,
            },
            number,
            name: name,
            r#type,
        })
    }

    fn parse_type(&mut self) -> Result<Type, SolGenError> {
        Ok(
            match self
                .lexer
                .bump()
                .ok_or_else(|| {
                    SolGenError::ExpectedToken(format!("u8, u64, or bool"), format!("None"))
                })?
                .r#type
            {
                TokenType::U8 => Type::U8,
                TokenType::U64 => Type::U64,
                TokenType::Bool => Type::Bool,
                token_ty => {
                    return Err(SolGenError::ExpectedToken(
                        format!("u8, u64, or bool"),
                        format!("{:?}", token_ty),
                    ));
                }
            },
        )
    }

    fn parse_accounts(&mut self) -> Result<Accounts<'a>, SolGenError> {
        let accounts = self.lexer.consume_if(TokenType::Accounts)?;
        let name = self.parse_identifer()?;
        let _l_brace = self.lexer.consume_if(TokenType::LBrace)?;

        let mut fields = Vec::new();
        while let Some(token) = self.lexer.curr() {
            if token.r#type == TokenType::RBrace {
                break;
            }
            fields.push(self.parse_accounts_field()?);
        }

        let r_brace = self.lexer.consume_if(TokenType::RBrace)?;

        Ok(Accounts {
            span: Span {
                start: accounts.span.start,
                end: r_brace.span.end,
            },
            name,
            fields,
        })
    }

    fn parse_accounts_field(&mut self) -> Result<AccountsField<'a>, SolGenError> {
        let mutable = self.lexer.consume_if(TokenType::Mut).is_ok();
        let init = self.lexer.consume_if(TokenType::Init).is_ok();

        assert!(!(mutable & init));

        let name = self.parse_identifer()?;
        let _colon = self.lexer.consume_if(TokenType::Colon)?;
        let account = self.parse_identifer()?;
        let _assign = self.lexer.consume_if(TokenType::Assign)?;
        let number = self.parse_int()?;
        let simi_colon = self.lexer.consume_if(TokenType::SimiColon)?;

        Ok(AccountsField {
            span: Span {
                start: name.span.start,
                end: simi_colon.span.end,
            },
            number,
            name,
            account,
            init,
            mutable,
        })
    }

    fn parse_contract(&mut self) -> Result<Contract<'a>, SolGenError> {
        let contract = self.lexer.consume_if(TokenType::Contract)?;
        let name = self.parse_identifer()?;
        let _l_brace = self.lexer.consume_if(TokenType::LBrace)?;

        let mut instructions = Vec::new();
        while let Some(token) = self.lexer.curr() {
            if token.r#type == TokenType::RBrace {
                break;
            }
            instructions.push(self.parse_instruction()?);
        }

        let r_brace = self.lexer.consume_if(TokenType::RBrace)?;

        Ok(Contract {
            span: Span {
                start: contract.span.start,
                end: r_brace.span.end,
            },
            name,
            instructions,
        })
    }

    fn parse_instruction(&mut self) -> Result<Instruction<'a>, SolGenError> {
        let instruction = self.lexer.consume_if(TokenType::Instruction)?;
        let name = self.parse_identifer()?;
        let _l_param = self.lexer.consume_if(TokenType::LParam)?;
        let accounts = self.parse_identifer()?;
        let _r_param = self.lexer.consume_if(TokenType::RParam)?;
        let _assign = self.lexer.consume_if(TokenType::Assign)?;
        let number = self.parse_int()?;
        let simi_colon = self.lexer.consume_if(TokenType::SimiColon)?;

        Ok(Instruction {
            span: Span {
                start: instruction.span.start,
                end: simi_colon.span.end,
            },
            number,
            name,
            accounts,
        })
    }

    fn parse_identifer(&mut self) -> Result<Identifer<'a>, SolGenError> {
        let identifer = self.lexer.consume_if(TokenType::Identifer)?;
        Ok(Identifer {
            span: identifer.span,
            value: self.lexer.src_span(identifer.span),
        })
    }

    fn parse_int(&mut self) -> Result<u8, SolGenError> {
        let int = self.lexer.consume_if(TokenType::Intager)?;
        Ok(self.lexer.src_span(int.span).parse()?)
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

    pub fn consume_if(&mut self, r#type: TokenType) -> Result<Token, SolGenError> {
        let token = self.curr.filter(|token| token.r#type == r#type);
        if token.is_some() {
            self.bump();
        }
        token.ok_or(SolGenError::ExpectedToken(
            format!("{:?}", r#type),
            format!("{:?}", self.curr),
        ))
    }

    pub fn src_span(&self, span: Span) -> &'a str {
        self.lexer.src_span(span)
    }
}
