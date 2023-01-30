use crate::errors::ProfileMetadataError;
use crate::events::{ProfileMetadataDeleted, ProfileMetadataNew, ProfileMetadataUpdated};
use crate::state::{Profile, ProfileMetadata, User, MAX_LEN_URI};

use anchor_lang::prelude::*;
use std::convert::AsRef;

use crate::constants::*;

// Create ProfileMetadata
#[derive(Accounts)]
#[instruction(metadata_uri: String)]
pub struct CreateProfileMetadata<'info> {
    // The account that will be initialized as a ProfileMetadata
    #[account(
        init,
        seeds = [
            PROFILE_METADATA_PREFIX_SEED.as_bytes(),
            profile.to_account_info().key.as_ref(),
        ],
        bump,
        payer = authority,
        space = ProfileMetadata::LEN
    )]
    pub profile_metadata: Account<'info, ProfileMetadata>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
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
    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new ProfileMetadata account
pub fn create_profile_metadata_handler(
    ctx: Context<CreateProfileMetadata>,
    metadata_uri: String,
) -> Result<()> {
    // CHECK metadata_uri length
    require!(
        metadata_uri.len() <= MAX_LEN_URI,
        ProfileMetadataError::URITooLong
    );

    let profile_metadata = &mut ctx.accounts.profile_metadata;
    profile_metadata.metadata_uri = metadata_uri;
    profile_metadata.profile = *ctx.accounts.profile.to_account_info().key;
    // emit new ProfileMetadata event
    emit!(ProfileMetadataNew {
        profile_metadata: *profile_metadata.to_account_info().key,
        profile: *ctx.accounts.profile.to_account_info().key,
        user: *ctx.accounts.user.to_account_info().key,
        metadata_uri: profile_metadata.metadata_uri.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

// Update a ProfileMetadata account
#[derive(Accounts)]
#[instruction(metadata_uri: String)]
pub struct UpdateProfileMetadata<'info> {
    // The ProfileMetadata account to update
    #[account(
        mut,
        seeds = [
            PROFILE_METADATA_PREFIX_SEED.as_bytes(),
            profile.to_account_info().key.as_ref(),
        ],
        bump,
        has_one = profile,
    )]
    pub profile_metadata: Account<'info, ProfileMetadata>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
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
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to update a ProfileMetadata account
pub fn update_profile_metadata_handler(
    ctx: Context<UpdateProfileMetadata>,
    metadata_uri: String,
) -> Result<()> {
    // CHECK metadata_uri length
    require!(
        metadata_uri.len() <= MAX_LEN_URI,
        ProfileMetadataError::URITooLong
    );
    let profile_metadata = &mut ctx.accounts.profile_metadata;
    profile_metadata.metadata_uri = metadata_uri;
    // emit update ProfileMetadata event
    emit!(ProfileMetadataUpdated {
        profile_metadata: *profile_metadata.to_account_info().key,
        profile: *ctx.accounts.profile.to_account_info().key,
        user: *ctx.accounts.user.to_account_info().key,
        metadata_uri: profile_metadata.metadata_uri.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

// Delete a ProfileMetadata account
#[derive(Accounts)]
pub struct DeleteProfileMetadata<'info> {
    // The ProfileMetadata account to delete
    #[account(
        mut,
        seeds = [
            PROFILE_METADATA_PREFIX_SEED.as_bytes(),
            profile.to_account_info().key.as_ref(),
        ],
        bump,
        has_one = profile,
        close = authority,
    )]
    pub profile_metadata: Account<'info, ProfileMetadata>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
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
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to delete a ProfileMetadata account
pub fn delete_profile_metadata_handler(ctx: Context<DeleteProfileMetadata>) -> Result<()> {
    // emit delete ProfileMetadata event
    emit!(ProfileMetadataDeleted {
        profile_metadata: *ctx.accounts.profile_metadata.to_account_info().key,
        profile: *ctx.accounts.profile.to_account_info().key,
        user: *ctx.accounts.user.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
