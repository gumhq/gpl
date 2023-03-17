use crate::nameservice::NameServiceParser;
use crate::NameServiceError;
use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SNSNameRecord {
    // Names are hierarchical.  `parent_name` contains the account address of the parent
    // name, or `Pubkey::default()` if no parent exists.
    pub parent_name: Pubkey,

    // The owner of this name
    pub owner: Pubkey,
    // The class of data this account represents (DNS record, twitter handle, SPL Token name/symbol, etc)
    //
    // If `Pubkey::default()` the data is unspecified.
    pub class: Pubkey,
}

impl SNSNameRecord {
    pub const LEN: usize = 96;
}

pub struct SNSNameService;

impl NameServiceParser for SNSNameService {
    type ServiceName = SNSNameRecord;

    fn id_str() -> &'static str {
        "namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX"
    }

    fn unpack(record: &AccountInfo) -> Result<SNSNameRecord> {
        // Check disciminator
        let name_record = SNSNameRecord::try_from_slice(&mut &record.data.borrow_mut()[..])?;
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

        if record.data_len() != SNSNameRecord::LEN {
            return Err(NameServiceError::InvalidDataLength.into());
        }

        let name_record = Self::unpack(record)?;

        // The authority should be the same as the owner in the record
        if authority.key != &name_record.owner {
            return Err(ProgramError::InvalidAccountData.into());
        }

        Ok(true)
    }
}
