use crate::errors::GumError;
use crate::state::{Connection, Profile, User};
use anchor_lang::prelude::*;
use gpl_session::{session_auth_or, Session, SessionError, SessionToken};
use std::convert::AsRef;

use crate::constants::*;
use crate::errors::ConnectionError;
use crate::events::{ConnectionDeleted, ConnectionNew};

// Create a connection between two profiles, ie from_profile -> to_profile
#[derive(Accounts, Session)]
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
            from_profile.namespace.as_ref().as_bytes(),
            from_profile.user.as_ref(),
        ],
        bump,
        has_one = user,
    )]
    pub from_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            to_profile.namespace.as_ref().as_bytes(),
            to_profile.user.as_ref(),
        ],
        bump,
    )]
    pub to_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump,
    )]
    pub user: Account<'info, User>,

    #[session(
        signer = authority,
        authority = user.authority.key()
    )]
    pub session_token: Option<Account<'info, SessionToken>>,

    #[account(mut)]
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new Connection account
#[session_auth_or(
    ctx.accounts.user.authority.key() == ctx.accounts.authority.key(),
    GumError::UnauthorizedSigner
)]
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
        user: *ctx.accounts.user.to_account_info().key,
        from_profile: *ctx.accounts.from_profile.to_account_info().key,
        to_profile: *ctx.accounts.to_profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Delete a connection between two profiles, ie from_profile -> to_profile
#[derive(Accounts, Session)]
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
        close = refund_receiver,
    )]
    pub connection: Account<'info, Connection>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.namespace.as_ref().as_bytes(),
            from_profile.user.as_ref(),
        ],
        bump,
        has_one = user,
    )]
    pub from_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            to_profile.namespace.as_ref().as_bytes(),
            to_profile.user.as_ref(),
        ],
        bump,
    )]
    pub to_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump,
    )]
    pub user: Account<'info, User>,

    #[session(
        signer = authority,
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

// Handler to delete a Connection account
#[session_auth_or(
    ctx.accounts.user.authority.key() == ctx.accounts.authority.key(),
    GumError::UnauthorizedSigner
)]
pub fn delete_connection_handler(ctx: Context<DeleteConnection>) -> Result<()> {
    // emit a delete connection event
    emit!(ConnectionDeleted {
        connection: *ctx.accounts.connection.to_account_info().key,
        user: *ctx.accounts.user.to_account_info().key,
        from_profile: *ctx.accounts.from_profile.to_account_info().key,
        to_profile: *ctx.accounts.to_profile.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
