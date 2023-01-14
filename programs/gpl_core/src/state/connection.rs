use anchor_lang::prelude::*;

#[account]
pub struct Connection {
    // The profile that owns this connection
    pub from_profile: Pubkey,
    // The profile that this connection is to
    pub to_profile: Pubkey,
    pub bump: u8,
}

impl Connection {
    pub const LEN: usize = 8 + std::mem::size_of::<Self>();
}
