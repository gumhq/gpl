use crate::errors::GumError;
use anchor_lang::prelude::*;

#[account]
pub struct Reaction {
    // The profile that owns this reaction
    pub from_profile: Pubkey,
    // The post that this reaction is to
    pub to_post: Pubkey,
    // NOTE:
    // The burden of validating the reaction is on the client
    // Since it is hard to define what a valid reaction is and will vary from app to app
    // It's better to let the client decide what is valid
    //
    // Might change this to a [u8; 32] in the future
    pub reaction_type: String,
}

impl Reaction {
    pub const REACTION_TYPE_MAX_LEN: usize = 32;
    pub const LEN: usize = 8 + 64 + Self::REACTION_TYPE_MAX_LEN + std::mem::size_of::<Reaction>();

    pub fn validate_reaction_type(reaction_type: &str) -> Result<()> {
        require!(
            reaction_type.len() <= Self::REACTION_TYPE_MAX_LEN,
            GumError::ReactionTypeTooLong
        );
        Ok(())
    }
}
