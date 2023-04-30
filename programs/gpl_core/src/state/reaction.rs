use crate::errors::GumError;
use anchor_lang::prelude::*;
use core::convert;
use unicode_segmentation::UnicodeSegmentation;
// use unic_emoji_char::is_emoji;

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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ReactionType {
    Emoji { emoji: String },
    Custom { marker: [u8; 32] },
}

impl convert::AsRef<[u8]> for ReactionType {
    fn as_ref(&self) -> &[u8] {
        match self {
            ReactionType::Emoji { emoji } => emoji.as_bytes(),
            ReactionType::Custom { marker } => marker.as_ref(),
        }
    }
}

impl ReactionType {
    pub fn validate(&self) -> Result<()> {
        if let ReactionType::Emoji { emoji } = self {
            // Collect graphemes from the emoji string
            let graphemes: Vec<&str> =
                UnicodeSegmentation::graphemes(emoji.as_str(), true).collect();

            // Validate that the emoji is a single grapheme
            if graphemes.len() != 1 {
                return Err(GumError::InvalidEmoji.into());
            }

            // Validate that the emoji is a single unicode emoji
            // if !graphemes[0].chars().all(|c| is_emoji(c)) {
            //     return Err(GumError::InvalidEmoji.into());
            // }
        }
        Ok(())
    }
}
