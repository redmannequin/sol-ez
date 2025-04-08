use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use sol_gen_common::{
    discriminator::{DiscriminatorGen, HashDiscriminatorGen},
    error::SolGenError,
    my_idl::{Account, InstructionAccount, IxAccountState, MyIdl, Type},
};

use crate::config::Config;

pub fn gen_from_config(config: Config) -> Result<TokenStream, SolGenError> {
    let idl = config.into();

    let dispatcher = gen_dispatcher::<HashDiscriminatorGen>(&idl)?;
    let contract = gen_contract(&idl);
    let types = gen_types::<HashDiscriminatorGen>(&idl);

    Ok(quote! {
        use core::marker::PhantomData;

        use borsh::{BorshDeserialize, BorshSerialize};
        use sol_ez::{account::*, account_info::*, AccountData, DataSize};
        use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

        #types
        #contract
        #dispatcher
    })
}

pub fn gen_dispatcher<D>(idl: &MyIdl) -> Result<TokenStream, SolGenError>
where
    D: DiscriminatorGen,
{
    let dispatcher_name = str_to_struct_name(&idl.name, Some("Dispatcher"));
    let contract_trait_name = str_to_struct_name(&idl.name, Some("Contract"));

    let ix_match_branch = idl
        .instructions
        .iter()
        .map(|ix| {
            let discriminator = {
                let bytes = D::from_instruction(&idl.name, ix, idl.instruction_discriminator_size);
                quote! { [#( #bytes ),*] }
            };

            let call = {
                let ix_name = str_to_field_name(&ix.name);
                let accounts_name = str_to_struct_name(&ix.name, Some("Accounts"));

                match ix.args.len() {
                    0 => quote! {{
                        let accounts = #accounts_name::load(accounts)?;
                        T::#ix_name(program_id, accounts)
                    }},
                    1 => {
                        let arg = ix
                            .args
                            .iter()
                            .next()
                            .map(|arg| str_to_field_name(&arg.name));
                        quote! {{
                            let accounts = #accounts_name::load(accounts)?;
                            let #arg = ix_data.deserialize_data()?;
                            T::#ix_name(program_id, accounts, #arg)
                        }}
                    }
                    _ => {
                        let args = ix.args.iter().map(|arg| str_to_field_name(&arg.name));
                        let args2 = args.clone();
                        quote! {{
                            let accounts = #accounts_name::load(accounts)?;
                            let ( #( #args ),* ) = ix_data.deserialize_data()?;
                            T::#ix_name(program_id, accounts, #( #args2 ),* )
                        }}
                    }
                }
            };

            Ok(quote! {
                #discriminator => #call
            })
        })
        .collect::<Result<Vec<_>, SolGenError>>()?;

    Ok(quote! {
        pub struct #dispatcher_name<T> {
            inner: PhantomData<T>
        }

        impl<T> sol_ez::Contract for #dispatcher_name<T>
        where
            T: #contract_trait_name
        {
            fn dispatch<'info>(
                program_id: &Pubkey,
                accounts: &'info [pinocchio::account_info::AccountInfo],
                payload: &[u8]
            ) -> Result<(), ProgramError> {
                let ix_data = sol_ez::InstructionData::new(payload)?;
                match ix_data.ix {
                    #( #ix_match_branch, )*
                    _ => Err(ProgramError::InvalidInstructionData)
                }
            }

        }
    })
}

pub fn gen_contract(idl: &MyIdl) -> TokenStream {
    let contract_trait_name = str_to_struct_name(&idl.name, Some("Contract"));
    let contract_ix_fns = idl.instructions.iter().map(|ix| {
        let fn_name = str_to_field_name(&ix.name);
        let accounts_name = str_to_struct_name(&ix.name, Some("Accounts"));

        if ix.args.len() == 0 {
            return quote! {
                fn #fn_name(program_id: &Pubkey, accounts: #accounts_name) -> Result<(), ProgramError>
            };
        }

        let args = ix.args.iter().map(|arg| {
            let arg_name = quote::format_ident!("{}", arg.name);
            let arg_ty = gen_type(&arg.ty);
            quote! { #arg_name: #arg_ty }
        });

        quote! {
            fn #fn_name(program_id: &Pubkey, accounts: #accounts_name #(, #args )*) -> Result<(), ProgramError>
        }
    });

    quote! {
        pub trait #contract_trait_name {
            #( #contract_ix_fns; )*
        }
    }
}

pub fn gen_types<D>(idl: &MyIdl) -> TokenStream
where
    D: DiscriminatorGen,
    D::Seed: quote::ToTokens,
{
    let account_types = idl
        .accounts
        .iter()
        .map(|acc| gen_account::<D>(&idl.name, acc));
    let accounts_types = idl
        .instructions
        .iter()
        .map(|ix| gen_accounts(&ix.name, &ix.accounts));

    quote! {
        #( #account_types )*
        #( #accounts_types )*
    }
}

fn gen_accounts(ix_name: &str, accounts: &[InstructionAccount]) -> TokenStream {
    let accounts_name = str_to_struct_name(ix_name, Some("Accounts"));
    let accounts_fields = accounts.iter().map(|acc| {
        let field_name = str_to_field_name(&acc.name);
        let account_type = acc
            .payload
            .as_ref()
            .map(|p| str_to_struct_name(&p, None))
            .unwrap_or_else(|| str_to_struct_name("empty", None));

        let state = match acc.state {
            IxAccountState::Create => str_to_struct_name("Init", None),
            IxAccountState::Immutable => str_to_struct_name("Immutable", None),
            IxAccountState::Mutable => str_to_struct_name("Mutable", None),
        };

        let signed = if acc.is_signed {
            str_to_struct_name("Signed", None)
        } else {
            str_to_struct_name("Unsigned", None)
        };

        if acc.state.is_create() {
            quote! { pub #field_name: Account<'info, PhantomData<#account_type>, #state, #signed> }
        } else {
            quote! { pub #field_name: Account<'info, #account_type, #state, #signed> }
        }
    });

    let load = accounts.iter().map(|acc| {
        let field_name = str_to_field_name(&acc.name);
        let id = acc.id as usize;

        if acc.state.is_create() {
            quote! {
                #field_name: Account::new_init(
                    AccountInfo::new_init(
                        accounts.get(#id).ok_or(ProgramError::NotEnoughAccountKeys)?
                    )?
                )
            }
        } else {
            let mut code = quote! {
                #field_name: AccountBuilder::new(
                    accounts.get(#id).ok_or(ProgramError::NotEnoughAccountKeys)?
                )
            };
            if acc.payload.is_some() {
                code.extend(quote! { .set_payload() });
            }
            if acc.state.is_mutable() {
                code.extend(quote! { .mutable()? });
            }
            if acc.is_signed {
                code.extend(quote! { .signed()? });
            }
            code.extend(quote! { .build()? });
            code
        }
    });

    quote! {
        pub struct #accounts_name<'info> {
            #( #accounts_fields, )*
        }

        impl<'info> #accounts_name<'info> {
            pub fn load(accounts: &'info[pinocchio::account_info::AccountInfo]) -> Result<Self, ProgramError> {
                Ok(Self {
                    #( #load, )*
                })
            }
        }
    }
}

fn gen_account<D>(program_name: &str, account: &Account) -> TokenStream
where
    D: DiscriminatorGen,
    D::Seed: quote::ToTokens,
{
    let account_name = str_to_struct_name(&account.name, None);
    let account_fields = account.fields.iter().map(|field| {
        let field_name = str_to_field_name(&field.name);
        let ty = gen_type(&field.ty);
        quote! { pub #field_name: #ty }
    });
    let discriminator_seed = D::account_seed(program_name, account);
    let discriminator_size = account.discriminator.as_ref().map(|d| d.size).unwrap_or(4) as usize;

    // TODO don't add discriminator if not defined

    quote! {
        #[derive(BorshSerialize, BorshDeserialize, AccountData)]
        #[account_data(hash(seed = #discriminator_seed,  size = #discriminator_size))]
        pub struct #account_name {
            #( #account_fields, )*
        }
    }
}

fn str_to_field_name(s: &str) -> syn::Ident {
    quote::format_ident!("{}", s.to_case(Case::Snake))
}

fn str_to_struct_name(s: &str, append: Option<&str>) -> syn::Ident {
    let s = match append {
        Some(a) => {
            format!("{}_{}", s.to_case(Case::Snake), a.to_case(Case::Snake))
        }
        None => s.to_string(),
    };
    quote::format_ident!("{}", s.to_case(Case::Pascal))
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
            let ty = quote::format_ident!("{}", ty);
            quote! { #ty }
        }
    }
}
