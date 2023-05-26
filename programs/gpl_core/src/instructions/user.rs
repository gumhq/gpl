use anchor_lang::prelude::*;

use crate::constants::*;
use crate::events::{UserAuthorityChanged, UserDeleted, UserNew};
use crate::state::User;

// Initialize a new user account
#[derive(Accounts)]
#[instruction(random_hash: [u8;32])]
pub struct CreateUser<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    // The account that will be initialized as a user
    #[account(
        init,
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            random_hash.as_ref(),
        ],
        bump,
        payer = payer,
        space = User::LEN
    )]
    pub user: Account<'info, User>,
    // The authority of the user
    pub authority: Signer<'info>,
    // The system program
    pub system_program: Program<'info, System>,
}

// Handler to create a new user account
pub fn create_user_handler(ctx: Context<CreateUser>, random_hash: [u8; 32]) -> Result<()> {
    let user = &mut ctx.accounts.user;
    user.random_hash = random_hash;
    user.authority = *ctx.accounts.authority.key;

    // emit new user event
    emit!(UserNew {
        user: *user.to_account_info().key,
        random_hash: random_hash,
        authority: *ctx.accounts.authority.key,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Update a user account with new authority
#[derive(Accounts)]
pub struct UpdateUser<'info> {
    // The user account to update
    #[account(
        mut,
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
    )]
    pub user: Account<'info, User>,

    // The new authority of the user
    pub new_authority: SystemAccount<'info>,
    // The authority of the user
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to update a user account with new authority
pub fn update_user_handler(ctx: Context<UpdateUser>) -> Result<()> {
    let user = &mut ctx.accounts.user;
    user.authority = *ctx.accounts.new_authority.key;
    // Emit user authority changed event
    emit!(UserAuthorityChanged {
        user: *user.to_account_info().key,
        old_authority: *ctx.accounts.authority.key,
        new_authority: *ctx.accounts.new_authority.key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

// Delete a user account
#[derive(Accounts)]
pub struct DeleteUser<'info> {
    // The user account to close
    #[account(
        mut,
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        bump,
        has_one = authority,
        close = authority
    )]
    pub user: Account<'info, User>,

    // The authority of the user
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to close a user account
pub fn delete_user_handler(ctx: Context<DeleteUser>) -> Result<()> {
    // Emit user deleted event
    emit!(UserDeleted {
        user: *ctx.accounts.user.to_account_info().key,
        authority: *ctx.accounts.authority.key,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}
