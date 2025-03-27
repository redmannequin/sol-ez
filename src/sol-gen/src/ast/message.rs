use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

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
    pub fn generate(self) -> TokenStream {
        let name = quote::format_ident!("{}", self.name());
        let fields = self.fields.into_iter().map(MessageField::generate);
        quote! {
            #[derive(Debug,  BorshDeserialize)]
            pub struct #name {
                #(#fields)*
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MessageField<'a> {
    pub span: Span,
    pub name: Identifer<'a>,
    pub r#type: Type,
}

impl<'a> MessageField<'a> {
    pub fn name(&self) -> String {
        self.name.value.to_string()
    }

    pub fn generate(self) -> TokenStream {
        let name = syn::Ident::new(self.name.value, proc_macro2::Span::call_site());
        let ty = self.r#type.generate();
        quote! {
            pub #name: #ty,
        }
    }
}
