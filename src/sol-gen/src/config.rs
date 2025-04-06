use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{error::SolGenError, idl};

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Config {
    pub program: Program,
    pub ix: BTreeMap<String, Ix>,
    pub ix_config: IxConfig,
    #[serde(default)]
    pub accounts: BTreeMap<String, Account>,
    #[serde(default)]
    pub message: BTreeMap<String, Message>,
}

impl Config {
    pub fn validate(&self) -> Result<(), SolGenError> {

        for (ix_name, ix) in self.ix.iter() {
            let mut idxs = vec![0; ix.accounts.len()];
            for (acc_name, acc) in ix.accounts.iter() {
                if idxs[acc.idx] == 1 {
                    Err(anyhow::anyhow!("duplicate idx in {} accounts({})", ix_name, acc_name))?;
                } else if acc.create && (acc.mutable | acc.signed) {
                    Err(anyhow::anyhow!("idx({}) account({}) cant be create and mutable or signed", ix_name, acc_name))?;
                } else if let Some(ty) = &acc.r#type {

                    let _account_def = self.accounts.get(ty).ok_or(
                        anyhow::anyhow!("idx({}) account({}) type {} not defined", ix_name, acc_name, ty)
                    )?;

                    // TODO check seed params from accounts and account seed

                }
                idxs[acc.idx] = 1;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Program {
    pub name: String,
    pub version: (u8, u8, u8),
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Ix {
    #[serde(default)]
    pub args: BTreeMap<String, Type>,
    pub accounts: BTreeMap<String, IxAccount>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct IxConfig {
    pub discriminator_size: u8,
    pub discriminator_type: DiscriminatorType,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscriminatorType {
    Hash,
    Index,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct IxAccount {
    idx: usize,
    #[serde(default)]
    pub create: bool,
    #[serde(default)]
    pub mutable: bool,
    #[serde(default)]
    pub signed: bool,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub seed: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Account {
    pub bump: bool,
    pub seed: String,
    pub payload: Message,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Message {
    Struct(BTreeMap<String, Type>),
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Bytes,
    String,
    PublicKey,
    Option(Box<Type>),
    FixedArray(Box<Type>, usize),
    DynamicArray(Box<Type>),
    Defined(String),
}

impl<'src> From<&'src Config> for idl::Idl<'src> {
    fn from(value: &'src Config) -> Self {
        idl::Idl {
            version: idl::Version {
                major: value.program.version.0,
                minor: value.program.version.1,
                patch: value.program.version.2,
            },
            name: value.program.name.as_str(),
            constants: vec![],
            accounts: value
                .accounts
                .iter()
                .map(|(name, account)| idl::Account {
                    name,
                    discriminator: None,
                    r#type: match &account.payload {
                        Message::Struct(fields) => idl::AccountDef {
                            kind: "struct",
                            fields: fields
                                .iter()
                                .map(|(name, ty)| idl::StructTypeDefField {
                                    name,
                                    r#type: ty.into(),
                                })
                                .collect(),
                        },
                    },
                })
                .collect(),
            instructions: value
                .ix
                .iter()
                .map(|(name, ix)| idl::Instruction {
                    name,
                    discriminator: None,
                    accounts: ix
                        .accounts
                        .iter()
                        .map(|(name, acc)| idl::InstructionAccount {
                            name,
                            is_mutable: acc.mutable,
                            is_signer: acc.signed,
                        })
                        .collect(),
                    args: ix
                        .args
                        .iter()
                        .map(|(name, ty)| idl::InstructionArg {
                            name,
                            r#type: ty.into(),
                        })
                        .collect(),
                })
                .collect(),
            types: value
                .message
                .iter()
                .map(|(name, ty)| idl::TypeDef {
                    name,
                    r#type: match ty {
                        Message::Struct(fields) => idl::TypeDefKind::Struct(idl::StructTypeDef {
                            fields: fields
                                .iter()
                                .map(|(name, ty)| idl::StructTypeDefField {
                                    name,
                                    r#type: ty.into(),
                                })
                                .collect(),
                        }),
                    },
                })
                .collect(),
            events: vec![],
            errors: vec![],
        }
    }
}

impl<'src> From<&'src Type> for idl::Type<'src> {
    fn from(value: &'src Type) -> Self {
        match value {
            Type::Bool => idl::Type::Bool,
            Type::U8 => idl::Type::U8,
            Type::U16 => idl::Type::U16,
            Type::U32 => idl::Type::U32,
            Type::U64 => idl::Type::U64,
            Type::U128 => idl::Type::U128,
            Type::I8 => idl::Type::I8,
            Type::I16 => idl::Type::I16,
            Type::I32 => idl::Type::I32,
            Type::I64 => idl::Type::I64,
            Type::I128 => idl::Type::I128,
            Type::Bytes => idl::Type::Bytes,
            Type::String => idl::Type::String,
            Type::PublicKey => idl::Type::PublicKey,
            Type::Option(ty) => idl::Type::Option(Box::new(ty.as_ref().into())),
            Type::FixedArray(ty, n) => idl::Type::FixedArray(Box::new(ty.as_ref().into()), *n),
            Type::DynamicArray(ty) => idl::Type::DynamicArray(Box::new(ty.as_ref().into())),
            Type::Defined(ty) => idl::Type::Defined(ty),
        }
    }
}

pub fn gen_program_from_config(config: Config) -> TokenStream {
    let program_trait_name = syn::Ident::new(
        &format!("{}Contract", config.program.name.to_case(Case::Pascal)),
        Span::call_site(),
    );

    let program_trait_fns = config.ix.iter().map(|(name, ix)| {
        let accounts = quote::format_ident!("{}Accounts", name.to_case(Case::Pascal));
        let name = syn::Ident::new(&name.to_case(Case::Snake), Span::call_site());

        let args = ix.args.iter().map(|(name, ty)| {
            let name = quote::format_ident!("{}", name.to_case(Case::Snake));
            let ty = gen_type(ty);
            quote! { #name: #ty }
        });

        quote! {
            fn #name(owner: &Pubkey, accounts: #accounts #(,#args)*) -> Result<(), ProgramError>
        }
    });

    let account = config.accounts.iter().map(|(name, acc)| {
        let struct_def = gen_struct(
            name,
            match &acc.payload {
                Message::Struct(fields) => fields.iter().map(|(name, ty)| (name.as_str(), ty)),
            },
        );

        quote! {
            #[derive(Debug, BorshSerialize, BorshDeserialize, AccountData)]
            #struct_def


        }
    });

    let accounts = config.ix.iter().map(|(name, ix)| {
        let name = format!("{}Accounts", name.to_case(Case::Pascal));
        let name = syn::Ident::new(&name, Span::call_site());
        let fields = ix.accounts.iter().map(|(name, acc)| {
            let name = syn::Ident::new(&name.to_case(Case::Snake), Span::call_site());
            let payload = acc
                .r#type
                .as_ref()
                .map(|ty| syn::Ident::new(&ty.to_case(Case::Pascal), Span::call_site()))
                .unwrap_or(quote::format_ident!("Empty"));

            let (payload, mutable) = match acc.create {
                true => (
                    quote! { PhantomData<#payload> },
                    quote::format_ident!("Init"),
                ),
                false => (
                    quote! { #payload },
                    match acc.mutable {
                        true => quote::format_ident!("Mutable"),
                        false => quote::format_ident!("ReadOnly"),
                    },
                ),
            };

            let signed = match acc.signed {
                true => quote::format_ident!("Signed"),
                false => quote::format_ident!("Unsigned"),
            };

            quote! { pub #name: Account<'info, #payload, #mutable, #signed> }
        });

        let load = ix.accounts.iter().map(|(name, acc)| {

            let name = quote::format_ident!("{}", name.to_case(Case::Snake));
            let idx = acc.idx; 

            match acc.create {
                true => quote! {
                    #name: Account::new_init(
                        AccountInfo::new_init(
                            accounts.get(#idx).ok_or(ProgramError::NotEnoughAccountKeys)?
                        )?
                    ) 
                },
                false => {
                    let mut code = quote! {
                        #name: AccountBuilder::new(
                            accounts.get(#idx).ok_or(ProgramError::NotEnoughAccountKeys)?
                        )
                    };
                    if  acc.r#type.is_some() {
                        code.extend(quote! { .set_payload() });
                    }
                    if  acc.mutable {
                        code.extend(quote! { .mutable()? });
                    }
                    if  acc.signed{
                        code.extend(quote! { .signed()? });
                    }
                    code.extend(quote! { .build()? });
                    
                    code
                } 
            }
            
        });

        quote! {
            pub struct #name<'info> {
                #( #fields, )*
            }

            impl<'info> #name<'info> {
                pub fn load(accounts: &'info[pinocchio::account_info::AccountInfo]) -> Result<Self, ProgramError> {
                    Ok(Self {
                        #( #load, )*
                    })
                }
            }
        }
    });

    let dispatcher = {
        let contract_dispatcher_name =
            quote::format_ident!("{}Dispatcher", config.program.name.to_case(Case::Pascal));

        let ix_match = config.ix.iter().map(|(name, ix)| {

            // TODO: check discriminator type
            let discriminator = {
                let hash = Sha256::digest(format!("ix|{}|{}", config.program.name.to_case(Case::Snake), name.to_case(Case::Snake)));
                let (bytes,_) = hash.split_at(config.ix_config.discriminator_size as usize);
                quote! { [#( #bytes ),*] }
            };

            let call = {
                let accounts = quote::format_ident!("{}Accounts", name.to_case(Case::Pascal));
                let name = syn::Ident::new(&name.to_case(Case::Snake), Span::call_site());

                if ix.args.len() == 0 {
                    quote! { T::#name(program_id, #accounts::load(accounts)?) }  
                } else if ix.args.len() == 1 {
                    let arg = ix.args.iter().next().map(|(name, _)| {
                        let name = quote::format_ident!("{}", name.to_case(Case::Snake));
                        quote! { #name }
                    });
                    let arg2 = arg.clone();
                    quote! {{
                        let #arg = ix_data.deserialize_data()?;
                        T::#name(program_id, #accounts::load(accounts)?, #arg2)
                    }}

                } else {
                    let args = ix.args.iter().map(|(name, _)| {
                        let name = quote::format_ident!("{}", name.to_case(Case::Snake));
                        quote! { #name }
                    });
                    let args2 = args.clone();
                    quote! {{
                        let (#( #args ),*) =  ix_data.deserialize_data()?;
                        T::#name(program_id, #accounts::load(accounts)? #(,#args2)*)
                    }}
                }
            };

            quote! { #discriminator => #call }
        }); 

        quote! {
            pub struct #contract_dispatcher_name<T> {
                inner: PhantomData<T>
            }

            impl<T> sol_ez::Contract for #contract_dispatcher_name<T>
            where
                T: #program_trait_name
            {
                fn dispatch<'info>(program_id: &Pubkey, accounts: &'info [pinocchio::account_info::AccountInfo], payload: &[u8]) -> Result<(), ProgramError> {
                    let ix_data = sol_ez::InstructionData::new(payload)?;
                    match ix_data.ix {
                        #( #ix_match, )*
                        _ => Err(ProgramError::InvalidInstructionData)
                    }
                }
            }
        }
    };

    quote! {
        use core::marker::PhantomData;

        use borsh::{BorshDeserialize, BorshSerialize};
        use sol_ez::{account::*, account_info::*, AccountData, DataSize};
        use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

        #dispatcher

        pub trait #program_trait_name {
            #( #program_trait_fns; )*
        }

        #( #account )*
        #( #accounts )*
    }
}

fn gen_struct<'src>(
    name: &str,
    fields: impl Iterator<Item = (&'src str, &'src Type)>,
) -> TokenStream {
    let name = syn::Ident::new(&name.to_case(Case::Pascal), Span::call_site());
    let fields = fields.map(|(name, ty)| {
        let name = syn::Ident::new(&name.to_case(Case::Snake), Span::call_site());
        let ty = gen_type(ty);
        quote! { pub #name: #ty }
    });
    quote! {
        pub struct #name {
            #( #fields, )*
        }
    }
}

fn gen_type(ty: &Type) -> TokenStream {
    match ty {
        Type::Bool => quote! {bool },
        Type::U8 => quote! { u8 },
        Type::U16 => quote! { u16 },
        Type::U32 => quote! { u32 },
        Type::U64 => quote! { u64 },
        Type::U128 => quote! { u128 },
        Type::I8 => quote! { i8 },
        Type::I16 => quote! { i16 },
        Type::I32 => quote! { i32 },
        Type::I64 => quote! { i64 },
        Type::I128 => quote! { i128 },
        Type::Bytes => quote! { Vec<u8> },
        Type::String => quote! { String },
        Type::PublicKey => quote! { [u8; 32] },
        Type::Option(ty) => {
            let ty = gen_type(ty);
            quote! { Option<#ty> }
        }
        Type::FixedArray(ty, n) => {
            let ty = gen_type(ty);
            quote! { [#ty; #n] }
        }
        Type::DynamicArray(ty) => {
            let ty = gen_type(ty);
            quote! { Vec<#ty> }
        }
        Type::Defined(ty) => {
            let ty = syn::Ident::new(&ty.to_case(Case::Pascal), Span::call_site());
            quote! { #ty }
        }
    }
}
