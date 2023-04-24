use crate::state::ReactionType;
use anchor_lang::prelude::*;

// This event is emitted whenever a new profile is created.
#[event]
pub struct ProfileNew {
    pub profile: Pubkey,
    pub authority: Pubkey,
    pub random_hash: [u8; 32],
    pub timestamp: i64,
    pub screen_name: Pubkey,
    pub metadata_uri: String,
}

// This event is emitted whenever a profile is updated.
#[event]
pub struct ProfileUpdated {
    pub profile: Pubkey,
    pub timestamp: i64,
    pub screen_name: Pubkey,
    pub metadata_uri: String,
}

// This event is emitted whenever a profile is deleted.
#[event]
pub struct ProfileDeleted {
    pub profile: Pubkey,
    pub timestamp: i64,
    pub screen_name: Pubkey,
    pub metadata_uri: String,
}

// This event is emitted whenever a new post is created.
#[event]
pub struct PostNew {
    pub post: Pubkey,
    pub profile: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub timestamp: i64,
}

// This event is emitted whenever a post is updated.
#[event]
pub struct PostUpdated {
    pub post: Pubkey,
    pub profile: Pubkey,
    pub metadata_uri: String,
    pub timestamp: i64,
}

// This event is emitted whenever a post is deleted.
#[event]
pub struct PostDeleted {
    pub post: Pubkey,
    pub profile: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a new comment is created.
#[event]
pub struct PostCommentNew {
    pub post: Pubkey,
    pub profile: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub reply_to: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a new connection is created.
#[event]
pub struct ConnectionNew {
    pub connection: Pubkey,
    pub from_profile: Pubkey,
    pub to_profile: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a connection is deleted.
#[event]
pub struct ConnectionDeleted {
    pub connection: Pubkey,
    pub from_profile: Pubkey,
    pub to_profile: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a new reaction is created.
#[event]
pub struct ReactionNew {
    pub reaction: Pubkey,
    pub reaction_type: ReactionType,
    pub from_profile: Pubkey,
    pub to_post: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a reaction is deleted.
#[event]
pub struct ReactionDeleted {
    pub reaction: Pubkey,
    pub reaction_type: ReactionType,
    pub from_profile: Pubkey,
    pub to_post: Pubkey,
    pub timestamp: i64,
}
