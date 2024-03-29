use std::convert::AsRef;

use gpl_core::state::{Post, Profile};

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hashv;

use gpl_core::constants::*;

use gpl_core::errors::PostError;
use gpl_core::program::GplCore;
use gpl_core::state::MAX_LEN_URI;

use spl_account_compression::program::SplAccountCompression;
use spl_account_compression::wrap_application_data_v1;
use spl_account_compression::Noop;

use crate::events::CompressedCommentNew;
use crate::state::TreeConfig;
use crate::utils::LeafSchema;
use crate::utils::{append_leaf, try_find_asset_id, verify_leaf};

// Create Comment
#[derive(Accounts)]
#[instruction(reply_to: Pubkey, metadata_uri: String, random_hash: [u8; 32], post_root: [u8; 32], post_leaf: [u8; 32], post_index: u32)]
pub struct CreateCompressedComment<'info> {
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.random_hash.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = authority,
    )]
    pub from_profile: Account<'info, Profile>,

    #[account(seeds = [merkle_tree.key.as_ref()], bump)]
    pub tree_config: Account<'info, TreeConfig>,

    #[account(mut)]
    /// CHECK The account must have the same authority as that of the config
    pub merkle_tree: UncheckedAccount<'info>,

    // TODO: The seeds should be more descriptive
    #[account(seeds = [target_merkle_tree.key.as_ref()], bump)]
    pub target_tree_config: Account<'info, TreeConfig>,

    /// CHECK The account must have the same authority as that of the config
    pub target_merkle_tree: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub compression_program: Program<'info, SplAccountCompression>,
    pub log_wrapper_program: Program<'info, Noop>,
    pub gpl_core_program: Program<'info, GplCore>,
    pub system_program: Program<'info, System>,
}

// Handler to create a compressed comment
pub fn create_compressed_comment_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateCompressedComment<'info>>,
    reply_to: Pubkey,
    metadata_uri: String,
    random_hash: [u8; 32],
    post_root: [u8; 32],
    post_leaf: [u8; 32],
    post_index: u32,
) -> Result<()> {
    require!(metadata_uri.len() <= MAX_LEN_URI, PostError::URITooLong);

    // Check if the reply_to exists
    // FIXME:
    // They can potentially pass any proof. How do we verify this belongs to the asset unless we
    // construct the leaf ourselves?
    verify_leaf(
        ctx.accounts.target_merkle_tree.key,
        ctx.bumps["target_tree_config"],
        post_root,
        post_leaf,
        post_index,
        ctx.remaining_accounts,
        &ctx.accounts.target_merkle_tree,
        &ctx.accounts.compression_program,
    )?;

    let post_seeds = [POST_PREFIX_SEED.as_bytes(), random_hash.as_ref()];

    let (post_id, post_bump) =
        Pubkey::try_find_program_address(&post_seeds, &GplCore::id()).unwrap();

    let seed_hash = hashv(&post_seeds).to_bytes();

    let asset_id = try_find_asset_id(ctx.accounts.merkle_tree.key, seed_hash)?;

    let post = Post {
        metadata_uri,
        random_hash,
        profile: *ctx.accounts.from_profile.to_account_info().key,
        reply_to: Some(reply_to),
    };

    let leaf = LeafSchema {
        asset_id,
        seed_hash,
        data_hash: hashv(&[&post.try_to_vec()?]).to_bytes(),
    };

    let leaf_node = leaf.to_node()?;

    wrap_application_data_v1(leaf_node.to_vec(), &ctx.accounts.log_wrapper_program)?;

    append_leaf(
        ctx.accounts.merkle_tree.key,
        ctx.bumps["tree_config"],
        &ctx.accounts.authority.to_account_info(),
        leaf_node,
        &ctx.accounts.merkle_tree,
        &ctx.accounts.compression_program,
        &ctx.accounts.log_wrapper_program,
    )?;

    emit!(CompressedCommentNew {
        asset_id,
        post_id,
        post_bump,
        reply_to,
        profile: *ctx.accounts.from_profile.to_account_info().key,
        random_hash: random_hash,
        metadata_uri: post.metadata_uri.clone(),
        timestamp: Clock::get()?.unix_timestamp,
        index: 0 // TODO: Get the index from the tree
    });
    Ok(())
}
