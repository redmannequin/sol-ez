use std::mem;

use anyhow::Context;
use lexer::Lexer;
use token::{Span, Token, TokenType};

use crate::{
    ast::{
        Account, AccountField, Accounts, AccountsField, Contract, Definitions, Identifer,
        Instruction, Message, MessageField, Type,
    },
    error::SolGenError,
};

pub mod lexer;
pub mod token;

pub struct Parser<'src> {
    lexer: TokenHandler<'src>,
}

impl<'src> Parser<'src> {
    pub fn new(lexer: Lexer<'src>) -> Self {
        Parser {
            lexer: TokenHandler::new(lexer),
        }
    }

    pub fn parse(mut self) -> Result<Definitions<'src>, SolGenError> {
        let mut account_defs = Vec::new();
        let mut accounts_defs = Vec::new();
        let mut contract = None;
        let mut message_defs = Vec::new();

        while let Some(token) = self.lexer.curr() {
            match token.r#type {
                TokenType::Account => account_defs.push(self.parse_account()?),
                TokenType::Accounts => accounts_defs.push(self.parse_accounts()?),
                TokenType::Contract => {
                    if contract.is_some() {
                        return Err(SolGenError::Other(anyhow::anyhow!(
                            "Only a single contract can be defined"
                        )));
                    }
                    contract = Some(self.parse_contract()?);
                }
                TokenType::Message => message_defs.push(self.parse_message()?),
                TokenType::InvalidChar(ch) => return Err(SolGenError::InvalidChar(ch)),
                token_ty => {
                    return Err(SolGenError::ExpectedToken(
                        String::from("account, accounts, or contract"),
                        format!("{:?}", token_ty),
                    ));
                }
            }
        }

        Ok(Definitions {
            message: message_defs,
            account: account_defs,
            accounts: accounts_defs,
            contract: contract.context("A contract must be defined")?,
        })
    }

    fn parse_message(&mut self) -> Result<Message<'src>, SolGenError> {
        let payload = self.lexer.consume_if(TokenType::Message)?;
        let name = self.parse_identifer()?;
        let _l_brace = self.lexer.consume_if(TokenType::LBrace)?;

        let mut fields = Vec::new();
        while let Some(token) = self.lexer.curr() {
            if token.r#type == TokenType::RBrace {
                break;
            }
            fields.push(self.parse_payload_field()?);
        }
        let r_brace = self.lexer.consume_if(TokenType::RBrace)?;

        Ok(Message {
            span: Span {
                start: payload.span.start,
                end: r_brace.span.end,
            },
            name,
            fields,
        })
    }

    fn parse_payload_field(&mut self) -> Result<MessageField<'src>, SolGenError> {
        let name = self.parse_identifer()?;
        let _colon = self.lexer.consume_if(TokenType::Colon)?;
        let r#type = self.parse_type()?;
        let _assign = self.lexer.consume_if(TokenType::Assign)?;
        let number = self.parse_int()?;
        let simi_colon = self.lexer.consume_if(TokenType::SimiColon)?;

        Ok(MessageField {
            span: Span {
                start: name.span.start,
                end: simi_colon.span.end,
            },
            number,
            name: name,
            r#type,
        })
    }

    fn parse_account(&mut self) -> Result<Account<'src>, SolGenError> {
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

    fn parse_account_field(&mut self) -> Result<AccountField<'src>, SolGenError> {
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

    fn parse_accounts(&mut self) -> Result<Accounts<'src>, SolGenError> {
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

    fn parse_accounts_field(&mut self) -> Result<AccountsField<'src>, SolGenError> {
        let mutable = self.lexer.consume_if(TokenType::Mutable).is_ok();
        let init = self.lexer.consume_if(TokenType::Init).is_ok();

        assert!(!(mutable & init));

        let name = self.parse_identifer()?;
        let _colon = self.lexer.consume_if(TokenType::Colon)?;
        let account = match self.lexer.curr() {
            Some(Token {
                r#type: TokenType::Signer,
                ..
            }) => {
                let signer = self.lexer.bump().unwrap();
                Identifer {
                    span: signer.span,
                    value: self.lexer.src_span(signer.span),
                }
            }
            _ => self.parse_identifer()?,
        };
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

    fn parse_contract(&mut self) -> Result<Contract<'src>, SolGenError> {
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

    fn parse_instruction(&mut self) -> Result<Instruction<'src>, SolGenError> {
        let instruction = self.lexer.consume_if(TokenType::Instruction)?;
        let name = self.parse_identifer()?;
        let _l_param = self.lexer.consume_if(TokenType::LParam)?;
        let accounts = self.parse_identifer()?;

        let payload = self
            .lexer
            .consume_if(TokenType::Comma)
            .ok()
            .map(|_| self.parse_identifer())
            .transpose()?;

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
            payload: payload,
        })
    }

    fn parse_identifer(&mut self) -> Result<Identifer<'src>, SolGenError> {
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

#[cfg(test)]
mod test_parser {
    use crate::{
        ast::{
            Account, AccountField, Accounts, AccountsField, Contract, Identifer, Instruction,
            Message, MessageField, Type,
        },
        parse::token::Span,
    };

    use super::{Parser, lexer::Lexer};

    #[test]
    fn test_message_parse() {
        let src = " message InitalValue { value: u8 = 1; } ";
        let msg = Parser::new(Lexer::new(src))
            .parse_message()
            .expect("failed to parse message src");

        assert_eq!(
            Message {
                span: Span { start: 1, end: 39 },
                name: Identifer {
                    span: Span { start: 9, end: 20 },
                    value: "InitalValue"
                },
                fields: vec![MessageField {
                    span: Span { start: 23, end: 37 },
                    number: 1,
                    name: Identifer {
                        span: Span { start: 23, end: 28 },
                        value: "value"
                    },
                    r#type: Type::U8
                }]
            },
            msg
        )
    }

    #[test]
    fn test_account_parse() {
        let src = " account InitalValue { value: u8 = 1; } ";
        let account = Parser::new(Lexer::new(src))
            .parse_account()
            .expect("failed to parse account src");

        assert_eq!(
            Account {
                span: Span { start: 1, end: 39 },
                name: Identifer {
                    span: Span { start: 9, end: 20 },
                    value: "InitalValue"
                },
                fields: vec![AccountField {
                    span: Span { start: 23, end: 37 },
                    number: 1,
                    name: Identifer {
                        span: Span { start: 23, end: 28 },
                        value: "value"
                    },
                    r#type: Type::U8
                }]
            },
            account
        )
    }

    #[test]
    fn test_accounts_parse() {
        let src = " accounts Initalize { user: User = 1; } ";
        let accoutns = Parser::new(Lexer::new(src))
            .parse_accounts()
            .expect("failed to parse accounts src");

        assert_eq!(
            Accounts {
                span: Span { start: 1, end: 39 },
                name: Identifer {
                    span: Span { start: 10, end: 19 },
                    value: "Initalize"
                },
                fields: vec![AccountsField {
                    span: Span { start: 22, end: 37 },
                    number: 1,
                    init: false,
                    mutable: false,
                    name: Identifer {
                        span: Span { start: 22, end: 26 },
                        value: "user"
                    },
                    account: Identifer {
                        span: Span { start: 28, end: 32 },
                        value: "User"
                    }
                }]
            },
            accoutns
        )
    }

    #[test]
    fn test_contract_parse() {
        let src = r#"
            contract test {
                instruction initalize(Init) = 1;
            }
        "#;
        let contract = Parser::new(Lexer::new(src))
            .parse_contract()
            .expect("failed to parse contract src");

        assert_eq!(
            Contract {
                span: Span { start: 13, end: 91 },
                name: Identifer {
                    span: Span { start: 22, end: 26 },
                    value: "test"
                },
                instructions: vec![Instruction {
                    span: Span { start: 45, end: 77 },
                    number: 1,
                    name: Identifer {
                        span: Span { start: 57, end: 66 },
                        value: "initalize"
                    },
                    accounts: Identifer {
                        span: Span { start: 67, end: 71 },
                        value: "Init"
                    },
                    payload: None
                }]
            },
            contract
        )
    }
}
