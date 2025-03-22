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
    let (account_defs, accounts_defs, contract_defs) = parser.parse()?;

    let mut code = TokenStream::new();

    code.extend(quote! {
        use std::marker::PhantomData;

        use borsh::{BorshDeserialize, BorshSerialize};
        use solana_program::{account_info::AccountInfo, program_error::ProgramError};
        use sol_ez::{account::*, AccountRent};
    });
    code.extend(account_defs.into_iter().map(ast::Account::generate));
    code.extend(accounts_defs.into_iter().map(ast::Accounts::generate));
    code.extend(contract_defs.into_iter().map(ast::Contract::generate));

    let code_file = syn::parse2(code).context("failed to parse token stream")?;
    let code_src = prettyplease::unparse(&code_file);

    let mut out_fp = File::create(out_path)?;
    out_fp.write_all(code_src.as_bytes())?;

    Ok(())
}
