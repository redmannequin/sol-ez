use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};

use crate::parse::token::Span;

use super::Identifer;

#[derive(Debug, PartialEq, Eq)]
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
            .map(|ix| Instruction::generate(ix, &name))
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
                        let instruction_data = sol_ez::InstructionData::new(payload)?;

                        match instruction_data.ix {
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

#[derive(Debug, PartialEq, Eq)]
pub struct Instruction<'a> {
    pub span: Span,
    pub number: u8,
    pub name: Identifer<'a>,
    pub accounts: Identifer<'a>,
    pub payload: Option<Identifer<'a>>,
}

impl<'a> Instruction<'a> {
    pub fn generate(self, contract_name: &str) -> (TokenStream, TokenStream) {
        let name = syn::Ident::new(self.name.value, proc_macro2::Span::call_site());
        let accounts = syn::Ident::new(self.accounts.value, proc_macro2::Span::call_site());

        let discriminator = {
            let hash = Sha256::digest(format!("ix|{}|{}", contract_name, self.name.value));
            let bytes = &hash[..4];
            quote! { [#( #bytes ),*] }
        };

        if let Some(payload) = self.payload {
            let payload = syn::Ident::new(payload.value, proc_macro2::Span::call_site());
            let instruction_match = quote! {
                #discriminator => {
                    let ctx = sol_ez::Context {
                        program_id,
                        accounts: super::#accounts::load(accounts)?
                    };
                    T::#name(ctx, instruction_data.deserialize_data()?)
                }
            };

            let instruction_fn = quote! {
                fn #name(accounts: sol_ez::Context<super::#accounts>, payload: super::#payload) -> ProgramResult;
            };
            return (instruction_match, instruction_fn);
        }

        let instruction_match = quote! {
            #discriminator => {
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
