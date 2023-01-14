use crate::state::{Connection, Profile, User};
use anchor_lang::prelude::*;
use std::convert::AsRef;

use crate::constants::*;

// Create a connection between two profiles, ie from_profile -> to_profile
#[derive(Accounts)]
pub struct CreateConnection<'info> {
    // The account that will be initialized as a Connection
    #[account(
        init,
        seeds = [
            CONNECTION_PREFIX_SEED.as_bytes(),
            from_profile.user.as_ref(),
            to_profile.user.as_ref(),
        ],
        bump,
        payer = authority,
        space = Connection::LEN
    )]
    pub connection: Account<'info, Connection>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.namespace.as_ref().as_bytes(),
            from_profile.user.as_ref(),
        ],
        bump = from_profile.bump,
        has_one = user,
    )]
    pub from_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            to_profile.namespace.as_ref().as_bytes(),
            to_profile.user.as_ref(),
        ],
        bump = to_profile.bump,
    )]
    pub to_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump = user.bump
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Connection account
pub fn create_connection_handler(ctx: Context<CreateConnection>) -> Result<()> {
    let connection = &mut ctx.accounts.connection;
    connection.from_profile = *ctx.accounts.from_profile.to_account_info().key;
    connection.to_profile = *ctx.accounts.to_profile.to_account_info().key;
    connection.bump = ctx.bumps["connection"];
    Ok(())
}

// Delete a connection between two profiles, ie from_profile -> to_profile
#[derive(Accounts)]
pub struct DeleteConnection<'info> {
    // The Connection account to delete
    #[account(
        mut,
        seeds = [
            CONNECTION_PREFIX_SEED.as_bytes(),
            from_profile.user.as_ref(),
            to_profile.user.as_ref(),
        ],
        bump = connection.bump,
        has_one = from_profile,
        has_one = to_profile,
    )]
    pub connection: Account<'info, Connection>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.namespace.as_ref().as_bytes(),
            from_profile.user.as_ref(),
        ],
        bump = from_profile.bump,
        has_one = user,
    )]
    pub from_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            to_profile.namespace.as_ref().as_bytes(),
            to_profile.user.as_ref(),
        ],
        bump = to_profile.bump,
    )]
    pub to_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump = user.bump
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

// Handler to delete a Connection account
pub fn delete_connection_handler(_ctx: Context<DeleteConnection>) -> Result<()> {
    Ok(())
}
