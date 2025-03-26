use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::token::Span;

use super::Identifer;

pub struct Contract<'a> {
    pub span: Span,
    pub name: Identifer<'a>,
    pub instructions: Vec<Instruction<'a>>,
}

impl<'a> Contract<'a> {
    pub fn name(&self) -> String {
        self.name.value.to_case(Case::Pascal)
    }

    pub fn generate(self) -> TokenStream {
        let name = self.name();
        let temp_name = format!("{}Contract", &name);
        let contract_mod_name = quote::format_ident!("{}", temp_name.to_case(Case::Snake));
        let contract_trait_name = quote::format_ident!("{}", temp_name);
        let contract_dispatcher_name = quote::format_ident!("{}Dispatcher", &name);
        let (instruction_match, instruction_fn): (Vec<_>, Vec<_>) = self
            .instructions
            .into_iter()
            .map(Instruction::generate)
            .unzip();
        quote! {
            pub mod #contract_mod_name {
                use core::marker::PhantomData;

                use pinocchio::{account_info::AccountInfo, ProgramResult, program_error::ProgramError, pubkey::Pubkey};

                pub struct #contract_dispatcher_name<T> {
                    inner: PhantomData<T>
                }


                impl<T> sol_ez::Contract for #contract_dispatcher_name<T>
                where
                    T: #contract_trait_name
                {
                    fn dispatch<'info>(program_id: &Pubkey, accounts: &'info [AccountInfo], payload: &[u8]) -> ProgramResult {
                        let (instruction, _rest) = payload.split_first().ok_or(ProgramError::InvalidInstructionData)?;

                        match instruction {
                            #(#instruction_match)*
                            _ => Err(ProgramError::InvalidInstructionData)
                        }
                    }
                }

                pub trait #contract_trait_name {
                    #(#instruction_fn)*
                }
            }
        }
    }
}

pub struct Instruction<'a> {
    pub span: Span,
    pub number: u8,
    pub name: Identifer<'a>,
    pub accounts: Identifer<'a>,
}

impl<'a> Instruction<'a> {
    pub fn generate(self) -> (TokenStream, TokenStream) {
        let name = syn::Ident::new(self.name.value, proc_macro2::Span::call_site());
        let accounts = syn::Ident::new(self.accounts.value, proc_macro2::Span::call_site());

        let id = self.number;

        let instruction_match = quote! {
            #id => {
                let ctx = sol_ez::Context {
                    program_id,
                    accounts: super::#accounts::load(accounts)?
                };
                T::#name(ctx)
            }
        };

        let instruction_fn = quote! {
            fn #name(accounts: sol_ez::Context<super::#accounts>) -> ProgramResult;
        };

        (instruction_match, instruction_fn)
    }
}
