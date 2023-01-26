use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hashv;
use borsh::{BorshDeserialize, BorshSerialize};
use gpl_core::state::Post;

use spl_account_compression::Node;

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
#[repr(u8)]
// Add this to the event
pub enum AssetInstruction {
    PostCreate,
    PostUpdate,
    PostDelete,
    ReactionCreate,
    ReactionUpdate,
    ReactionDelete,
    ConnectionCreate,
    ConnectionUpdate,
    ConnectionDelete,
}

// Leaf Schema
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct LeafSchema {
    pub asset_id: Pubkey,
    pub seeds: Vec<u8>,
    pub data_hash: [u8; 32],
}

impl LeafSchema {
    pub fn to_node(&self) -> Result<Node> {
        let serialized = self.try_to_vec()?;
        let node = hashv(&[&serialized]).to_bytes();
        Ok(node)
    }
}

// Derive asset_id
pub fn derive_asset_id(merkle_tree: &Pubkey, seeds: &[u8]) -> Pubkey {
    let asset_seeds = [b"asset".as_ref(), merkle_tree.as_ref(), seeds];
    msg!("asset_seeds: {:?}", asset_seeds);
    // Pubkey::find_program_address(&asset_seeds, &crate::ID).0
    Pubkey::new_unique()
}

pub fn replace_leaf<'info>(
    merkle_tree: &Pubkey,
    bump: u8,
    compression_program: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    merkle_tree_account: &AccountInfo<'info>,
    log_wrapper: &AccountInfo<'info>,
    remaining_accounts: &[AccountInfo<'info>],
    root_node: Node,
    previous_leaf: Node,
    new_leaf: Node,
    index: u32,
) -> Result<()> {
    let seeds = &[merkle_tree.as_ref(), &[bump]];
    let authority_pda_signer = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(
        compression_program.clone(),
        spl_account_compression::cpi::accounts::Modify {
            authority: authority.clone(),
            merkle_tree: merkle_tree_account.clone(),
            noop: log_wrapper.clone(),
        },
        authority_pda_signer,
    )
    .with_remaining_accounts(remaining_accounts.to_vec());
    spl_account_compression::cpi::replace_leaf(cpi_ctx, root_node, previous_leaf, new_leaf, index)
}

pub fn append_leaf<'info>(
    merkle_tree: &Pubkey,
    bump: u8,
    compression_program: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    merkle_tree_account: &AccountInfo<'info>,
    log_wrapper: &AccountInfo<'info>,
    leaf_node: Node,
) -> Result<()> {
    let seeds = &[merkle_tree.as_ref(), &[bump]];
    let authority_pda_signer = &[&seeds[..]];
    let cpi_ctx = CpiContext::new_with_signer(
        compression_program.clone(),
        spl_account_compression::cpi::accounts::Modify {
            authority: authority.clone(),
            merkle_tree: merkle_tree_account.clone(),
            noop: log_wrapper.clone(),
        },
        authority_pda_signer,
    );
    spl_account_compression::cpi::append(cpi_ctx, leaf_node)
}

// append_leaf -> Instruction { Context with Args, Accounts everything required to identify the asset
// off-chain }

// replace_leaf -> Instruction { Context with Args, Accounts everything required to identify the asset
// off-chain }
