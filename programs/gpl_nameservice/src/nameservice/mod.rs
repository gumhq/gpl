use crate::nameservice::{
    ans::ANSNameService, gpl_nameservice::GplNameService, sns::SNSNameService,
};

use crate::NameServiceError;

use anchor_lang::prelude::*;

mod ans;
mod gpl_nameservice;
mod nameservice_parser;
mod sns;

pub use nameservice_parser::*;

enum NameService {
    GplNameService,
    SNSNameService,
    ANSNameService,
}

impl NameService {
    pub fn from_program_id(program_id: &Pubkey) -> Option<Self> {
        if program_id == &GplNameService::id() {
            Some(Self::GplNameService)
        } else if program_id == &SNSNameService::id() {
            Some(Self::SNSNameService)
        } else if program_id == &ANSNameService::id() {
            Some(Self::ANSNameService)
        } else {
            None
        }
    }

    pub fn validate(&self, accounts: &[AccountInfo]) -> Result<bool> {
        match self {
            Self::GplNameService => GplNameService::validate(accounts),
            Self::SNSNameService => SNSNameService::validate(accounts),
            Self::ANSNameService => ANSNameService::validate(accounts),
        }
    }
}

pub fn validate(accounts: &[AccountInfo]) -> Result<bool> {
    let record = &accounts[0];
    let name_service =
        NameService::from_program_id(&record.owner).ok_or(NameServiceError::InvalidNameService)?;
    name_service.validate(accounts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NameRecord;
    use anchor_lang::system_program::System;
    use anchor_lang::Discriminator;

    // test validate_record
    #[test]
    fn test_validate_gpl_namerecord() {
        let program_id = crate::id();
        let authority = Pubkey::new_unique();

        let gum_tld = NameRecord {
            name: "gum".to_string(),
            authority,
            domain: Pubkey::default(),
        };

        // Create the NameRecord PDA
        let (gum_tld_key, _bump_seed) = Pubkey::find_program_address(
            &[
                NameRecord::SEED_PREFIX.as_bytes(),
                &NameRecord::hash(&gum_tld.name),
                &gum_tld.domain.to_bytes(),
            ],
            &crate::id(),
        );

        let mut lamports = 10u64;

        let mut account_data: Vec<u8> = vec![];
        account_data.append(&mut NameRecord::DISCRIMINATOR.to_vec());
        account_data.append(&mut gum_tld.try_to_vec().unwrap());

        // Create the NameRecord account
        let gum_tld_account = AccountInfo::new(
            &gum_tld_key,
            false,
            false,
            &mut lamports,
            &mut account_data,
            &program_id,
            false,
            0,
        );

        let mut authority_lamports = 10u64;

        let mut authority_account_data: Vec<u8> = vec![];

        let system_program_id = System::id();

        let record_authority_account = AccountInfo::new(
            &gum_tld.authority,
            true,
            false,
            &mut authority_lamports,
            &mut authority_account_data,
            &system_program_id,
            false,
            0,
        );

        // Validate the NameRecord
        let result = validate(&[gum_tld_account, record_authority_account]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
