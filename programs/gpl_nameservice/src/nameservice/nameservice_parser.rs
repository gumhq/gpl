use anchor_lang::prelude::*;
use std::str::FromStr;

pub trait NameServiceParser {
    type ServiceName;

    fn id_str() -> &'static str;

    fn id() -> Pubkey {
        // Unwrap because this is infallible
        Pubkey::from_str(Self::id_str()).unwrap()
    }
    fn unpack(record: &AccountInfo) -> Result<Self::ServiceName>;

    fn from_program_id(program_id: &Pubkey) -> Option<Self>
    where
        Self: Sized;

    fn validate(accounts: &[AccountInfo]) -> Result<bool>;

    fn validate_owner(record: &AccountInfo) -> Result<bool> {
        if record.owner != &Self::id() {
            return Err(ProgramError::IncorrectProgramId.into());
        }
        Ok(true)
    }
}
