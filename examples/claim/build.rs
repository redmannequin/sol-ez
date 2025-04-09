fn main() -> Result<(), Box<dyn std::error::Error>> {
    sol_gen::generate2("contract.toml", "src/claim_contract.rs")?;
    Ok(())
}
