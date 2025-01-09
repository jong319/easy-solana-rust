
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
    pub fn token_2022_program() -> Pubkey {
        Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap()
    }
    pub fn associated_token_account_program() -> Pubkey {
        Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap()
    }
    pub fn rent_program() -> Pubkey {
        Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap()
    }
    pub fn sol_pubkey() -> Pubkey {
        Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap()
    }
}

pub mod raydium_accounts {
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    pub fn raydium_liquidity_pool_v4() -> Pubkey {
        Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap()
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
    pub fn buy_instruction_data() -> Vec<u8> {
        vec![
            0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea, // Instruction code
        ]
    }
    pub fn sell_instruction_data() -> Vec<u8> {
        vec![
            0x33, 0xe6, 0x85, 0xa4, 0x01, 0x7f, 0x83, 0xad,
        ]
    }
    pub const PUMP_TOKEN_DECIMALS: u32 = 6;
}
