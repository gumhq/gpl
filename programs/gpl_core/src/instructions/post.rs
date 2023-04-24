use crate::errors::PostError;
use crate::events::{PostCommentNew, PostDeleted, PostNew, PostUpdated};
use crate::state::{Post, Profile, MAX_LEN_URI};

use anchor_lang::prelude::*;
use std::convert::AsRef;

use crate::constants::*;

// Create Post
#[derive(Accounts)]
#[instruction(metadata_uri: String, random_hash: [u8;32])]
pub struct CreatePost<'info> {
    // The account that will be initialized as a Post
    #[account(
        init,
        seeds = [
            POST_PREFIX_SEED.as_bytes(),
            random_hash.as_ref(),
        ],
        bump,
        payer = authority,
        space = Post::LEN
    )]
    pub post: Account<'info, Post>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Post account
pub fn create_post_handler(
    ctx: Context<CreatePost>,
    metadata_uri: String,
    random_hash: [u8; 32],
) -> Result<()> {
    // CHECK metadata_uri length
    require!(metadata_uri.len() <= MAX_LEN_URI, PostError::URITooLong);

    let post = &mut ctx.accounts.post;
    post.metadata_uri = metadata_uri;
    post.random_hash = random_hash;
    post.profile = *ctx.accounts.profile.to_account_info().key;
    // emit new post event
    emit!(PostNew {
        post: *post.to_account_info().key,
        profile: *ctx.accounts.profile.to_account_info().key,
        random_hash: random_hash,
        metadata_uri: post.metadata_uri.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

// Update a post account
#[derive(Accounts)]
#[instruction(metadata_uri: String)]
pub struct UpdatePost<'info> {
    // The Post account to update
    #[account(
        mut,
        seeds = [
            POST_PREFIX_SEED.as_bytes(),
            post.random_hash.as_ref(),
        ],
        bump,
        has_one = profile,
    )]
    pub post: Account<'info, Post>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to update a Post account
pub fn update_post_handler(ctx: Context<UpdatePost>, metadata_uri: String) -> Result<()> {
    // CHECK metadata_uri length
    require!(metadata_uri.len() <= MAX_LEN_URI, PostError::URITooLong);
    let post = &mut ctx.accounts.post;
    post.metadata_uri = metadata_uri;
    // emit update post event
    emit!(PostUpdated {
        post: *post.to_account_info().key,
        profile: *ctx.accounts.profile.to_account_info().key,
        metadata_uri: post.metadata_uri.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

// Create a comment as a new post account with reply_to set to the parent post
#[derive(Accounts)]
#[instruction(metadata_uri: String, random_hash: [u8;32])]
pub struct CreateComment<'info> {
    // The account that will be initialized as a Post
    #[account(
        init,
        seeds = [
            POST_PREFIX_SEED.as_bytes(),
            random_hash.as_ref(),
        ],
        bump,
        payer = authority,
        space = Post::LEN
    )]
    pub post: Account<'info, Post>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub profile: Account<'info, Profile>,
    #[account(
        seeds = [
            POST_PREFIX_SEED.as_bytes(),
            reply_to.random_hash.as_ref(),
        ],
        bump,
    )]
    pub reply_to: Account<'info, Post>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to add a comment to a post
pub fn create_comment_handler(
    ctx: Context<CreateComment>,
    metadata_uri: String,
    random_hash: [u8; 32],
) -> Result<()> {
    // Check metadata_uri length
    require!(metadata_uri.len() <= MAX_LEN_URI, PostError::URITooLong);

    let post = &mut ctx.accounts.post;
    post.metadata_uri = metadata_uri;
    post.random_hash = random_hash;
    post.profile = *ctx.accounts.profile.to_account_info().key;
    post.reply_to = Some(*ctx.accounts.reply_to.to_account_info().key);
    // emit new comment event
    emit!(PostCommentNew {
        post: *post.to_account_info().key,
        profile: *ctx.accounts.profile.to_account_info().key,
        random_hash: random_hash,
        metadata_uri: post.metadata_uri.clone(),
        reply_to: *ctx.accounts.reply_to.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

// Delete a post account
#[derive(Accounts)]
pub struct DeletePost<'info> {
    // The Post account to delete
    #[account(
        mut,
        seeds = [
            POST_PREFIX_SEED.as_bytes(),
            post.random_hash.as_ref(),
        ],
        bump,
        has_one = profile,
        close = authority,
    )]
    pub post: Account<'info, Post>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to delete a Post account
pub fn delete_post_handler(ctx: Context<DeletePost>) -> Result<()> {
    // emit delete post event
    emit!(PostDeleted {
        post: *ctx.accounts.post.to_account_info().key,
        profile: *ctx.accounts.profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
