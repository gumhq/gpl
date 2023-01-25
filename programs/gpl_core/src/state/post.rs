use anchor_lang::prelude::*;

pub const MAX_LEN_URI: usize = 128;

#[account]
#[derive(Default)]
pub struct Post {
    pub profile: Pubkey,
    pub metadata_uri: String,
    pub post_metadata: String,
    pub bump: u8,
    pub random_hash: [u8; 32],

    //Comments are just replies
    pub reply_to: Option<Pubkey>,
}

impl Post {
    pub const LEN: usize = 8 + 32 + std::mem::size_of::<Self>() + MAX_LEN_URI;
}
