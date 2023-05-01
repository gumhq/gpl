use crate::errors::GumError;
use anchor_lang::prelude::*;
use core::convert;
use std::str::FromStr;

#[account]
pub struct Reaction {
    // The profile that owns this reaction
    pub from_profile: Pubkey,
    // The post that this reaction is to
    pub to_post: Pubkey,
    pub reaction_type: ReactionType,
}

impl Reaction {
    pub const LEN: usize = 8 // Discriminant size
        + 32 // from_profile size
        + 32 // to_post size
        + (1 + 32); // reaction_type size
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ReactionType {
    Emoji { emoji: String },
    Custom { tag: String },
}

impl convert::AsRef<[u8]> for ReactionType {
    fn as_ref(&self) -> &[u8] {
        match self {
            ReactionType::Emoji { emoji } => emoji.as_bytes(),
            ReactionType::Custom { tag } => tag.as_bytes(),
        }
    }
}

// Display a reaction type
impl core::fmt::Display for ReactionType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ReactionType::Emoji { emoji } => write!(f, "{}", emoji),
            ReactionType::Custom { tag } => write!(f, "{}", tag),
        }
    }
}

impl ReactionType {
    pub const MAX_LEN: usize = 32;

    pub fn validate(&self) -> Result<()> {
        match self {
            ReactionType::Emoji { emoji } => {
                if emojis::get(emoji.as_str()).is_none() {
                    return Err(GumError::InvalidEmoji.into());
                }
            }
            ReactionType::Custom { tag } => {
                if tag.len() > Self::MAX_LEN {
                    return Err(GumError::CustomTagTooLong.into());
                }
            }
        }
        Ok(())
    }
}

// Convert a string to a ReactionType
impl FromStr for ReactionType {
    type Err = GumError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let emoji = emojis::get(s);
        if let Some(emoji) = emoji {
            return Ok(ReactionType::Emoji {
                emoji: emoji.to_string(),
            });
        }

        if s.len() > Self::MAX_LEN {
            return Err(GumError::CustomTagTooLong.into());
        }

        Ok(ReactionType::Custom { tag: s.to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji() {
        let reaction = ReactionType::Emoji {
            emoji: "👍".to_string(),
        };
        assert_eq!(reaction.as_ref(), "👍".as_bytes());
        assert!(reaction.validate().is_ok());
    }

    #[test]
    fn test_from_str() {
        let reaction = ReactionType::from_str("👍").unwrap();
        assert_eq!(reaction.as_ref(), "👍".as_bytes());
        assert!(reaction.validate().is_ok());
    }
}
