use anchor_lang::prelude::*;

use strum_macros::AsRefStr;

#[account]
pub struct Profile {
    // The user PDA that owns this profile
    pub user: Pubkey,
    // The namespace that this profile is in
    pub namespace: Namespace,
    // should there be metadata here?
    pub bump: u8,
}

impl Profile {
    pub const LEN: usize = 8 + std::mem::size_of::<Self>();
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, AsRefStr)]
pub enum Namespace {
    Professional,
    Personal,
    Gaming,
    Degen,
}
