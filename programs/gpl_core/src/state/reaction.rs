use anchor_lang::prelude::*;

use strum_macros::{AsRefStr, EnumString};

#[account]
pub struct Reaction {
    // The profile that owns this reaction
    pub from_profile: Pubkey,
    // The post that this reaction is to
    pub to_post: Pubkey,
    pub reaction_type: ReactionType,
}

impl Reaction {
    pub const LEN: usize = 8 + std::mem::size_of::<Self>();
}

// Probably better to use emoji codes instead of strings
#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, AsRefStr, EnumString,
)]
pub enum ReactionType {
    #[strum(ascii_case_insensitive)]
    Like,
    #[strum(ascii_case_insensitive)]
    Dislike,
    #[strum(ascii_case_insensitive)]
    Love,
    #[strum(ascii_case_insensitive)]
    Haha,
    #[strum(ascii_case_insensitive)]
    Wow,
    #[strum(ascii_case_insensitive)]
    Sad,
    #[strum(ascii_case_insensitive)]
    Angry,
}
