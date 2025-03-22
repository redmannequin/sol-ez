use solana_program::pubkey::Pubkey;

pub struct Context<'a, T> {
    pub program_id: &'a Pubkey,
    pub accounts: T,
}
