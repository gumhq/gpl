use anchor_lang::prelude::*;
use std::mem::size_of;

// Account to hold the compressed data in a tree
#[account]
pub struct TreeConfig {
    pub authority: Pubkey,
    pub merkle_tree: Pubkey,
}

impl TreeConfig {
    pub const LEN: usize = 8 + size_of::<Self>();
}
