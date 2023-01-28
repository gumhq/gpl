use crate::state::TreeConfig;
use crate::utils::append_leaf;
use crate::utils::replace_leaf;
use crate::utils::try_find_asset_id;
use crate::utils::LeafSchema;
use crate::CompressedPostNew;
use crate::CompressedPostUpdated;

use gpl_core::errors::PostError;
use spl_account_compression::wrap_application_data_v1;
use spl_account_compression::Node;

use gpl_core::state::{Post, Profile, User, MAX_LEN_URI};

use anchor_lang::prelude::*;
use std::convert::AsRef;

use gpl_core::constants::*;

use anchor_lang::solana_program::keccak::hashv;
use gpl_core::program::GplCore;
use spl_account_compression::program::SplAccountCompression;
use spl_account_compression::Noop;

// Create Post
#[derive(Accounts)]
#[instruction(metadata_uri: String, random_hash: [u8;32])]
pub struct CreateCompressedPost<'info> {
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = user,
    )]
    pub profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = authority,
    )]
    pub user: Account<'info, User>,

    #[account(seeds = [merkle_tree.key.as_ref()], bump)]
    pub tree_config: Account<'info, TreeConfig>,

    #[account(mut)]
    /// CHECK The account must have the same authority as that of the config
    pub merkle_tree: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub compression_program: Program<'info, SplAccountCompression>,
    pub log_wrapper_program: Program<'info, Noop>,
    pub gpl_core_program: Program<'info, GplCore>,
    pub system_program: Program<'info, System>,
}

// Handler to create a new Post account
pub fn create_compressed_post_handler(
    ctx: Context<CreateCompressedPost>,
    metadata_uri: String,
    random_hash: [u8; 32],
) -> Result<()> {
    require!(metadata_uri.len() <= MAX_LEN_URI, PostError::URITooLong);

    let post_seeds = [POST_PREFIX_SEED.as_bytes(), random_hash.as_ref()];

    let (post_id, post_bump) =
        Pubkey::try_find_program_address(&post_seeds, &GplCore::id()).unwrap();

    let seed_hash = hashv(&post_seeds).to_bytes();

    let asset_id = try_find_asset_id(ctx.accounts.merkle_tree.key, seed_hash)?;

    let post = Post {
        metadata_uri,
        random_hash,
        profile: *ctx.accounts.profile.to_account_info().key,
        reply_to: None,
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

    emit!(CompressedPostNew {
        asset_id,
        post_id,
        post_bump,
        profile: *ctx.accounts.profile.to_account_info().key,
        user: *ctx.accounts.user.to_account_info().key,
        random_hash: random_hash,
        metadata_uri: post.metadata_uri.clone(),
        timestamp: Clock::get()?.unix_timestamp,
        index: 0
    });

    Ok(())
}

// Update a post account
#[derive(Accounts)]
#[instruction(metadata_uri: String, new_metadata_uri: String, random_hash: [u8;32], root: [u8;32], index: u32)]
pub struct UpdateCompressedPost<'info> {
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = user,
    )]
    pub profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = authority,
    )]
    pub user: Account<'info, User>,
    #[account(seeds = [merkle_tree.key.as_ref()], bump)]
    pub tree_config: Account<'info, TreeConfig>,

    #[account(mut)]
    /// CHECK The account must have the same authority as that of the config
    pub merkle_tree: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub compression_program: Program<'info, SplAccountCompression>,
    pub log_wrapper_program: Program<'info, Noop>,
    pub gpl_core_program: Program<'info, GplCore>,
    pub system_program: Program<'info, System>,
}

// Handler to update a Post account
pub fn update_compressed_post_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, UpdateCompressedPost<'info>>,
    metadata_uri: String,
    new_metadata_uri: String,
    random_hash: [u8; 32],
    root: [u8; 32],
    index: u32,
) -> Result<()> {
    // CHECK metadata_uri length
    require!(metadata_uri.len() <= MAX_LEN_URI, PostError::URITooLong);

    let post_seeds = [POST_PREFIX_SEED.as_bytes(), random_hash.as_ref()];

    let (post_id, post_bump) =
        Pubkey::try_find_program_address(&post_seeds, &GplCore::id()).unwrap();

    let seed_hash = hashv(&post_seeds).to_bytes();

    let asset_id = try_find_asset_id(ctx.accounts.merkle_tree.key, seed_hash)?;

    let old_post = Post {
        metadata_uri,
        random_hash,
        profile: *ctx.accounts.profile.to_account_info().key,
        reply_to: None,
    };

    let old_leaf = LeafSchema {
        asset_id,
        seed_hash,
        // May be better as a trait?
        data_hash: hashv(&[&old_post.try_to_vec()?]).to_bytes(),
    };

    let old_leaf_node = old_leaf.to_node()?;

    let new_post = Post {
        metadata_uri: new_metadata_uri,
        random_hash,
        profile: *ctx.accounts.profile.to_account_info().key,
        reply_to: None,
    };

    let new_leaf = LeafSchema {
        asset_id,
        seed_hash,
        // May be better as a trait?
        data_hash: hashv(&[&new_post.try_to_vec()?]).to_bytes(),
    };

    let new_leaf_node = new_leaf.to_node()?;

    wrap_application_data_v1(new_leaf_node.to_vec(), &ctx.accounts.log_wrapper_program)?;

    replace_leaf(
        ctx.accounts.merkle_tree.key,
        ctx.bumps["tree_config"],
        &ctx.accounts.authority.to_account_info(),
        &ctx.accounts.merkle_tree,
        root,
        old_leaf_node,
        new_leaf_node,
        index,
        ctx.remaining_accounts,
        &ctx.accounts.compression_program,
        &ctx.accounts.log_wrapper_program,
    )?;

    // emit update post event
    emit!(CompressedPostUpdated {
        asset_id,
        post_id,
        post_bump,
        profile: *ctx.accounts.profile.to_account_info().key,
        user: *ctx.accounts.user.to_account_info().key,
        random_hash: random_hash,
        metadata_uri: new_post.metadata_uri.clone(),
        timestamp: Clock::get()?.unix_timestamp,
        index: index
    });

    Ok(())
}

// // Delete a post account
// #[derive(Accounts)]
// pub struct DeletePost<'info> {
//     // The Post account to delete
//     #[account(
//         mut,
//         seeds = [
//             POST_PREFIX_SEED.as_bytes(),
//             post.random_hash.as_ref(),
//         ],
//         bump,
//         has_one = profile,
//         close = authority,
//     )]
//     pub post: Account<'info, Post>,
//     #[account(
//         seeds = [
//             PROFILE_PREFIX_SEED.as_bytes(),
//             profile.namespace.as_ref().as_bytes(),
//             user.to_account_info().key.as_ref(),
//         ],
//         bump,
//         has_one = user,
//     )]
//     pub profile: Account<'info, Profile>,
//     #[account(
//         seeds = [
//             USER_PREFIX_SEED.as_bytes(),
//             user.random_hash.as_ref(),
//         ],
//         bump,
//         has_one = authority,
//     )]
//     pub user: Account<'info, User>,
//     #[account(mut)]
//     pub authority: Signer<'info>,
//     pub system_program: Program<'info, System>,
// }

// // Handler to delete a Post account
// pub fn delete_post_handler(ctx: Context<DeletePost>) -> Result<()> {
//     // emit delete post event
//     emit!(PostDeleted {
//         post: *ctx.accounts.post.to_account_info().key,
//         profile: *ctx.accounts.profile.to_account_info().key,
//         user: *ctx.accounts.user.to_account_info().key,
//         timestamp: Clock::get()?.unix_timestamp,
//     });
//     Ok(())
// }

// // Create a comment as a new post account with reply_to set to the parent post
// #[derive(Accounts)]
// #[instruction(metadata_uri: String, random_hash: [u8;32])]
// pub struct CreateComment<'info> {
//     // The account that will be initialized as a Post
//     #[account(
//         init,
//         seeds = [
//             POST_PREFIX_SEED.as_bytes(),
//             random_hash.as_ref(),
//         ],
//         bump,
//         payer = authority,
//         space = Post::LEN
//     )]
//     pub post: Account<'info, Post>,
//     #[account(
//         seeds = [
//             PROFILE_PREFIX_SEED.as_bytes(),
//             profile.namespace.as_ref().as_bytes(),
//             user.to_account_info().key.as_ref(),
//         ],
//         bump,
//         has_one = user,
//     )]
//     pub profile: Account<'info, Profile>,
//     #[account(
//         seeds = [
//             USER_PREFIX_SEED.as_bytes(),
//             user.random_hash.as_ref(),
//         ],
//         bump,
//         has_one = authority,
//     )]
//     pub user: Account<'info, User>,
//     #[account(
//         seeds = [
//             POST_PREFIX_SEED.as_bytes(),
//             reply_to.random_hash.as_ref(),
//         ],
//         bump,
//     )]
//     pub reply_to: Account<'info, Post>,
//     #[account(mut)]
//     pub authority: Signer<'info>,
//     // The system program
//     pub system_program: Program<'info, System>,
// }

// // Handler to add a comment to a post
// pub fn create_comment_handler(
//     ctx: Context<CreateComment>,
//     metadata_uri: String,
//     random_hash: [u8; 32],
// ) -> Result<()> {
//     // Check metadata_uri length
//     require!(metadata_uri.len() <= MAX_LEN_URI, PostError::URITooLong);

//     let post = &mut ctx.accounts.post;
//     post.metadata_uri = metadata_uri;
//     post.random_hash = random_hash;
//     post.profile = *ctx.accounts.profile.to_account_info().key;
//     post.reply_to = Some(*ctx.accounts.reply_to.to_account_info().key);
//     // emit new comment event
//     emit!(PostCommentNew {
//         post: *post.to_account_info().key,
//         profile: *ctx.accounts.profile.to_account_info().key,
//         user: *ctx.accounts.user.to_account_info().key,
//         random_hash: random_hash,
//         metadata_uri: post.metadata_uri.clone(),
//         reply_to: *ctx.accounts.reply_to.to_account_info().key,
//         timestamp: Clock::get()?.unix_timestamp,
//     });
//     Ok(())
// }
