use solana_sdk::{
    clock::Epoch,
    program_pack::Pack, 
    pubkey::Pubkey
};
use spl_token::state::{ 
    Account as AssociatedTokenAccount,
    AccountState,
    Mint
};

use crate::{
    reader::account::{
        AccountReader, 
        MetadataAccount
    },
    error::AccountReaderError
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
    /// Pubkey of associated token account itself
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

/// Convenience interpretation of a Mint account 
#[derive(Debug)]
pub struct EasySolanaMintAccount {
    // Mint address of token
    pub mint: Pubkey,
    pub token_name: String,
    pub token_ticker: String,
    pub token_uri: String,
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


impl EasySolanaAccount {
    /// Converts an `EasySolanaAccount` to an `EasySolanaAssociatedTokenAccount` if 
    /// account_type is `AccountType::AssociatedTokenAccount` otherwise returns None
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
pub fn filter_associated_token_accounts(accounts: &Vec<EasySolanaAccount>) -> Vec<EasySolanaAssociatedTokenAccount> {
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


/// Filters mint accounts from a `Vec<EasySolanaAccount>` and returns Easy Solana's Mint Account struct.
/// Non mint account types are filtered out and removed. 
pub fn filter_mint_accounts(account_reader: AccountReader, accounts: &Vec<EasySolanaAccount>) -> Result<Vec<EasySolanaMintAccount>, AccountReaderError> {
    // filter for Mint account type only
    let filtered_accounts: Vec<&EasySolanaAccount> = accounts
        .iter()
        .filter(|account| account.account_type == AccountType::MintAccount)
        .collect();

    // Get pubkeys of mint account types
    let token_pubkeys: Vec<Pubkey> = filtered_accounts
        .iter()
        .map(|account| account.pubkey)
        .collect();

    // Get metadata accounts of mint account types
    let metadata_accounts: Vec<MetadataAccount> = account_reader
        .get_metadata_of_tokens(&token_pubkeys)
        .map_err(|err| err )?
        .into_iter()
        .collect();

    // Unpack mint account types as Mint, create `EasySolanaMintAccount` struct with metadata 
    let result = filtered_accounts
        .into_iter()
        .filter_map(|account| {
            if let Ok(mint_account) = Mint::unpack(&account.data) {
                let matching_metadata_accounts = metadata_accounts
                .iter()
                .find(|metadata_account| metadata_account.mint == account.pubkey);

                matching_metadata_accounts.map(|metadata| EasySolanaMintAccount {
                    mint: metadata.mint,
                    token_name: metadata.data.name.clone(),
                    token_ticker: metadata.data.symbol.clone(),
                    token_uri: metadata.data.uri.clone(),
                    supply: mint_account.supply,
                    decimals: mint_account.decimals,
                    is_initialized: mint_account.is_initialized,
                    freeze_authority: mint_account.freeze_authority.into(),
                    mint_authority: mint_account.mint_authority.into()
                })  
            } else {
                None
            }
        })
        .collect();
    Ok(result)
}



#[cfg(test)]
mod tests {
    use crate::{
        addresses_to_pubkeys, create_rpc_client, filter_associated_token_accounts, filter_mint_accounts,
    };
    use super::*;

    const WALLET_ADDRESS: &str = "joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg";
    const WALLET_ASSOCIATED_HAPPY_CAT_ADDRESS: &str = "4ZVBVjcaLUqUxVi3EHaVKp1pZ96AZoznyGWgWxKYZhsD";
    const HAPPY_CAT_MINT_ADDRESS: &str = "2Df5GphWffyegX9Xjwhjz96hozATStb8YBaouaYcpump";

    #[test]
    fn test_filter_associated_token_accounts() {
        let addresses: Vec<String> = 
        vec![
            WALLET_ADDRESS,
            WALLET_ASSOCIATED_HAPPY_CAT_ADDRESS,
            HAPPY_CAT_MINT_ADDRESS,
        ].iter_mut()
        .map(|address| address.to_string())
        .collect();

        let client = create_rpc_client("RPC_URL");
        // Convert String addresses to Pubkeys
        let pubkeys = addresses_to_pubkeys(addresses);
        let account_reader = AccountReader::new(client);
        let easy_solana_accounts = account_reader.get_easy_solana_accounts(&pubkeys).expect("Failed to fetch accounts");
        let associated_token_accounts = filter_associated_token_accounts(&easy_solana_accounts);
        // Only 1 associated token account
        assert!(associated_token_accounts.len() == 1);
        let associated_token_account = &associated_token_accounts[0];
        // Owner of the account should be wallet address
        assert!(associated_token_account.owner.to_string() == WALLET_ADDRESS);
        // Mint of the account should be Happy Cat Mint Address
        assert!(associated_token_account.mint_address.to_string() == HAPPY_CAT_MINT_ADDRESS);
        assert!(associated_token_account.token_balance == 869439);
        assert!(associated_token_account.sol_balance == 0.00203928);
    }

    #[test]
    fn test_filter_mint_accounts() {
        let addresses: Vec<String> = 
        vec![
            WALLET_ADDRESS,
            WALLET_ASSOCIATED_HAPPY_CAT_ADDRESS,
            HAPPY_CAT_MINT_ADDRESS,
        ].iter_mut()
        .map(|address| address.to_string())
        .collect();

        let client = create_rpc_client("RPC_URL");
        // Convert String addresses to Pubkeys
        let pubkeys = addresses_to_pubkeys(addresses);
        let account_reader = AccountReader::new(client);
        let easy_solana_accounts = account_reader.get_easy_solana_accounts(&pubkeys).expect("Failed to fetch easy solana accounts");
        let mint_accounts = filter_mint_accounts(account_reader,&easy_solana_accounts).expect("Failed to fetch mint accounts");
        // Only 1 mint account
        assert!(mint_accounts.len() == 1);
        let happy_cat_account = &mint_accounts[0];
        // Decimals of happy cat token should be 6
        assert!(happy_cat_account.decimals == 6);
        assert!(happy_cat_account.token_name == "HAPPY".to_string());
        assert!(happy_cat_account.token_ticker == "Happy Cat".to_string());
    }
}