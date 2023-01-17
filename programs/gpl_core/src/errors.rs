use anchor_lang::prelude::*;

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
pub enum ConnectionError {
    CannotConnectToSelf,
}
