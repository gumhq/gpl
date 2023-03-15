use anchor_lang::Discriminator;

use crate::nameservice::NameServiceParser;
use crate::NameRecord;
use anchor_lang::prelude::*;

pub struct GplNameService;
impl NameServiceParser for GplNameService {
    type ServiceName = NameRecord;

    fn id_str() -> &'static str {
        "7LEuQxAEegasvBSq7dDrMregc3mrDtTyHiytNK9pU68u"
    }

    fn unpack(data: &[u8]) -> Result<NameRecord> {
        let record = NameRecord::try_from_slice(&data[8..])?;
        Ok(record)
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

        let record_data = &record.try_borrow_data()?;

        // The first 8 bytes should be same as the discriminator
        if record_data[0..8] != NameRecord::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData.into());
        }

        let name_record = Self::unpack(&record_data)?;

        // record.key should be the same as the PDA generated from NameRecord::SEED_PREFIX, hash of
        // the name, domain key and the bump seed

        let (expected_pda, _) = Pubkey::find_program_address(
            &[
                NameRecord::SEED_PREFIX.as_bytes(),
                &NameRecord::hash(&name_record.name),
                &name_record.domain.to_bytes(),
            ],
            &Self::id(),
        );

        if record.key != &expected_pda {
            return Err(ProgramError::InvalidSeeds.into());
        }

        // The authority should be the same as the one in the record
        if authority.key != &name_record.authority {
            return Err(ProgramError::InvalidAccountData.into());
        }

        Ok(true)
    }
}
