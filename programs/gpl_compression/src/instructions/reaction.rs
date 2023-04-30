use anchor_lang::Discriminator;
use std::convert::AsRef;

use gpl_core::state::Profile;
use gpl_core::state::Reaction;
use gpl_core::state::ReactionType;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hashv;

use gpl_core::constants::*;

use gpl_core::program::GplCore;
use spl_account_compression::program::SplAccountCompression;
use spl_account_compression::wrap_application_data_v1;
use spl_account_compression::Node;
use spl_account_compression::Noop;

use crate::events::{CompressedReactionDeleted, CompressedReactionNew};
use crate::state::TreeConfig;
use crate::utils::verify_leaf;
use crate::utils::LeafSchema;
use crate::utils::{append_leaf, replace_leaf, try_find_asset_id};

// Create Reaction
#[derive(Accounts)]
#[instruction(to_post: Pubkey, reaction_type: String, post_root: [u8; 32], post_leaf: [u8; 32], post_index: u32)]
pub struct CreateCompressedReaction<'info> {
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.random_hash.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = authority,
    )]
    pub from_profile: Account<'info, Profile>,

    #[account(seeds = [merkle_tree.key.as_ref()], bump)]
    pub tree_config: Account<'info, TreeConfig>,

    #[account(mut)]
    /// CHECK The account must have the same authority as that of the config
    pub merkle_tree: UncheckedAccount<'info>,

    // TODO: The seeds should be more descriptive
    #[account(seeds = [target_merkle_tree.key.as_ref()], bump)]
    pub target_tree_config: Account<'info, TreeConfig>,

    /// CHECK The account must have the same authority as that of the config
    pub target_merkle_tree: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub compression_program: Program<'info, SplAccountCompression>,
    pub log_wrapper_program: Program<'info, Noop>,
    pub gpl_core_program: Program<'info, GplCore>,
    pub system_program: Program<'info, System>,
}

// Handler to create a compressed reaction
pub fn create_compressed_reaction_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CreateCompressedReaction<'info>>,
    to_post: Pubkey,
    reaction_type: ReactionType,
    post_root: [u8; 32],
    post_leaf: [u8; 32],
    post_index: u32,
) -> Result<()> {
    // Validate the reaction type
    reaction_type.validate()?;

    let from_profile = &ctx.accounts.from_profile;

    // Check if the to_post exists
    // FIXME:
    // They can potentially pass any proof. How do we verify this belongs to the asset unless we
    // construct the leaf ourselves?
    verify_leaf(
        ctx.accounts.target_merkle_tree.key,
        ctx.bumps["target_tree_config"],
        post_root,
        post_leaf,
        post_index,
        ctx.remaining_accounts,
        &ctx.accounts.target_merkle_tree,
        &ctx.accounts.compression_program,
    )?;

    let reaction_seeds = [
        REACTION_PREFIX_SEED.as_bytes(),
        reaction_type.as_ref(),
        to_post.as_ref(),
        from_profile.to_account_info().key.as_ref(),
    ];

    let (reaction_id, reaction_bump) =
        Pubkey::try_find_program_address(&reaction_seeds, &GplCore::id()).unwrap();

    let seed_hash = hashv(&reaction_seeds).to_bytes();

    let asset_id = try_find_asset_id(ctx.accounts.merkle_tree.key, seed_hash)?;

    let reaction = Reaction {
        from_profile: *from_profile.to_account_info().key,
        to_post,
        reaction_type: reaction_type.clone(),
    };

    let leaf = LeafSchema {
        asset_id,
        seed_hash,
        data_hash: hashv(&[&Reaction::DISCRIMINATOR, &reaction.try_to_vec()?]).to_bytes(),
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

    // emit a compressed reaction event
    emit!(CompressedReactionNew {
        from_profile: *from_profile.to_account_info().key,
        to_post,
        reaction_type: reaction_type.clone(),
        reaction_id,
        reaction_bump,
        asset_id,
        index: 0, //TODO: get the index
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

// Delete a compressed reaction
#[derive(Accounts)]
#[instruction(to_post: Pubkey, reaction_type: String, root: [u8; 32], index: u32)]
pub struct DeleteCompressedReaction<'info> {
    #[account(
        seeds = [
            PROFILE_PREFIX_SEED.as_bytes(),
            from_profile.random_hash.as_ref(),
        ],
        seeds::program = gpl_core_program.key(),
        bump,
        has_one = authority,
    )]
    pub from_profile: Account<'info, Profile>,

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

// Handler to delete a compressed reaction
pub fn delete_compressed_reaction_handler<'info>(
    ctx: Context<'_, '_, '_, 'info, DeleteCompressedReaction<'info>>,
    to_post: Pubkey,
    reaction_type: ReactionType,
    root: [u8; 32],
    index: u32,
) -> Result<()> {
    // Validate the reaction type
    reaction_type.validate()?;

    let from_profile = &ctx.accounts.from_profile;
    let reaction_seeds = [
        REACTION_PREFIX_SEED.as_bytes(),
        reaction_type.as_ref(),
        to_post.as_ref(),
        from_profile.to_account_info().key.as_ref(),
    ];

    let (reaction_id, reaction_bump) =
        Pubkey::try_find_program_address(&reaction_seeds, &GplCore::id()).unwrap();

    let seed_hash = hashv(&reaction_seeds).to_bytes();

    let asset_id = try_find_asset_id(ctx.accounts.merkle_tree.key, seed_hash)?;

    let old_reaction = Reaction {
        from_profile: *from_profile.to_account_info().key,
        to_post,
        reaction_type: reaction_type.clone(),
    };

    let old_leaf = LeafSchema {
        asset_id,
        seed_hash,
        data_hash: hashv(&[&Reaction::DISCRIMINATOR, &old_reaction.try_to_vec()?]).to_bytes(),
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

    // emit a compressed reaction delete event
    emit!(CompressedReactionDeleted {
        from_profile: *from_profile.to_account_info().key,
        to_post,
        reaction_type: reaction_type.clone(),
        reaction_id,
        reaction_bump,
        asset_id,
        index,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
