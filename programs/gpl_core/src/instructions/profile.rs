use crate::state::Profile;
use anchor_lang::prelude::*;

use crate::constants::*;
use crate::events::{ProfileDeleted, ProfileNew, ProfileUpdated};

use gpl_nameservice::validate as validate_screen_name;

// Initialize a new profile account
#[derive(Accounts)]
#[instruction(random_hash: [u8; 32], metadata_uri: String)]
pub struct CreateProfile<'info> {
    #[account(
        init,
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            random_hash.as_ref(),
        ],
        bump,
        payer = authority,
        space = Profile::LEN
    )]
    pub profile: Account<'info, Profile>,
    /// CHECK that this PDA is either SNS, ANS or GPL Nameservice
    #[account(
        constraint = validate_screen_name(&[screen_name.clone(), authority.to_account_info()])?,
    )]
    pub screen_name: AccountInfo<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Profile account
pub fn create_profile_handler(
    ctx: Context<CreateProfile>,
    random_hash: [u8; 32],
    metadata_uri: String,
) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    profile.set_inner(Profile {
        authority: *ctx.accounts.authority.key,
        random_hash,
        metadata_uri,
        screen_name: *ctx.accounts.screen_name.key,
    });
    // Emit new profile event
    emit!(ProfileNew {
        profile: *profile.to_account_info().key,
        authority: *ctx.accounts.authority.key,
        random_hash,
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
            profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub profile: Account<'info, Profile>,

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
            profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
        close = authority,
    )]
    pub profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

// Handler to close a profile account
pub fn delete_profile_handler(ctx: Context<DeleteProfile>) -> Result<()> {
    // Emit profile deleted event
    emit!(ProfileDeleted {
        profile: *ctx.accounts.profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
        screen_name: ctx.accounts.profile.screen_name,
        metadata_uri: ctx.accounts.profile.metadata_uri.clone(),
    });
    Ok(())
}
