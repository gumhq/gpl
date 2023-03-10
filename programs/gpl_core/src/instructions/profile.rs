use crate::state::{Namespace, Profile, User};
use anchor_lang::prelude::*;
use std::convert::AsRef;
use std::str::FromStr;

use crate::constants::*;
use crate::events::{ProfileDeleted, ProfileNew, ProfileUpdated};

// Initialize a new profile account
#[derive(Accounts)]
#[instruction(namespace: String, metadata_uri: String)]
pub struct CreateProfile<'info> {
    #[account(
        init,
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            namespace.as_bytes(),
            user.to_account_info().key.as_ref()
        ],
        bump,
        payer = authority,
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

    /// CHECK that this PDA is either SNS, ANS or GPL Nameservice
    pub screen_name: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Profile account
pub fn create_profile_handler(
    ctx: Context<CreateProfile>,
    namespace: String,
    metadata_uri: String,
) -> Result<()> {
    // TODO: validate screen_name

    let profile = &mut ctx.accounts.profile;
    profile.set_inner(Profile {
        user: *ctx.accounts.user.to_account_info().key,
        namespace: Namespace::from_str(&namespace).unwrap(),
        metadata_uri,
        screen_name: *ctx.accounts.screen_name.key,
    });
    // Emit new profile event
    emit!(ProfileNew {
        profile: *profile.to_account_info().key,
        namespace: profile.namespace,
        user: *ctx.accounts.user.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
        screen_name: profile.screen_name,
        metadata_uri: profile.metadata_uri.clone(),
    });
    Ok(())
}

// Update a profile account
#[derive(Accounts)]
#[instruction(metadata_uri: String)]
pub struct UpdateProfile<'info> {
    #[account(
        mut,
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref()
        ],
        bump,
        has_one = user,
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

    /// CHECK that this PDA is either SNS, ANS or GPL Nameservice and is owned by the user
    pub screen_name: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

// Handler to update a Profile account
pub fn update_profile_handler(ctx: Context<UpdateProfile>, metadata_uri: String) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    profile.metadata_uri = metadata_uri;
    // Emit a profile update event
    emit!(ProfileUpdated {
        profile: *profile.to_account_info().key,
        namespace: profile.namespace,
        user: *ctx.accounts.user.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
        screen_name: profile.screen_name,
        metadata_uri: profile.metadata_uri.clone(),
    });
    Ok(())
}

// Delete a profile account
#[derive(Accounts)]
pub struct DeleteProfile<'info> {
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
        screen_name: ctx.accounts.profile.screen_name,
        metadata_uri: ctx.accounts.profile.metadata_uri.clone(),
    });
    Ok(())
}
