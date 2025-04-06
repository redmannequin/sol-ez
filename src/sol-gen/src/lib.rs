use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::Context;
use config::gen_program_from_config;
use error::SolGenError;
use serde::Deserialize;

pub mod config;
pub mod error;
pub mod idl;

pub fn generate2(src_path: &str, out_path: &str) -> Result<(), SolGenError> {
    let mut fp = File::open(src_path)?;
    let mut src = String::new();
    fp.read_to_string(&mut src)?;

    let config = config::Config::deserialize(toml::Deserializer::new(&src))
        .context("failed to parse config")?;
    let code = gen_program_from_config(config);

    let code_file = syn::parse2(code).context("failed to parse token stream")?;
    let code_src = prettyplease::unparse(&code_file);

    let mut out_fp = File::create(out_path)?;
    out_fp.write_all(code_src.as_bytes())?;

    Ok(())
}
