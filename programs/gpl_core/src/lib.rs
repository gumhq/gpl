use anchor_lang::prelude::*;
use solana_security_txt::security_txt;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("CDDMdCAWB5AXgvEy7XJRggAu37QPG1b9aJXndZoPUkkm");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "gpl_core",
    project_url: "https://gum.fun",
    contacts: "email:hello@gum.fun,twitter:@gumisfunn",
    policy: "",
    preferred_languages: "en",
    source_code: "https://github.com/gumhq/gpl"
}

#[program]
pub mod gpl_core {

    use super::*;

    // Create a new user account
    pub fn create_user(ctx: Context<CreateUser>, random_hash: [u8; 32]) -> Result<()> {
        create_user_handler(ctx, random_hash)
    }

    // Update a user account with new authority
    pub fn update_user(ctx: Context<UpdateUser>) -> Result<()> {
        update_user_handler(ctx)
    }

    // Delete a user account
    pub fn delete_user(ctx: Context<DeleteUser>) -> Result<()> {
        delete_user_handler(ctx)
    }

    // Create a new profile account
    pub fn create_profile(
        ctx: Context<CreateProfile>,
        namespace: String,
        metadata_uri: String,
    ) -> Result<()> {
        create_profile_handler(ctx, namespace, metadata_uri)
    }

    // update a profile account
    pub fn update_profile(ctx: Context<UpdateProfile>, metadata_uri: String) -> Result<()> {
        update_profile_handler(ctx, metadata_uri)
    }

    // Delete a profile account
    pub fn delete_profile(ctx: Context<DeleteProfile>) -> Result<()> {
        delete_profile_handler(ctx)
    }

    // create a new profile_metadata account
    /// DEPRECATED
    pub fn create_profile_metadata(
        ctx: Context<CreateProfileMetadata>,
        metadata_uri: String,
    ) -> Result<()> {
        create_profile_metadata_handler(ctx, metadata_uri)
    }

    // update a profile_metadata
    /// DEPRECATED
    pub fn update_profile_metadata(
        ctx: Context<UpdateProfileMetadata>,
        metadata_uri: String,
    ) -> Result<()> {
        update_profile_metadata_handler(ctx, metadata_uri)
    }

    // delete a profile_metadata
    /// DEPRECATED
    pub fn delete_profile_metadata(ctx: Context<DeleteProfileMetadata>) -> Result<()> {
        delete_profile_metadata_handler(ctx)
    }

    // create a new post account
    pub fn create_post(
        ctx: Context<CreatePost>,
        metadata_uri: String,
        random_hash: [u8; 32],
    ) -> Result<()> {
        create_post_handler(ctx, metadata_uri, random_hash)
    }

    // update a post
    pub fn update_post(ctx: Context<UpdatePost>, metadata_uri: String) -> Result<()> {
        update_post_handler(ctx, metadata_uri)
    }

    // create a comment
    pub fn create_comment(
        ctx: Context<CreateComment>,
        metadata_uri: String,
        random_hash: [u8; 32],
    ) -> Result<()> {
        create_comment_handler(ctx, metadata_uri, random_hash)
    }

    // delete a post
    pub fn delete_post(ctx: Context<DeletePost>) -> Result<()> {
        delete_post_handler(ctx)
    }

    // create a connection account
    pub fn create_connection(ctx: Context<CreateConnection>) -> Result<()> {
        create_connection_handler(ctx)
    }

    // delete a connection account
    pub fn delete_connection(ctx: Context<DeleteConnection>) -> Result<()> {
        delete_connection_handler(ctx)
    }

    // create a reaction account with reaction type
    pub fn create_reaction(ctx: Context<CreateReaction>, reaction_type: String) -> Result<()> {
        create_reaction_handler(ctx, reaction_type)
    }

    // delete a reaction account
    pub fn delete_reaction(ctx: Context<DeleteReaction>) -> Result<()> {
        delete_reaction_handler(ctx)
    }
}
