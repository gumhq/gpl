use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_lang::system_program;

#[cfg(feature = "no-entrypoint")]
pub use gpl_session_macros::*;

declare_id!("3ao63wcSRNa76bncC2M3KupNtXBFiDyNbgK52VG7dLaE");

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "gpl_session",
    project_url: "https://gum.fun",
    contacts: "email:hello@gum.fun,twitter:@gumisfunn",
    policy: "",
    preferred_languages: "en",
    source_code: "https://github.com/gumhq/gpl"
}

#[program]
pub mod gpl_session {
    use super::*;

    // create a session token
    pub fn create_session(
        ctx: Context<CreateSessionToken>,
        top_up: Option<bool>,
        valid_until: Option<i64>,
    ) -> Result<()> {
        // Set top up to false by default
        let top_up = top_up.unwrap_or(false);
        // Set valid until to 1 hour from now by default
        let valid_until = valid_until.unwrap_or(Clock::get()?.unix_timestamp + 60 * 60 * 1);
        create_session_token_handler(ctx, top_up, valid_until)
    }

    // revoke a session token
    pub fn revoke_session(ctx: Context<RevokeSessionToken>) -> Result<()> {
        revoke_session_token_handler(ctx)
    }
}

// Create a SessionToken account
#[derive(Accounts)]
pub struct CreateSessionToken<'info> {
    #[account(
        init,
        seeds = [
            SessionToken::SEED_PREFIX.as_bytes(),
            target_program.key().as_ref(),
            session_signer.key().as_ref(),
            authority.key().as_ref()
        ],
        bump,
        payer = authority,
        space = SessionToken::LEN
    )]
    pub session_token: Account<'info, SessionToken>,

    #[account(mut)]
    pub session_signer: Signer<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK the target program is actually a program.
    #[account(executable)]
    pub target_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

// Handler to create a session token account
pub fn create_session_token_handler(
    ctx: Context<CreateSessionToken>,
    top_up: bool,
    valid_until: i64,
) -> Result<()> {
    // Valid until can't be greater than a week
    require!(
        valid_until <= Clock::get()?.unix_timestamp + (60 * 60 * 24 * 7),
        SessionError::ValidityTooLong
    );

    let session_token = &mut ctx.accounts.session_token;
    session_token.set_inner(SessionToken {
        authority: ctx.accounts.authority.key(),
        target_program: ctx.accounts.target_program.key(),
        session_signer: ctx.accounts.session_signer.key(),
        valid_until,
    });

    // Top up the session signer account with some lamports to pay for the transaction fees from
    // the authority account.
    if top_up {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.authority.to_account_info(),
                    to: ctx.accounts.session_signer.to_account_info(),
                },
            ),
            LAMPORTS_PER_SOL.checked_div(100).unwrap(),
        )?;
    }

    Ok(())
}

// Revoke a session token
// We allow *anyone* to revoke a session token. This is because the session token is designed to
// expire on it's own after a certain amount of time. However, if the session token is compromised
// anyone can revoke it immediately.
//
// One attack vector here to consider, however is that a malicious actor could enumerate all the tokens
// created using the program and revoke them all or keep revoking them as they are created. It is a
// nuisance but not a security risk. We can easily address this by whitelisting a revoker.
#[derive(Accounts)]
pub struct RevokeSessionToken<'info> {
    #[account(
        mut,
        seeds = [
            SessionToken::SEED_PREFIX.as_bytes(),
            session_token.target_program.key().as_ref(),
            session_token.session_signer.key().as_ref(),
            session_token.authority.key().as_ref()
        ],
        bump,
        has_one = authority,
        close = authority,
    )]
    pub session_token: Account<'info, SessionToken>,

    #[account(mut)]
    // Only the token authority can reclaim the rent
    pub authority: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

// Handler to revoke a session token
pub fn revoke_session_token_handler(_: Context<RevokeSessionToken>) -> Result<()> {
    Ok(())
}

pub struct ValidityChecker<'info> {
    pub session_token: Account<'info, SessionToken>,
    pub session_signer: Signer<'info>,
    pub authority: Pubkey,
    pub target_program: Pubkey,
}

// SessionToken Account
#[account]
#[derive(Copy)]
pub struct SessionToken {
    pub authority: Pubkey,
    pub target_program: Pubkey,
    pub session_signer: Pubkey,
    pub valid_until: i64,
}

impl SessionToken {
    pub const LEN: usize = 8 + std::mem::size_of::<Self>();
    pub const SEED_PREFIX: &'static str = "session_token";

    fn is_expired(&self) -> Result<bool> {
        let now = Clock::get()?.unix_timestamp;
        Ok(now < self.valid_until)
    }

    // validate the token
    pub fn validate(&self, ctx: ValidityChecker) -> Result<bool> {
        let target_program = ctx.target_program;
        let session_signer = ctx.session_signer.key();
        let authority = ctx.authority.key();

        // Check the PDA seeds
        let seeds = &[
            SessionToken::SEED_PREFIX.as_bytes(),
            target_program.as_ref(),
            session_signer.as_ref(),
            authority.as_ref(),
        ];

        let (pda, _) = Pubkey::find_program_address(seeds, &crate::id());

        require_eq!(pda, ctx.session_token.key(), SessionError::InvalidToken);

        // Check if the token has expired
        self.is_expired()
    }
}

pub trait Session<'info> {
    fn session_token(&self) -> Option<Account<'info, SessionToken>>;
    fn session_signer(&self) -> Signer<'info>;
    fn session_authority(&self) -> Pubkey;
    fn target_program(&self) -> Pubkey;

    fn is_valid(&self) -> Result<bool> {
        let session_token = self.session_token().ok_or(SessionError::NoToken)?;
        let validity_ctx = ValidityChecker {
            session_token: session_token.clone(),
            session_signer: self.session_signer(),
            authority: self.session_authority(),
            target_program: self.target_program(),
        };
        // Check if the token is valid
        session_token.validate(validity_ctx)
    }
}

#[error_code]
pub enum SessionError {
    #[msg("Requested validity is too long")]
    ValidityTooLong,
    #[msg("Invalid session token")]
    InvalidToken,
    #[msg("No session token provided")]
    NoToken,
}
