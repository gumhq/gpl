use anchor_lang::prelude::*;

use strum_macros::AsRefStr;

#[account]
pub struct Reaction {
    // The profile that owns this reaction
    pub from_profile: Pubkey,
    // The post that this reaction is to
    pub to_post: Pubkey,
    pub reaction_type: ReactionType,
    pub bump: u8,
}

impl Reaction {
    pub const LEN: usize = 8 + std::mem::size_of::<Self>();
}

// Probably better to use emoji codes instead of strings
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, AsRefStr)]
pub enum ReactionType {
    Like,
    Dislike,
    Love,
    Haha,
    Wow,
    Sad,
    Angry,
}
