use crate::state::TreeConfig;
use anchor_lang::prelude::*;
use spl_account_compression::program::SplAccountCompression;
use spl_account_compression::Noop;

//Initialize TreeConfig
#[derive(Accounts)]
#[instruction(max_depth: u32, max_buffer_size: u32)]
pub struct InitializeTreeConfig<'info> {
    #[account(
        init,
        seeds = [merkle_tree.to_account_info().key.as_ref()],
        bump,
        payer = authority,
        space = TreeConfig::LEN
    )]
    pub tree_config: Account<'info, TreeConfig>,

    #[account(zero)]
    /// CHECK: This account must be all zeros
    pub merkle_tree: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub log_wrapper: Program<'info, Noop>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub system_program: Program<'info, System>,
}

// Handler for InitializeTreeConfig
pub fn initialize_tree_handler(
    ctx: Context<InitializeTreeConfig>,
    max_depth: u32,
    max_buffer_size: u32,
) -> Result<()> {
    let merkle_tree = ctx.accounts.merkle_tree.to_account_info();
    let seeds = &[merkle_tree.key.as_ref(), &[ctx.bumps["tree_config"]]];
    let tree_config = &mut ctx.accounts.tree_config;
    tree_config.set_inner(TreeConfig {
        authority: ctx.accounts.authority.key(),
        merkle_tree: merkle_tree.key(),
    });
    let authority_pda_signer = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.compression_program.to_account_info(),
        spl_account_compression::cpi::accounts::Initialize {
            authority: ctx.accounts.authority.to_account_info(),
            merkle_tree,
            noop: ctx.accounts.log_wrapper.to_account_info(),
        },
        authority_pda_signer,
    );
    spl_account_compression::cpi::init_empty_merkle_tree(cpi_ctx, max_depth, max_buffer_size)
}
