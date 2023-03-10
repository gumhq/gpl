use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hashv;

mod nameservice;

pub use nameservice::*;

declare_id!("7LEuQxAEegasvBSq7dDrMregc3mrDtTyHiytNK9pU68u");

pub const MAX_NAME_LENGTH: usize = 16;
pub const MAX_TLD_LENGTH: usize = 8;

#[program]
pub mod gpl_nameservice {
    use super::*;

    // create a new name record
    pub fn create_name_record(ctx: Context<CreateNameRecord>, name: String) -> Result<()> {
        create_name_record_handler(ctx, name)
    }
}

// Create a new NameRecord
#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateNameRecord<'info> {
    #[account(
        init,
        seeds = [NameRecord::SEED_PREFIX.as_bytes(), &NameRecord::hash(&name), tld.key().as_ref()],
        bump,
        space = 8 + NameRecord::LEN,
        payer = authority,
    )]
    pub name_record: Account<'info, NameRecord>,

    pub tld: Account<'info, NameRecord>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// Handler to create a new NameRecord
pub fn create_name_record_handler(ctx: Context<CreateNameRecord>, name: String) -> Result<()> {
    // Name must be less than MAX_NAME_LENGTH
    require!(name.len() <= MAX_NAME_LENGTH, NameServiceError::NameTooLong);

    let name_record = &mut ctx.accounts.name_record;
    name_record.set_inner(NameRecord {
        name: name.clone(),
        authority: ctx.accounts.authority.key(),
        tld: ctx.accounts.tld.key(),
    });

    Ok(())
}

// NameRecord Account
#[account]
#[derive(Debug)]
pub struct NameRecord {
    pub name: String,
    pub authority: Pubkey,
    // Recurrence relation to the NameRecord
    // This allows us to create as many levels of subdomains as we want
    pub tld: Pubkey,
}

impl NameRecord {
    pub const LEN: usize = 8 + MAX_NAME_LENGTH + std::mem::size_of::<Self>();

    pub const SEED_PREFIX: &'static str = "name_record";

    // keccak256 hash of name
    pub fn hash(name: &str) -> [u8; 32] {
        hashv(&[name.as_bytes()]).to_bytes()
    }
}

#[error_code]
pub enum NameServiceError {
    #[msg("Name is too long.")]
    NameTooLong,
    #[msg("Name is already taken.")]
    NameTaken,
    #[msg("The PDA is not issued by a supported name service program")]
    InvalidNameService,
    InvalidOwner,
    InvalidDataLength,
    InvalidAuthority,
    InvalidNameRecord,
}
