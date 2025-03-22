fn main() -> Result<(), Box<dyn std::error::Error>> {
    sol_gen::generate("contracts/counter.ez", "src/counter_contract.rs")?;
    Ok(())
}
