use solana_sdk::pubkey::Pubkey;
use crate::MetadataAccount;
use super::account::EasySolanaAccount;


/// Convenience interpretation of a Mint account 
#[derive(Debug)]
pub struct MintAccount {
    pub base: EasySolanaAccount,
    /// Contains the metadata of the token, None until queried.
    pub metadata: MetadataAccount,
    /// Total supply of tokens.
    pub supply: u64,
    /// Number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
    /// Is `true` if this structure has been initialized
    pub is_initialized: bool,
    /// Optional authority to freeze token accounts.
    pub freeze_authority: Option<Pubkey>,
    // Optional authority to mint tokens
    pub mint_authority: Option<Pubkey>,
}
