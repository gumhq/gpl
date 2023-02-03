use crate::events::{CompressedConnectionDeleted, CompressedConnectionNew};
use crate::state::TreeConfig;
use crate::utils::{append_leaf, replace_leaf, try_find_asset_id, LeafSchema};
use anchor_lang::Discriminator;
use gpl_core::errors::ConnectionError;
use spl_account_compression::wrap_application_data_v1;
use spl_account_compression::Node;

use gpl_core::state::{Connection, Profile, User};

use anchor_lang::prelude::*;
use std::convert::AsRef;

use gpl_core::constants::*;

use anchor_lang::solana_program::keccak::hashv;
use gpl_core::program::GplCore;
use spl_account_compression::program::SplAccountCompression;
use spl_account_compression::Noop;

// Create Connection
#[derive(Accounts)]
pub struct CreateCompressedConnection<'info> {
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
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
        seeds::program = gpl_core_program.key(),
        bump,
    )]
    pub to_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = authority,
    )]
    pub user: Account<'info, User>,

    #[account(seeds = [merkle_tree.key.as_ref()], bump)]
    pub tree_config: Account<'info, TreeConfig>,

    #[account(mut)]
    /// CHECK The account must have the same authority as that of the config
    pub merkle_tree: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub compression_program: Program<'info, SplAccountCompression>,
    pub log_wrapper_program: Program<'info, Noop>,
    pub gpl_core_program: Program<'info, GplCore>,
    pub system_program: Program<'info, System>,
}

// Handler to create a new Connection account
pub fn create_compressed_connection_handler(
    ctx: Context<CreateCompressedConnection>,
) -> Result<()> {
    let from_profile = &ctx.accounts.from_profile;
    let to_profile = &ctx.accounts.to_profile;

    // CHECK that the from_profile and to_profile are not the same
    require_neq!(
        from_profile.key(),
        to_profile.key(),
        ConnectionError::CannotConnectToSelf
    );

    let connection_seeds = [
        CONNECTION_PREFIX_SEED.as_bytes(),
        from_profile.to_account_info().key.as_ref(),
        to_profile.to_account_info().key.as_ref(),
    ];

    let (connection_id, connection_bump) =
        Pubkey::try_find_program_address(&connection_seeds, &GplCore::id()).unwrap();

    let seed_hash = hashv(&connection_seeds).to_bytes();

    let asset_id = try_find_asset_id(ctx.accounts.merkle_tree.key, seed_hash)?;

    let connection = Connection {
        from_profile: *from_profile.to_account_info().key,
        to_profile: *to_profile.to_account_info().key,
    };

    let leaf = LeafSchema {
        asset_id,
        seed_hash,
        data_hash: hashv(&[&Connection::DISCRIMINATOR, &connection.try_to_vec()?]).to_bytes(),
    };

    let leaf_node = leaf.to_node()?;

    wrap_application_data_v1(leaf_node.to_vec(), &ctx.accounts.log_wrapper_program)?;

    append_leaf(
        ctx.accounts.merkle_tree.key,
        ctx.bumps["tree_config"],
        &ctx.accounts.authority.to_account_info(),
        leaf_node,
        &ctx.accounts.merkle_tree,
        &ctx.accounts.compression_program,
        &ctx.accounts.log_wrapper_program,
    )?;

    // emit a compressed connection event
    emit!(CompressedConnectionNew {
        connection_id,
        connection_bump,
        from_profile: *from_profile.to_account_info().key,
        to_profile: *to_profile.to_account_info().key,
        asset_id,
        user: *ctx.accounts.user.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
        index: 0 // TODO: Get the index from the tree
    });

    Ok(())
}

// Delete a Connection account
#[derive(Accounts)]
// Ideally this should be compacted down to asset_id, root, index
#[instruction(root: [u8;32], index: u32)]
pub struct DeleteCompressedConnection<'info> {
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.namespace.as_ref().as_bytes(),
            user.to_account_info().key.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
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
        seeds::program = gpl_core_program.key(),
        bump,
    )]
    pub to_profile: Account<'info, Profile>,
    #[account(
        seeds = [
            USER_PREFIX_SEED.as_bytes(),
            user.random_hash.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = authority,
    )]
    pub user: Account<'info, User>,

    #[account(seeds = [merkle_tree.key.as_ref()], bump)]
    pub tree_config: Account<'info, TreeConfig>,

    #[account(mut)]
    /// CHECK The account must have the same authority as that of the config
    pub merkle_tree: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub log_wrapper_program: Program<'info, Noop>,
    pub gpl_core_program: Program<'info, GplCore>,
    pub system_program: Program<'info, System>,
}

// Handler to delete a compressed Connection account
pub fn delete_compressed_connection_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, DeleteCompressedConnection<'info>>,
    root: [u8; 32],
    index: u32,
) -> Result<()> {
    let from_profile = &ctx.accounts.from_profile;
    let to_profile = &ctx.accounts.to_profile;

    // CHECK that the from_profile and to_profile are not the same
    require_neq!(
        from_profile.key(),
        to_profile.key(),
        ConnectionError::CannotConnectToSelf
    );

    let connection_seeds = [
        CONNECTION_PREFIX_SEED.as_bytes(),
        from_profile.to_account_info().key.as_ref(),
        to_profile.to_account_info().key.as_ref(),
    ];

    let (connection_id, connection_bump) =
        Pubkey::try_find_program_address(&connection_seeds, &GplCore::id()).unwrap();

    let seed_hash = hashv(&connection_seeds).to_bytes();

    let asset_id = try_find_asset_id(ctx.accounts.merkle_tree.key, seed_hash)?;

    let old_connection = Connection {
        from_profile: *from_profile.to_account_info().key,
        to_profile: *to_profile.to_account_info().key,
    };

    let old_leaf = LeafSchema {
        asset_id,
        seed_hash,
        data_hash: hashv(&[&Connection::DISCRIMINATOR, &old_connection.try_to_vec()?]).to_bytes(),
    };

    let old_leaf_node = old_leaf.to_node()?;

    let new_leaf_node = Node::default();

    wrap_application_data_v1(new_leaf_node.to_vec(), &ctx.accounts.log_wrapper_program)?;

    replace_leaf(
        ctx.accounts.merkle_tree.key,
        ctx.bumps["tree_config"],
        &ctx.accounts.authority.to_account_info(),
        &ctx.accounts.merkle_tree,
        root,
        old_leaf_node,
        new_leaf_node,
        index,
        ctx.remaining_accounts,
        &ctx.accounts.compression_program,
        &ctx.accounts.log_wrapper_program,
    )?;

    // emit a compressed connection event
    emit!(CompressedConnectionDeleted {
        connection_id,
        connection_bump,
        from_profile: *from_profile.to_account_info().key,
        to_profile: *to_profile.to_account_info().key,
        asset_id,
        user: *ctx.accounts.user.to_account_info().key,
        timestamp: Clock::get()?.unix_timestamp,
        index
    });

    Ok(())
}
