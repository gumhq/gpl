use crate::errors::GumError;
use crate::state::{Post, Profile, Reaction, ReactionType, User};

use anchor_lang::prelude::*;
use std::convert::AsRef;
use std::str::FromStr;

use crate::constants::*;
use crate::events::{ReactionDeleted, ReactionNew};
use gpl_session::gpl_session::{session_auth_or, Session};
use gpl_session::{Session, SessionError, SessionToken};

// Create a reaction to a post from a profile
#[derive(Accounts, Session)]
#[instruction(reaction_type: String)]
pub struct CreateReaction<'info> {
    // The account that will be initialized as a Reaction
    #[account(
        init,
        seeds = [
            REACTION_PREFIX_SEED.as_bytes(),
            reaction_type.as_bytes(),
            to_post.to_account_info().key.as_ref(),
            from_profile.to_account_info().key.as_ref(),
        ],
        bump,
        payer = authority,
        space = Reaction::LEN
    )]
    pub reaction: Account<'info, Reaction>,
    #[account(
        seeds = [
            POST_PREFIX_SEED.as_bytes(),
            to_post.random_hash.as_ref(),
        ],
        bump,
    )]
    pub to_post: Account<'info, Post>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
        ],
        bump,
        has_one = user,
    )]
    pub from_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump,
    )]
    pub user: Account<'info, User>,

    #[session(
        signer = authority.key(),
        authority = user.authority.key()
    )]
    pub session_token: Option<Account<'info, SessionToken>>,

    #[account(mut)]
    pub authority: Signer<'info>,

    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Reaction account
#[session_auth_or(
    ctx.accounts.user.authority.key() == ctx.accounts.authority.key(),
    GumError::UnauthorizedSigner
)]
pub fn create_reaction_handler(ctx: Context<CreateReaction>, reaction_type: String) -> Result<()> {
    let reaction = &mut ctx.accounts.reaction;
    reaction.reaction_type = ReactionType::from_str(&reaction_type).unwrap();
    reaction.to_post = *ctx.accounts.to_post.to_account_info().key;
    reaction.from_profile = *ctx.accounts.from_profile.to_account_info().key;

    // emit a new reaction event
    emit!(ReactionNew {
        reaction: *reaction.to_account_info().key,
        reaction_type: reaction.reaction_type,
        user: *ctx.accounts.user.to_account_info().key,
        to_post: *ctx.accounts.to_post.to_account_info().key,
        from_profile: *ctx.accounts.from_profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Delete a reaction account
#[derive(Accounts, Session)]
pub struct DeleteReaction<'info> {
    #[account(
        mut,
        seeds = [
            REACTION_PREFIX_SEED.as_bytes(),
            reaction.reaction_type.as_ref().as_bytes(),
            reaction.to_post.as_ref(),
            reaction.from_profile.as_ref(),
        ],
        bump,
        has_one = to_post,
        has_one = from_profile,
        close = refund_receiver,
    )]
    pub reaction: Account<'info, Reaction>,
    #[account(
        seeds = [
            POST_PREFIX_SEED.as_bytes(),
            to_post.random_hash.as_ref(),
        ],
        bump,
    )]
    pub to_post: Account<'info, Post>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
        ],
        bump,
        has_one = user,
    )]
    pub from_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump,
    )]
    pub user: Account<'info, User>,

    #[session(
        signer = authority.key(),
        authority = user.authority.key()
    )]
    pub session_token: Option<Account<'info, SessionToken>>,
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(mut, constraint = refund_receiver.key() == user.authority)]
    pub refund_receiver: SystemAccount<'info>,

    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to delete a Reaction account
#[session_auth_or(
    ctx.accounts.user.authority.key() == ctx.accounts.authority.key(),
    GumError::UnauthorizedSigner
)]
pub fn delete_reaction_handler(ctx: Context<DeleteReaction>) -> Result<()> {
    // emit a reaction deleted event
    emit!(ReactionDeleted {
        reaction: *ctx.accounts.reaction.to_account_info().key,
        reaction_type: ctx.accounts.reaction.reaction_type,
        user: *ctx.accounts.user.to_account_info().key,
        to_post: *ctx.accounts.to_post.to_account_info().key,
        from_profile: *ctx.accounts.from_profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
