use crate::state::MAX_LEN_URI;
use anchor_lang::prelude::*;
use std::mem::size_of;

#[account]
pub struct Badge {
    pub issuer: Pubkey,
    // Profile of the user who earned the badge
    pub holder: Pubkey,
    pub update_authority: Pubkey,
    pub schema: Pubkey,
    pub metadata_uri: String,
}

impl Badge {
    pub const SEED_PREFIX: &'static str = "badge";

    pub const LEN: usize = 8 + 64 + MAX_LEN_URI + size_of::<Self>();
}

#[account]
pub struct Issuer {
    pub authority: Pubkey,
    pub verified: bool,
}

impl Issuer {
    pub const SEED_PREFIX: &'static str = "issuer";

    pub const LEN: usize = 8 + 64 + size_of::<Self>();
}

#[account]
pub struct Schema {
    pub authority: Pubkey,
    pub metadata_uri: String,
    pub random_hash: [u8; 32],
}

impl Schema {
    pub const SEED_PREFIX: &'static str = "schema";

    pub const LEN: usize = 8 + 64 + MAX_LEN_URI + size_of::<Self>();
}
