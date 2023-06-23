use crate::constants::*;
use crate::errors::GumError;
use crate::state::MAX_LEN_URI;
use crate::state::{Badge, Issuer, Profile, Schema};
use std::str::FromStr;

use anchor_lang::prelude::*;

// Create a badge

#[derive(Accounts)]
#[instruction(metadata_uri: String)]
pub struct CreateBadge<'info> {
    #[account(
        init,
        seeds = [Badge::SEED_PREFIX.as_bytes(), issuer.key().as_ref(), schema.key().as_ref(), holder.key().as_ref()],
        bump,
        payer = authority,
        space = Badge::LEN
    )]
    pub badge: Account<'info, Badge>,
    #[account(
        seeds = [Issuer::SEED_PREFIX.as_bytes(), authority.key().as_ref()],
        bump,
        has_one = authority,
        constraint = issuer.verified @ GumError::UnverifiedIssuer
    )]
    pub issuer: Account<'info, Issuer>,

    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            holder.random_hash.as_ref(),
        ],
        bump,
    )]
    pub holder: Account<'info, Profile>,

    #[account(
        seeds = [Schema::SEED_PREFIX.as_bytes(), schema.random_hash.as_ref()],
        bump,
    )]
    pub schema: Account<'info, Schema>,

    /// CHECK the update_authority of the badge issuer
    pub update_authority: Option<UncheckedAccount<'info>>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to create a badge
pub fn create_badge_handler(ctx: Context<CreateBadge>, metadata_uri: String) -> Result<()> {
    require!(metadata_uri.len() <= MAX_LEN_URI, GumError::URITooLong);

    let badge = &mut ctx.accounts.badge;

    badge.set_inner(Badge {
        issuer: ctx.accounts.issuer.key(),
        holder: ctx.accounts.holder.key(),
        schema: ctx.accounts.schema.key(),
        metadata_uri,
        // If an update_authority is not provided, use the issuer ie current signer as the update_authority
        update_authority: ctx
            .accounts
            .update_authority
            .clone()
            .map(|account| account.key())
            .unwrap_or(ctx.accounts.authority.key()),
    });

    Ok(())
}

// Update a badge
#[derive(Accounts)]
#[instruction(metadata_uri: String)]
pub struct UpdateBadge<'info> {
    #[account(
        mut,
        seeds = [Badge::SEED_PREFIX.as_bytes(), issuer.key().as_ref(), schema.key().as_ref(), badge.holder.key().as_ref()],
        bump,
        has_one = issuer,
        has_one = schema
    )]
    pub badge: Account<'info, Badge>,
    #[account(
        seeds = [Issuer::SEED_PREFIX.as_bytes(), issuer.authority.key().as_ref()],
        bump,
        constraint = issuer.verified @ GumError::UnverifiedIssuer
    )]
    pub issuer: Account<'info, Issuer>,

    #[account(
        seeds = [Schema::SEED_PREFIX.as_bytes(), schema.random_hash.as_ref()],
        bump,
    )]
    pub schema: Account<'info, Schema>,

    #[account(
        // The badge can be updated by the issuer or update authority set in the badge
        constraint = badge.update_authority == signer.key() || issuer.authority == signer.key() @ProgramError::MissingRequiredSignature
    )]
    pub signer: Signer<'info>,
}

// Handler to update a badge
pub fn update_badge_handler(ctx: Context<UpdateBadge>, metadata_uri: String) -> Result<()> {
    require!(metadata_uri.len() <= MAX_LEN_URI, GumError::URITooLong);

    let badge = &mut ctx.accounts.badge;

    badge.metadata_uri = metadata_uri;

    Ok(())
}

// Burn a badge
// Either the holder or the issuer can burn a badge
#[derive(Accounts)]
pub struct BurnBadge<'info> {
    #[account(
        mut,
        seeds = [Badge::SEED_PREFIX.as_bytes(), issuer.key().as_ref(), schema.key().as_ref(), holder.key().as_ref()],
        bump,
        has_one = issuer,
        has_one = holder,
        has_one = schema,
        close = signer
    )]
    pub badge: Account<'info, Badge>,

    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            holder.random_hash.as_ref(),
        ],
        bump,
    )]
    pub holder: Account<'info, Profile>,

    #[account(
        seeds = [Issuer::SEED_PREFIX.as_bytes(), issuer.authority.key().as_ref()],
        bump,
        constraint = issuer.verified @ GumError::UnverifiedIssuer
    )]
    pub issuer: Account<'info, Issuer>,

    #[account(
        seeds = [Schema::SEED_PREFIX.as_bytes(), schema.random_hash.as_ref()],
        bump,
    )]
    pub schema: Account<'info, Schema>,

    #[account(
        mut,
        constraint = signer.key() == holder.authority.key() || signer.key() == issuer.key() @ProgramError::MissingRequiredSignature
    )]
    pub signer: Signer<'info>,
}

// Handler to burn a badge
pub fn burn_badge_handler(_: Context<BurnBadge>) -> Result<()> {
    Ok(())
}

// Create a schema
#[derive(Accounts)]
#[instruction(metadata_uri: String, random_hash: [u8; 32])]
pub struct CreateSchema<'info> {
    #[account(
        init,
        seeds = [Schema::SEED_PREFIX.as_bytes(), random_hash.as_ref()],
        bump,
        payer = authority,
        space = Schema::LEN
    )]
    pub schema: Account<'info, Schema>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to create a schema
pub fn create_schema_handler(
    ctx: Context<CreateSchema>,
    metadata_uri: String,
    random_hash: [u8; 32],
) -> Result<()> {
    require_eq!(
        ctx.accounts.authority.key(),
        // FIXME: Move this to constant byte and importantly use a different key before deploying.
        // This is a quick hack to test the patch.
        Pubkey::from_str("Bi2ZL1UijCXwtNYi132NyMDRnzVxpuAVcgsqVuUgee5A").unwrap(),
        GumError::UnauthorizedSigner
    );
    require!(metadata_uri.len() <= MAX_LEN_URI, GumError::URITooLong);

    let schema = &mut ctx.accounts.schema;

    schema.set_inner(Schema {
        authority: ctx.accounts.authority.key(),
        metadata_uri,
        random_hash,
    });

    Ok(())
}

// Update a schema
#[derive(Accounts)]
#[instruction(metadata_uri: String)]
pub struct UpdateSchema<'info> {
    #[account(
        mut,
        seeds = [Schema::SEED_PREFIX.as_bytes(), schema.random_hash.as_ref()],
        bump,
        has_one = authority
    )]
    pub schema: Account<'info, Schema>,

    pub authority: Signer<'info>,
}

// Handler to update a schema
pub fn update_schema_handler(ctx: Context<UpdateSchema>, metadata_uri: String) -> Result<()> {
    require_eq!(
        ctx.accounts.authority.key(),
        // FIXME: Move this to constant byte and importantly use a different key before deploying.
        // This is a quick hack to test the patch.
        Pubkey::from_str("Bi2ZL1UijCXwtNYi132NyMDRnzVxpuAVcgsqVuUgee5A").unwrap(),
        GumError::UnauthorizedSigner
    );
    require!(metadata_uri.len() <= MAX_LEN_URI, GumError::URITooLong);

    let schema = &mut ctx.accounts.schema;

    schema.metadata_uri = metadata_uri;

    Ok(())
}

// Delete a schema
#[derive(Accounts)]
pub struct DeleteSchema<'info> {
    #[account(
        mut,
        seeds = [Schema::SEED_PREFIX.as_bytes(), schema.random_hash.as_ref()],
        bump,
        has_one = authority,
        close = authority
    )]
    pub schema: Account<'info, Schema>,

    pub authority: Signer<'info>,
}

// Handler to delete a schema
pub fn delete_schema_handler(ctx: Context<DeleteSchema>) -> Result<()> {
    require_eq!(
        ctx.accounts.authority.key(),
        // FIXME: Move this to constant byte and importantly use a different key before deploying.
        // This is a quick hack to test the patch.
        Pubkey::from_str("Bi2ZL1UijCXwtNYi132NyMDRnzVxpuAVcgsqVuUgee5A").unwrap(),
        GumError::UnauthorizedSigner
    );
    Ok(())
}

// Create an issuer
// TODO: Think about if the issuer should have a profile themselves
#[derive(Accounts)]
pub struct CreateIssuer<'info> {
    #[account(
        init,
        seeds = [Issuer::SEED_PREFIX.as_bytes(), authority.key().as_ref()],
        bump,
        payer = authority,
        space = Issuer::LEN
    )]
    pub issuer: Account<'info, Issuer>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Handler to create an issuer
pub fn create_issuer_handler(ctx: Context<CreateIssuer>) -> Result<()> {
    let issuer = &mut ctx.accounts.issuer;

    issuer.set_inner(Issuer {
        authority: ctx.accounts.authority.key(),
        verified: false,
    });

    Ok(())
}

// Delete an issuer
#[derive(Accounts)]
pub struct DeleteIssuer<'info> {
    #[account(
        mut,
        seeds = [Issuer::SEED_PREFIX.as_bytes(), authority.key().as_ref()],
        bump,
        has_one = authority,
        close = authority
    )]
    pub issuer: Account<'info, Issuer>,
    pub authority: Signer<'info>,
}
// Handler to delete an issuer
pub fn delete_issuer_handler(_: Context<DeleteIssuer>) -> Result<()> {
    Ok(())
}

// Verify an issuer
#[derive(Accounts)]
pub struct VerifyIssuer<'info> {
    #[account(mut)]
    pub issuer: Account<'info, Issuer>,

    pub signer: Signer<'info>,
}

// Handler to verify an issuer
pub fn verify_issuer_handler(ctx: Context<VerifyIssuer>) -> Result<()> {
    require_eq!(
        ctx.accounts.signer.key(),
        // FIXME: Move this to constant byte and importantly use a different key before deploying.
        // This is a quick hack to test the patch.
        Pubkey::from_str("Bi2ZL1UijCXwtNYi132NyMDRnzVxpuAVcgsqVuUgee5A").unwrap(),
        GumError::InvalidSignerToVerify
    );

    let issuer = &mut ctx.accounts.issuer;

    issuer.verified = true;

    Ok(())
}
