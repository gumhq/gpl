use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
mod utils;

use crate::instructions::*;

declare_id!("41kNwkQ9jESNYZJyAA1ENscQfx7vfkEf6uetVSFmfyaW");

#[program]
pub mod gpl_compression {
    use super::*;

    // initialize tree
    pub fn initialize_tree(
        ctx: Context<InitializeTreeConfig>,
        max_depth: u32,
        max_buffer_size: u32,
    ) -> Result<()> {
        initialize_tree_handler(ctx, max_depth, max_buffer_size)
    }

    // create a compressed post
    pub fn create_compressed_post(
        ctx: Context<CreateCompressedPost>,
        metadata_uri: String,
        random_hash: [u8; 32],
    ) -> Result<()> {
        create_compressed_post_handler(ctx, metadata_uri, random_hash)
    }

    // update a compressed post
    pub fn update_compressed_post<'info>(
        ctx: Context<'_, '_, '_, 'info, UpdateCompressedPost<'info>>,
        metadata_uri: String,
        new_metadata_uri: String,
        random_hash: [u8; 32],
        root: [u8; 32],
        index: u32,
    ) -> Result<()> {
        update_compressed_post_handler(
            ctx,
            metadata_uri,
            new_metadata_uri,
            random_hash,
            root,
            index,
        )
    }
}

#[error_code]
pub enum GplCompressionError {
    #[msg("Invalid authority provided")]
    AssetIDNotFound,
}

#[event]
pub struct CompressedPostNew {
    pub asset_id: Pubkey,
    pub post_id: Pubkey,
    pub post_bump: u8,
    pub index: u32,
    pub profile: Pubkey,
    pub user: Pubkey,
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
    pub user: Pubkey,
    pub random_hash: [u8; 32],
    pub metadata_uri: String,
    pub timestamp: i64,
}
