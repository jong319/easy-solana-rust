use solana_sdk::{
    clock::Epoch,
    program_pack::Pack, 
    pubkey::Pubkey
};
use spl_token::state::{ 
    Account as AssociatedTokenAccount,
    AccountState
};

/// Convenience interpretation of a basic solana account, containing all the important variables and information.
/// Use the relevant schemas to decode data variable if needed.
#[derive(Debug)]
pub struct EasySolanaAccount {
    pub pubkey: Pubkey,
    /// balance is in SOL, not lamports
    pub sol_balance: f64,
    pub account_type: AccountType,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: Epoch,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum AccountType {
    /// A program account is executable
    Program,
    /// A wallet account is owned by the system program
    Wallet,
    /// An associated token account
    AssociatedTokenAccount,
    /// An mint account
    MintAccount,
    /// Unknown account type
    Others
}

/// Convenience interpretation of a Associated Token account 
#[derive(Debug)]
pub struct EasySolanaAssociatedTokenAccount {
    pub pubkey: Pubkey,
    /// balance is in SOL, not lamports
    pub sol_balance: f64, 
    /// The mint associated with this account
    pub mint_address: Pubkey,
    /// amount of tokens this account holds, decimals unknown
    pub token_balance: u64, 
    /// Wallet owner of the associated account
    pub owner: Pubkey, 
    /// The account's state
    pub state: AccountState,
    /// Optional authority to close the account.
    pub close_authority: Option<Pubkey>,
}

impl EasySolanaAccount {
    /// Converts an `EasySolanaAccount` to an `EasySolanaAssociatedTokenAccount` if 
    /// `AccountType::AssociatedTokenAccount` otherwise returns None
    pub fn get_associated_token_account(&self) -> Option<EasySolanaAssociatedTokenAccount> {
        if self.account_type == AccountType::AssociatedTokenAccount {
            let associated_token_account = AssociatedTokenAccount::unpack(&self.data).ok()?;
            Some(EasySolanaAssociatedTokenAccount {
                pubkey: self.pubkey,
                sol_balance: self.sol_balance,
                mint_address: associated_token_account.mint,
                token_balance: associated_token_account.amount,
                owner: associated_token_account.owner,
                state: associated_token_account.state,
                close_authority: associated_token_account.close_authority.into(),
            });
        };
        None
    }
}


/// Filters associated token accounts from a `Vec<EasySolanaAccount>` and returns Easy Solana's Associated Token Account struct.
/// Non associated token account types are filtered out and removed. 
pub fn get_associated_token_accounts(accounts: &Vec<EasySolanaAccount>) -> Vec<EasySolanaAssociatedTokenAccount> {
accounts
    .iter()
    .filter(|account| account.account_type == AccountType::AssociatedTokenAccount)
    .filter_map(|filtered_account| {
        // Attempt to unpack associated token account data
        if let Ok(associated_token_account) = AssociatedTokenAccount::unpack(&filtered_account.data) {
            Some(EasySolanaAssociatedTokenAccount {
                pubkey: filtered_account.pubkey,
                sol_balance: filtered_account.sol_balance,
                mint_address: associated_token_account.mint,
                token_balance: associated_token_account.amount,
                owner: associated_token_account.owner,
                state: associated_token_account.state,
                close_authority: associated_token_account.close_authority.into(),
            })
        } else {
            None
        }
    })
    .collect()
}