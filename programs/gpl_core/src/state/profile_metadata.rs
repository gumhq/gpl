use anchor_lang::prelude::*;

#[account]
pub struct ProfileMetadata {
    pub profile: Pubkey,
    pub metadata_uri: String,
}

impl ProfileMetadata {
    pub const LEN: usize = 8 + std::mem::size_of::<Self>();
}
