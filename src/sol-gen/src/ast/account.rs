use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::token::Span;

use super::Identifer;

pub struct Account<'a> {
    pub span: Span,
    pub name: Identifer<'a>,
    pub fields: Vec<AccountField<'a>>,
}

impl<'a> Account<'a> {
    pub fn name(&self) -> String {
        self.name.value.to_case(Case::Pascal)
    }

    pub fn generate(self) -> TokenStream {
        let name = quote::format_ident!("{}", self.name());
        let fields = self.fields.into_iter().map(AccountField::generate);
        quote! {
            #[derive(Debug, BorshSerialize, BorshDeserialize, AccountData)]
            pub struct #name {
                #(#fields)*
            }
        }
    }
}

pub struct AccountField<'a> {
    pub span: Span,
    pub number: u8,
    pub name: Identifer<'a>,
    pub r#type: Type,
}

impl<'a> AccountField<'a> {
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

pub enum Type {
    Bool,
    U8,
    U64,
    F32,
    F64,
    Str(Str),
    Arr(Arr),
}

impl Type {
    pub fn generate(self) -> syn::Ident {
        match self {
            Type::Bool => syn::Ident::new("bool", proc_macro2::Span::call_site()),
            Type::U8 => syn::Ident::new("u8", proc_macro2::Span::call_site()),
            Type::U64 => syn::Ident::new("u64", proc_macro2::Span::call_site()),
            Type::F32 => syn::Ident::new("f32", proc_macro2::Span::call_site()),
            Type::F64 => syn::Ident::new("f64", proc_macro2::Span::call_site()),
            _ => todo!(),
        }
    }
}

pub struct Str {
    pub max: u8,
}

pub struct Arr {
    pub max: u8,
    pub r#type: Box<Type>,
}
