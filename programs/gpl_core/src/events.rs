use crate::state::Namespace;
use anchor_lang::prelude::*;

// This event is emitted whenever a new user is created.
#[event]
pub struct UserNew {
    pub user: Pubkey,
    pub random_hash: [u8; 32],
    pub bump: u8,
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
    pub bump: u8,
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
