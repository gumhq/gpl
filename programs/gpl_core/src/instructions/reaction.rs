use crate::state::{Post, Profile, Reaction, ReactionType};
use anchor_lang::prelude::*;
use std::convert::AsRef;
use std::str::FromStr;

use crate::constants::*;
use crate::events::{ReactionDeleted, ReactionNew};
// Create a reaction to a post from a profile
#[derive(Accounts)]
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
    pub to_post: Account<'info, Post>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub from_profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Reaction account
pub fn create_reaction_handler(ctx: Context<CreateReaction>, reaction_type: String) -> Result<()> {
    let reaction = &mut ctx.accounts.reaction;
    reaction.reaction_type = ReactionType::from_str(&reaction_type).unwrap();
    reaction.to_post = *ctx.accounts.to_post.to_account_info().key;
    reaction.from_profile = *ctx.accounts.from_profile.to_account_info().key;

    // emit a new reaction event
    emit!(ReactionNew {
        reaction: *reaction.to_account_info().key,
        reaction_type: reaction.reaction_type,
        to_post: *ctx.accounts.to_post.to_account_info().key,
        from_profile: *ctx.accounts.from_profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Delete a reaction account
#[derive(Accounts)]
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
        close = authority,
    )]
    pub reaction: Account<'info, Reaction>,
    pub to_post: Account<'info, Post>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub from_profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to delete a Reaction account
pub fn delete_reaction_handler(ctx: Context<DeleteReaction>) -> Result<()> {
    // emit a reaction deleted event
    emit!(ReactionDeleted {
        reaction: *ctx.accounts.reaction.to_account_info().key,
        reaction_type: ctx.accounts.reaction.reaction_type,
        to_post: *ctx.accounts.to_post.to_account_info().key,
        from_profile: *ctx.accounts.from_profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
