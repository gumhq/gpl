use anchor_lang::prelude::*;
use gpl_core::state::ReactionType;

#[event]
pub struct CompressedPostNew {
    pub asset_id: Pubkey,
    pub post_id: Pubkey,
    pub post_bump: u8,
    pub index: u32,
    pub profile: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub timestamp: i64,
}

#[event]
pub struct CompressedPostUpdated {
    pub asset_id: Pubkey,
    pub post_id: Pubkey,
    pub post_bump: u8,
    pub index: u32,
    pub profile: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub timestamp: i64,
}

#[event]
pub struct CompressedPostDeleted {
    pub asset_id: Pubkey,
    pub post_id: Pubkey,
    pub post_bump: u8,
    pub index: u32,
    pub profile: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub timestamp: i64,
}

#[event]
pub struct CompressedConnectionNew {
    pub asset_id: Pubkey,
    pub connection_id: Pubkey,
    pub connection_bump: u8,
    pub index: u32,
    pub from_profile: Pubkey,
    pub to_profile: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct CompressedConnectionDeleted {
    pub asset_id: Pubkey,
    pub connection_id: Pubkey,
    pub connection_bump: u8,
    pub index: u32,
    pub from_profile: Pubkey,
    pub to_profile: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct CompressedReactionNew {
    pub asset_id: Pubkey,
    pub reaction_id: Pubkey,
    pub reaction_bump: u8,
    pub index: u32,
    pub from_profile: Pubkey,
    pub to_post: Pubkey,
    pub reaction_type: String,
    pub timestamp: i64,
}

#[event]
pub struct CompressedReactionDeleted {
    pub asset_id: Pubkey,
    pub reaction_id: Pubkey,
    pub reaction_bump: u8,
    pub index: u32,
    pub from_profile: Pubkey,
    pub to_post: Pubkey,
    pub reaction_type: String,
    pub timestamp: i64,
}

#[event]
pub struct CompressedCommentNew {
    pub asset_id: Pubkey,
    pub post_id: Pubkey,
    pub post_bump: u8,
    pub index: u32,
    pub reply_to: Pubkey,
    pub profile: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub timestamp: i64,
}
