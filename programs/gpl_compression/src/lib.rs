use anchor_lang::prelude::*;
use solana_security_txt::security_txt;

mod errors;
mod events;
pub mod instructions;
pub mod state;
mod utils;

use crate::errors::GplCompressionError;
use crate::instructions::*;

declare_id!("41kNwkQ9jESNYZJyAA1ENscQfx7vfkEf6uetVSFmfyaW");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "gpl_compression",
    project_url: "https://gum.fun",
    contacts: "email:hello@gum.fun,twitter:@gumhq",
    policy: "",
    preferred_languages: "en",
    source_code: "https://github.com/gumhq/gpl"
}

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

    // delete a compressed post
    pub fn delete_compressed_post<'info>(
        ctx: Context<'_, '_, '_, 'info, DeleteCompressedPost<'info>>,
        metadata_uri: String,
        random_hash: [u8; 32],
        root: [u8; 32],
        index: u32,
    ) -> Result<()> {
        delete_compressed_post_handler(ctx, metadata_uri, random_hash, root, index)
    }

    // create a compressed connection
    pub fn create_compressed_connection(ctx: Context<CreateCompressedConnection>) -> Result<()> {
        create_compressed_connection_handler(ctx)
    }

    // delete a compressed connection
    pub fn delete_compressed_connection<'info>(
        ctx: Context<'_, '_, '_, 'info, DeleteCompressedConnection<'info>>,
        root: [u8; 32],
        index: u32,
    ) -> Result<()> {
        delete_compressed_connection_handler(ctx, root, index)
    }

    // create a compressed reaction
    pub fn create_compressed_reaction<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateCompressedReaction<'info>>,
        to_post: Pubkey,
        reaction_type: String,
        post_root: [u8; 32],
        post_leaf: [u8; 32],
        post_index: u32,
    ) -> Result<()> {
        create_compressed_reaction_handler(
            ctx,
            to_post,
            reaction_type,
            post_root,
            post_leaf,
            post_index,
        )
    }

    // delete a compressed reaction
    pub fn delete_compressed_reaction<'info>(
        ctx: Context<'_, '_, '_, 'info, DeleteCompressedReaction<'info>>,
        to_post: Pubkey,
        reaction_type: String,
        root: [u8; 32],
        index: u32,
    ) -> Result<()> {
        delete_compressed_reaction_handler(ctx, to_post, reaction_type, root, index)
    }

    // create a compressed comment
    pub fn create_compressed_comment<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateCompressedComment<'info>>,
        reply_to: Pubkey,
        metadata_uri: String,
        random_hash: [u8; 32],
        post_root: [u8; 32],
        post_leaf: [u8; 32],
        post_index: u32,
    ) -> Result<()> {
        create_compressed_comment_handler(
            ctx,
            reply_to,
            metadata_uri,
            random_hash,
            post_root,
            post_leaf,
            post_index,
        )
    }
}
