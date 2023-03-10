use crate::state::{Namespace, ProfileV2, User};
use anchor_lang::prelude::*;
use std::convert::AsRef;
use std::str::FromStr;

use crate::constants::*;
use crate::events::{ProfileDeleted, ProfileNew};

// Initialize a new profile account
#[derive(Accounts)]
#[instruction(namespace: String, metadata_uri: String)]
pub struct CreateProfileV2<'info> {
    // The account that will be initialized as a Profile
    #[account(
        init,
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            namespace.as_bytes(),
            user.to_account_info().key.as_ref()
        ],
        bump,
        payer = authority,
        space = ProfileV2::LEN
    )]
    pub profile_v2: Account<'info, ProfileV2>,
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
pub fn create_profile_v2_handler(
    ctx: Context<CreateProfileV2>,
    namespace: String,
    metadata_uri: String,
) -> Result<()> {
    // TODO: validate screen_name

    let profile = &mut ctx.accounts.profile_v2;
    profile.set_inner(ProfileV2 {
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
    });
    Ok(())
}

// Update a profile account
#[derive(Accounts)]
#[instruction(metadata_uri: String)]
pub struct UpdateProfileV2<'info> {
    // The account that will be initialized as a Profile
    #[account(
        mut,
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile_v2.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref()
        ],
        bump,
        has_one = user,
    )]
    pub profile_v2: Account<'info, ProfileV2>,
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
}

// Handler to update a Profile account
pub fn update_profile_v2_handler(
    ctx: Context<UpdateProfileV2>,
    metadata_uri: String,
) -> Result<()> {
    let profile = &mut ctx.accounts.profile_v2;
    profile.metadata_uri = metadata_uri;
    Ok(())
}

// Delete a profile account
#[derive(Accounts)]
pub struct DeleteProfileV2<'info> {
    // The Profile account to delete
    #[account(
        mut,
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            profile_v2.namespace.as_ref().as_bytes(),
            profile_v2.user.as_ref(),
        ],
        bump,
        has_one = user,
        close = authority,
    )]
    pub profile_v2: Account<'info, ProfileV2>,
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
pub fn delete_profile_v2_handler(ctx: Context<DeleteProfileV2>) -> Result<()> {
    // Emit profile deleted event
    emit!(ProfileDeleted {
        profile: *ctx.accounts.profile_v2.to_account_info().key,
        namespace: ctx.accounts.profile_v2.namespace,
        user: *ctx.accounts.user.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
