use anchor_lang::prelude::*;

#[account]
pub struct User {
    // The public key of the wallet that owns this User
    // We use this PDA as a seed to derive other downstream
    // accounts.
    //
    // User -> Profile -> Post -> [Connection, Reaction]
    pub authority: Pubkey,
    pub random_hash: [u8; 32],
    pub bump: u8,
}

impl User {
    pub const LEN: usize = 8 + std::mem::size_of::<Self>();
}
