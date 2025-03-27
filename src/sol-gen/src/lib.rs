use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::Context;
use error::SolGenError;
use parse::{Parser, lexer::Lexer};
use proc_macro2::TokenStream;
use quote::quote;

pub mod ast;
pub mod error;
pub mod parse;

pub fn generate(src_path: &str, out_path: &str) -> Result<(), SolGenError> {
    let mut fp = File::open(src_path)?;
    let mut src = String::new();
    fp.read_to_string(&mut src)?;

    let lexer = Lexer::new(src.as_str());
    let parser = Parser::new(lexer);
    let defs = parser.parse()?;

    let mut code = TokenStream::new();

    code.extend(quote! {
        use core::marker::PhantomData;

        use borsh::{BorshDeserialize, BorshSerialize};
        use sol_ez::{account::*, account_info::*, AccountData, DataSize};

        mod pinocchio {
            pub use pinocchio::{program_error::ProgramError, account_info::AccountInfo};
        }
    });
    code.extend(defs.message.into_iter().map(ast::Message::generate));
    code.extend(defs.account.into_iter().map(ast::Account::generate));
    code.extend(defs.accounts.into_iter().map(ast::Accounts::generate));
    code.extend(defs.contract.generate());

    let code_file = syn::parse2(code).context("failed to parse token stream")?;
    let code_src = prettyplease::unparse(&code_file);

    let mut out_fp = File::create(out_path)?;
    out_fp.write_all(code_src.as_bytes())?;

    Ok(())
}
