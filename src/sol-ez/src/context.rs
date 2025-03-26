use pinocchio::pubkey::Pubkey;

pub struct Context<'a, A> {
    pub program_id: &'a Pubkey,
    pub accounts: A,
}
