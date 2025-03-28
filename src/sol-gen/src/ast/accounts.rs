use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::token::Span;

use super::Identifer;

#[derive(Debug, PartialEq, Eq)]
pub struct Accounts<'a> {
    pub span: Span,
    pub name: Identifer<'a>,
    pub fields: Vec<AccountsField<'a>>,
}

impl<'a> Accounts<'a> {
    pub fn name(&self) -> String {
        self.name.value.to_case(Case::Pascal)
    }

    pub fn generate(self) -> TokenStream {
        let accounts_name = quote::format_ident!("{}", self.name());
        let (struct_fields, load_fields): (Vec<_>, Vec<_>) =
            self.fields.into_iter().map(AccountsField::generate).unzip();
        quote! {
            pub struct #accounts_name<'info> {
                #(#struct_fields)*
            }

            impl<'info> #accounts_name<'info> {
                pub fn load(accounts: &'info [pinocchio::AccountInfo]) -> Result<Self, pinocchio::ProgramError> {
                    Ok(Self {
                        #(#load_fields)*
                    })
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AccountsField<'a> {
    pub span: Span,
    pub number: u8,
    pub init: bool,
    pub mutable: bool,
    pub name: Identifer<'a>,
    pub account: Identifer<'a>,
}

impl<'a> AccountsField<'a> {
    pub fn name(&self) -> String {
        self.name.value.to_case(Case::Pascal)
    }

    pub fn generate(self) -> (TokenStream, TokenStream) {
        let name = syn::Ident::new(self.name.value, proc_macro2::Span::call_site());
        let ty = syn::Ident::new(self.account.value, proc_macro2::Span::call_site());
        let idx = (self.number - 1) as usize;

        if self.account.value == "Signer" {
            assert!(!self.init, "a Singer should never be init");
            return if self.mutable {
                (
                    quote! {
                        pub #name: Account<'info, #ty, Mutable>,
                    },
                    quote! {
                        #name: Account::new_singer(
                            AccountInfo::new_mut(
                                accounts.get(#idx).ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                            )?
                        )?,
                    },
                )
            } else {
                (
                    quote! {
                        pub #name: Account<'info, #ty, Read>,
                    },
                    quote! {
                        #name: Account::new_singer(
                            AccountInfo::new_read(
                                accounts.get(#idx).ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                            )?
                        )?,
                    },
                )
            };
        }

        if self.init {
            (
                quote! {
                    pub #name: Account<'info, PhantomData<#ty>, Init>,
                },
                quote! {
                    #name: Account::new_init(
                        AccountInfo::new_init(
                            accounts.get(#idx).ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                        )?
                    ),
                },
            )
        } else if self.mutable {
            (
                quote! {
                    pub #name: Account<'info, #ty, Mutable>,
                },
                quote! {
                    #name: Account::new(
                        AccountInfo::new_mut(
                            accounts.get(#idx).ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                        )?
                    )?,
                },
            )
        } else {
            (
                quote! {
                    pub #name: Account<'info, #ty, Read>,
                },
                quote! {
                    #name: Account::new(
                        AccountInfo::new_read(
                            accounts.get(#idx).ok_or(pinocchio::ProgramError::NotEnoughAccountKeys)?,
                        )?
                    )?,
                },
            )
        }
    }
}
