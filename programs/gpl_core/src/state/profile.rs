use crate::state::MAX_LEN_URI;
use anchor_lang::prelude::*;

#[account]
pub struct Profile {
    // The user PDA that owns this profile
    pub authority: Pubkey,

    // This collapses the ProfileMetadata PDA into the Profile struct.
    pub metadata_uri: String,

    // External reference to SNS, ANS or GPL Nameservice
    pub screen_name: Pubkey,

    pub random_hash: [u8; 32],
}

impl Profile {
    pub const LEN: usize = 8 + MAX_LEN_URI + std::mem::size_of::<Self>();
}
