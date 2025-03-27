use convert_case::{Case, Casing};

use crate::parse::token::Span;

use super::{Identifer, Type};

#[derive(Debug, PartialEq, Eq)]
pub struct Message<'a> {
    pub span: Span,
    pub name: Identifer<'a>,
    pub fields: Vec<MessageField<'a>>,
}

impl<'a> Message<'a> {
    pub fn name(&self) -> String {
        self.name.value.to_case(Case::Pascal)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MessageField<'a> {
    pub span: Span,
    pub number: u8,
    pub name: Identifer<'a>,
    pub r#type: Type,
}

impl<'a> MessageField<'a> {
    pub fn name(&self) -> String {
        self.name.value.to_string()
    }
}
