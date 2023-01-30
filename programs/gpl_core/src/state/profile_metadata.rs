use anchor_lang::prelude::*;

use crate::state::{MAX_LEN_URI};

#[account]
pub struct ProfileMetadata {
    pub profile: Pubkey,
    pub metadata_uri: String,
}

impl ProfileMetadata {
    pub const LEN: usize = 8 + std::mem::size_of::<Self>() + MAX_LEN_URI;
}
