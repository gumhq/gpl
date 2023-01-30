use anchor_lang::prelude::*;

#[allow(dead_code)]
pub enum GumError {
    PostError,
    ProfileError,
    UserError,
    ReactionError,
    ConnectionError,
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
