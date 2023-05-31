use crate::nameservice::NameServiceParser;
use crate::NameRecord;
use anchor_lang::prelude::*;

pub struct GplNameService;
impl NameServiceParser for GplNameService {
    type ServiceName = NameRecord;

    fn id_str() -> &'static str {
        "5kWEYrdyryq3jGP5sUcKwTySzxr3dHzWFBVA3vkt6Nj5"
    }

    fn unpack(record: &AccountInfo) -> Result<NameRecord> {
        let name_record = NameRecord::try_deserialize(&mut &record.data.borrow_mut()[..])?;
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

        let name_record = Self::unpack(record)?;

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
