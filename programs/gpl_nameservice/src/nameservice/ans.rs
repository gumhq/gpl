use crate::nameservice::NameServiceParser;
use crate::NameServiceError;
use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ANSNameRecord {
    pub parent_name: Pubkey,

    // The owner of this name
    pub owner: Pubkey,

    // If `Pubkey::default()` the data is unspecified.
    pub class: Pubkey,

    pub expires_at: u64,
}

impl ANSNameRecord {
    pub const LEN: usize = 200;
}

pub struct ANSNameService;

impl NameServiceParser for ANSNameService {
    type ServiceName = ANSNameRecord;

    fn id_str() -> &'static str {
        "ALTNSZ46uaAUU7XUV6awvdorLGqAsPwa9shm7h4uP2FK"
    }

    fn unpack(record: &AccountInfo) -> Result<ANSNameRecord> {
        // discriminators are unique to programs, since we have validations below.
        // unless we hard code the discriminator value we will have to ignore them for now.
        let name_record = ANSNameRecord::try_from_slice(&mut &record.data.borrow_mut()[8..])?;
        Ok(name_record)
    }

    fn from_program_id(program_id: &Pubkey) -> Option<Self>
    where
        Self: Sized,
    {
        if program_id == &Self::id() {
            Some(Self)
        } else {
            None
        }
    }

    fn validate(accounts: &[AccountInfo]) -> Result<bool> {
        let accounts_iter = &mut accounts.iter();

        let record = next_account_info(accounts_iter)?;

        let authority = next_account_info(accounts_iter)?;

        // Validate the owner
        Self::validate_owner(record)?;

        // Data size may be longer than the fixed ans record name.
        if ANSNameRecord::LEN >= record.data_len() {
            return Err(NameServiceError::InvalidDataLength.into());
        }

        // Check if the owner of the account is ANS program.
        if record.owner.to_string() != Self::id_str() {
            return Err(ProgramError::InvalidAccountData.into());
        }

        let name_record = Self::unpack(record)?;

        // The authority should be the same as the owner in the record
        if authority.key != &name_record.owner {
            return Err(ProgramError::InvalidAccountData.into());
        }

        Ok(true)
    }
}
