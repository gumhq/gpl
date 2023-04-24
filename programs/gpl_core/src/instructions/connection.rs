use crate::state::{Connection, Profile};
use anchor_lang::prelude::*;
use std::convert::AsRef;

use crate::constants::*;
use crate::errors::ConnectionError;
use crate::events::{ConnectionDeleted, ConnectionNew};

// Create a connection between two profiles, ie from_profile -> to_profile
#[derive(Accounts)]
pub struct CreateConnection<'info> {
    // The account that will be initialized as a Connection
    #[account(
        init,
        seeds = [
            CONNECTION_PREFIX_SEED.as_bytes(),
            from_profile.key().as_ref(),
            to_profile.key().as_ref()
        ],
        bump,
        payer = authority,
        space = Connection::LEN
    )]
    pub connection: Account<'info, Connection>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub from_profile: Account<'info, Profile>,
    pub to_profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Connection account
pub fn create_connection_handler(ctx: Context<CreateConnection>) -> Result<()> {
    // CHECK that the from_profile and to_profile are not the same
    require_neq!(
        ctx.accounts.from_profile.key(),
        ctx.accounts.to_profile.key(),
        ConnectionError::CannotConnectToSelf
    );

    let connection = &mut ctx.accounts.connection;
    connection.from_profile = *ctx.accounts.from_profile.to_account_info().key;
    connection.to_profile = *ctx.accounts.to_profile.to_account_info().key;
    // emit a new connection event
    emit!(ConnectionNew {
        connection: *connection.to_account_info().key,
        from_profile: *ctx.accounts.from_profile.to_account_info().key,
        to_profile: *ctx.accounts.to_profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });

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
            from_profile.key().as_ref(),
            to_profile.key().as_ref()
        ],
        bump,
        has_one = from_profile,
        has_one = to_profile,
        close = authority,
    )]
    pub connection: Account<'info, Connection>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub from_profile: Account<'info, Profile>,
    pub to_profile: Account<'info, Profile>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

// Handler to delete a Connection account
pub fn delete_connection_handler(ctx: Context<DeleteConnection>) -> Result<()> {
    // emit a delete connection event
    emit!(ConnectionDeleted {
        connection: *ctx.accounts.connection.to_account_info().key,
        from_profile: *ctx.accounts.from_profile.to_account_info().key,
        to_profile: *ctx.accounts.to_profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
