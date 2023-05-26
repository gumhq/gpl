use crate::state::{Namespace, Profile, User};
use anchor_lang::prelude::*;
use std::convert::AsRef;
use std::str::FromStr;

use crate::constants::*;
use crate::events::{ProfileDeleted, ProfileNew};

// Initialize a new profile account
#[derive(Accounts)]
#[instruction(namespace: String)]
pub struct CreateProfile<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    // The account that will be initialized as a Profile
    #[account(
        init,
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            namespace.as_bytes(),
            user.to_account_info().key.as_ref()
        ],
        bump,
        payer = payer,
        space = Profile::LEN
    )]
    pub profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub user: Account<'info, User>,
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Profile account
pub fn create_profile_handler(ctx: Context<CreateProfile>, namespace: String) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    profile.namespace = Namespace::from_str(&namespace).unwrap();
    profile.user = *ctx.accounts.user.to_account_info().key;

    // Emit new profile event
    emit!(ProfileNew {
        profile: *profile.to_account_info().key,
        namespace: profile.namespace,
        user: *ctx.accounts.user.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

// Delete a profile account
#[derive(Accounts)]
pub struct DeleteProfile<'info> {
    // The Profile account to delete
    #[account(
        mut,
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.namespace.as_ref().as_bytes(),
            profile.user.as_ref(),
        ],
        bump,
        has_one = user,
        close = authority,
    )]
    pub profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

// Handler to close a profile account
pub fn delete_profile_handler(ctx: Context<DeleteProfile>) -> Result<()> {
    // Emit profile deleted event
    emit!(ProfileDeleted {
        profile: *ctx.accounts.profile.to_account_info().key,
        namespace: ctx.accounts.profile.namespace,
        user: *ctx.accounts.user.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
