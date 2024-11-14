
// Solana default program accounts
pub mod solana_programs {
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;
    
    pub fn metadata_program() -> Pubkey {
        Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap()
    }
    pub fn system_program() -> Pubkey {
        Pubkey::from_str("11111111111111111111111111111111").unwrap()
    }
    pub fn token_program() -> Pubkey {
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap()
    }
    pub fn associated_token_account_program() -> Pubkey {
        Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap()
    }
    pub fn rent_program() -> Pubkey {
        Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap()
    }
}

// Pumpfun program accounts
pub mod pumpfun_accounts {
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;
    
    pub fn pumpfun_program() -> Pubkey {
        Pubkey::from_str("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P").unwrap()
    }
    pub fn pumpfun_token_mint_authority_program() -> Pubkey {
        Pubkey::from_str("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM").unwrap()
    }
    pub fn pumpfun_fee_account() -> Pubkey {
        Pubkey::from_str("CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM").unwrap()
    }
    pub fn pumpfun_global_account() -> Pubkey {
        Pubkey::from_str("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf").unwrap()
    }
    pub fn pumpfun_event_authority_account() -> Pubkey {
        Pubkey::from_str("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1").unwrap()
    }
}
