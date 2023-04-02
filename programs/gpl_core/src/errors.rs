use anchor_lang::prelude::*;

#[error_code]
pub enum GumError {
    UnauthorizedSigner,
}

#[error_code]
pub enum PostError {
    URITooLong,
}

#[error_code]
pub enum ProfileMetadataError {
    URITooLong,
}

#[error_code]
pub enum ConnectionError {
    CannotConnectToSelf,
}
