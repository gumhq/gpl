use crate::state::MAX_LEN_URI;
use anchor_lang::prelude::*;

use strum_macros::{AsRefStr, EnumString};

#[account]
pub struct Profile {
    // The user PDA that owns this profile
    pub user: Pubkey,
    // The namespace that this profile is in
    pub namespace: Namespace,

    // This collapses the ProfileMetadata PDA into the Profile struct.
    pub metadata_uri: String,

    // External reference to SNS, ANS or GPL Nameservice
    pub screen_name: Pubkey,
}

impl Profile {
    pub const LEN: usize = 8 + MAX_LEN_URI + std::mem::size_of::<Self>();
}

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, AsRefStr, EnumString,
)]
pub enum Namespace {
    #[strum(ascii_case_insensitive)]
    Professional,
    #[strum(ascii_case_insensitive)]
    Personal,
    #[strum(ascii_case_insensitive)]
    Gaming,
    #[strum(ascii_case_insensitive)]
    Degen,
}
