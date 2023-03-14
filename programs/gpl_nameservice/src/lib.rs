use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hashv;

declare_id!("7LEuQxAEegasvBSq7dDrMregc3mrDtTyHiytNK9pU68u");

#[program]
pub mod gpl_nameservice {
    use super::*;

    // create a new tld as name record
    pub fn create_tld(ctx: Context<CreateTLDAsNameRecord>, tld: String) -> Result<()> {
        create_tld_as_name_record_handler(ctx, tld)
    }

    // create a new name record
    pub fn create_name_record(ctx: Context<CreateNameRecord>, name: String) -> Result<()> {
        create_name_record_handler(ctx, name)
    }

    // transfer a name record
    pub fn transfer_name_record(ctx: Context<TransferNameRecord>) -> Result<()> {
        transfer_name_record_handler(ctx)
    }
}

// CreateTLDAsNameRecord
#[derive(Accounts)]
#[instruction(tld: String)]
pub struct CreateTLDAsNameRecord<'info> {
    #[account(
        init,
        seeds = [NameRecord::SEED_PREFIX.as_bytes(), &NameRecord::hash(&tld), Pubkey::default().as_ref()],
        bump,
        space = 8 + NameRecord::LEN,
        payer = authority,
    )]
    pub name_record: Account<'info, NameRecord>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// Handler to create a new tld as name record
pub fn create_tld_as_name_record_handler<'info>(
    ctx: Context<CreateTLDAsNameRecord<'info>>,
    tld: String,
) -> Result<()> {
    // TLD must be less than MAX_TLD_LENGTH
    require!(
        tld.len() <= NameRecord::MAX_TLD_LENGTH,
        NameServiceError::TldTooLong
    );

    let name_record = &mut ctx.accounts.name_record;
    name_record.set_inner(NameRecord {
        name: tld.clone(),
        authority: ctx.accounts.authority.key(),
        domain: Pubkey::default(),
    });
    Ok(())
}

// Create a new NameRecord
#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateNameRecord<'info> {
    #[account(
        init,
        seeds = [NameRecord::SEED_PREFIX.as_bytes(), &NameRecord::hash(&name), domain.key().as_ref()],
        bump,
        space = 8 + NameRecord::LEN,
        payer = authority,
    )]
    pub name_record: Account<'info, NameRecord>,

    #[account(
        // Temporarily disable subdomains
        constraint = domain.is_tld()
    )]
    pub domain: Account<'info, NameRecord>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// Handler to create a new NameRecord
pub fn create_name_record_handler(ctx: Context<CreateNameRecord>, name: String) -> Result<()> {
    // Name must be less than MAX_NAME_LENGTH
    require!(
        name.len() <= NameRecord::MAX_NAME_LENGTH,
        NameServiceError::NameTooLong
    );

    let name_record = &mut ctx.accounts.name_record;
    name_record.set_inner(NameRecord {
        name: name.clone(),
        authority: ctx.accounts.authority.key(),
        domain: ctx.accounts.domain.key(),
    });

    Ok(())
}

// Transfer a NameRecord
#[derive(Accounts)]
pub struct TransferNameRecord<'info> {
    #[account(
        mut,
        seeds = [
            NameRecord::SEED_PREFIX.as_bytes(),
            &NameRecord::hash(&name_record.name),
            name_record.domain.key().as_ref()
        ],
        bump,
        has_one = authority,
    )]
    pub name_record: Account<'info, NameRecord>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub new_authority: SystemAccount<'info>,
}

// Handler to transfer a NameRecord
pub fn transfer_name_record_handler(ctx: Context<TransferNameRecord>) -> Result<()> {
    let name_record = &mut ctx.accounts.name_record;
    name_record.authority = ctx.accounts.new_authority.key();
    Ok(())
}

// NameRecord Account
#[account]
pub struct NameRecord {
    pub name: String,
    pub authority: Pubkey,
    // Recurrence relation to the NameRecord
    // This allows us to create as many levels of subdomains as we want
    pub domain: Pubkey,
}

impl NameRecord {
    pub const MAX_NAME_LENGTH: usize = 16;

    pub const MAX_TLD_LENGTH: usize = 8;

    pub const LEN: usize = 8 + Self::MAX_NAME_LENGTH + std::mem::size_of::<Self>();

    pub const SEED_PREFIX: &'static str = "name_record";

    // keccak256 hash of name
    pub fn hash(name: &str) -> [u8; 32] {
        hashv(&[name.as_bytes()]).to_bytes()
    }

    // Check if the name is a TLD
    pub fn is_tld(&self) -> bool {
        self.domain == Pubkey::default()
    }
}

#[error_code]
pub enum NameServiceError {
    #[msg("Name is too long.")]
    NameTooLong,
    #[msg("TLD is too long.")]
    TldTooLong,
    #[msg("Name is already taken.")]
    NameTaken,
}
