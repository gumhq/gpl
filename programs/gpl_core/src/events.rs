use crate::state::{Namespace, ReactionType};
use anchor_lang::prelude::*;

// TODO: Explicitly add the signer to all events
// This would be useful to quickly find out who owns the current user account. Perhaps this can be
// looked up? Let's see how this works out.

// This event is emitted whenever a new user is created.
#[event]
pub struct UserNew {
    pub user: Pubkey,
    pub random_hash: [u8; 32],
    pub authority: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever the user's authority is changed.
#[event]
pub struct UserAuthorityChanged {
    pub user: Pubkey,
    pub new_authority: Pubkey,
    pub old_authority: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever the user's account is deleted.
#[event]
pub struct UserDeleted {
    pub user: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a new profile is created.
#[event]
pub struct ProfileNew {
    pub profile: Pubkey,
    pub user: Pubkey,
    pub namespace: Namespace,
    pub timestamp: i64,
}

// This event is emitted whenever a profile is deleted.
#[event]
pub struct ProfileDeleted {
    pub profile: Pubkey,
    pub user: Pubkey,
    pub namespace: Namespace,
    pub timestamp: i64,
}

// This event is emitted whenever a new post is created.
#[event]
pub struct PostNew {
    pub post: Pubkey,
    pub profile: Pubkey,
    pub user: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub timestamp: i64,
}

// This event is emitted whenever a post is updated.
#[event]
pub struct PostUpdated {
    pub post: Pubkey,
    pub profile: Pubkey,
    pub user: Pubkey,
    pub metadata_uri: String,
    pub timestamp: i64,
}

// This event is emitted whenever a post is deleted.
#[event]
pub struct PostDeleted {
    pub post: Pubkey,
    pub profile: Pubkey,
    pub user: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a new comment is created.
#[event]
pub struct PostCommentNew {
    pub post: Pubkey,
    pub profile: Pubkey,
    pub user: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub reply_to: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a new connection is created.
#[event]
pub struct ConnectionNew {
    pub connection: Pubkey,
    pub user: Pubkey,
    pub from_profile: Pubkey,
    pub to_profile: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a connection is deleted.
#[event]
pub struct ConnectionDeleted {
    pub connection: Pubkey,
    pub user: Pubkey,
    pub from_profile: Pubkey,
    pub to_profile: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a new reaction is created.
#[event]
pub struct ReactionNew {
    pub reaction: Pubkey,
    pub reaction_type: ReactionType,
    pub user: Pubkey,
    pub from_profile: Pubkey,
    pub to_post: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a reaction is deleted.
#[event]
pub struct ReactionDeleted {
    pub reaction: Pubkey,
    pub reaction_type: ReactionType,
    pub user: Pubkey,
    pub from_profile: Pubkey,
    pub to_post: Pubkey,
    pub timestamp: i64,
}

// This event is emitted whenever a new profile metadata is created.
#[event]
pub struct ProfileMetadataNew {
    pub profile_metadata: Pubkey,
    pub profile: Pubkey,
    pub user: Pubkey,
    pub metadata_uri: String,
    pub timestamp: i64,
}

// This event is emitted whenever a profile metadata is updated.
#[event]
pub struct ProfileMetadataUpdated {
    pub profile_metadata: Pubkey,
    pub profile: Pubkey,
    pub user: Pubkey,
    pub metadata_uri: String,
    pub timestamp: i64,
}

// This event is emitted whenever a profile metadata is deleted.
#[event]
pub struct ProfileMetadataDeleted {
    pub profile_metadata: Pubkey,
    pub profile: Pubkey,
    pub user: Pubkey,
    pub timestamp: i64,
}
