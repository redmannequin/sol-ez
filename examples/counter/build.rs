fn main() -> Result<(), Box<dyn std::error::Error>> {
    sol_gen::generate2("contracts/counter.toml", "src/counter_contract.rs")?;
    Ok(())
}
