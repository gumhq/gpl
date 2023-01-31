use anchor_lang::prelude::*;

#[error_code]
pub enum GplCompressionError {
    #[msg("Invalid authority provided")]
    AssetIDNotFound,
}
