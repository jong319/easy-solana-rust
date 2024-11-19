use solana_sdk::pubkey::Pubkey;
use spl_token::state::AccountState;
use super::account::EasySolanaAccount;


/// Convenience interpretation of a Associated Token account 
#[derive(Debug)]
pub struct AssociatedTokenAccount {
    pub base: EasySolanaAccount,
    /// The mint associated with this account
    pub mint_pubkey: Pubkey,
    /// amount of tokens this account holds, decimals unknown
    pub token_balance: u64, 
    /// Wallet owner of the associated account
    pub owner_pubkey: Pubkey, 
    /// The account's state
    pub state: AccountState,
    /// Optional authority to close the account.
    pub close_authority: Option<Pubkey>,
}