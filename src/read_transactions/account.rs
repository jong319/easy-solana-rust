use borsh::BorshDeserialize;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, program_pack::Pack};
use solana_client::rpc_client::RpcClient;
use spl_token::state::{
    Account as SplAssociatedTokenAccount,
    Mint as SplMintAccount,
};
use crate::{
    constants::solana_programs::system_program, 
    error::ReadTransactionError, 
    utils::{address_to_pubkey, addresses_to_pubkeys},
};

use super::metadata::MetadataAccount;

/// A generic struct for any account on Solana, mainly used when the account type is unknown.
///
/// ### Fields
/// 
/// - `pubkey`: The public key of the account.
/// - `sol_balance`: The sol balance in the account in ui format e.g 0.1 SOL
/// - `account_type`: The type of account with the relevant data deserialized.
/// - `data`: The data held within the account, custom programs can be borsh deserialized given that the user knows the struct of the data.
#[derive(Debug)]
pub struct Account {
    pub pubkey: String,
    pub sol_balance: f64,
    pub account_type: AccountType,
    pub data: Vec<u8>
}

/// Types of Solana accounts
/// - Wallet: Owned by a user. It can be used as a signer to interact with programs, including the System Program to transfer SOL to other accounts. 
/// 
/// - AssociatedToken: contains the token data belonging to a wallet account, such as token balance, token metadata and more. The wallet account owner has write permissions to transfer tokens and close the account. 
/// 
/// - Mint: Commonly known as the token address, it contains the overall token data such as token supply, decimals and the authority account of the token.
/// 
/// - Metadata: holds the metadata of a token, such as token names, token tickers, and their URIs. 
/// 
/// - Program: Accounts which are executable, meaning that wallet accounts can interact with these program accounts. 
#[derive(Debug)]
pub enum AccountType {
    Wallet,
    AssociatedToken(SplAssociatedTokenAccount),
    Mint(SplMintAccount),
    Metadata(MetadataAccount),
    Program,
    Others
}

/// Gets the account of any solana address.
/// 
/// # Arguments
/// 
/// * `client` - An instance of the RPC client used to communicate with the blockchain.
/// * `address` - address of any solana account
/// 
/// # Returns
/// 
/// `Result<Account, ReadTransactionError>` - Returns the `Account` 
/// struct on success, or an error if invalid address or non existent account
/// 
pub fn get_account(client: &RpcClient, address: &str) -> Result<Account, ReadTransactionError> {
    // Parse the public address into a Pubkey
    let pubkey = address_to_pubkey(address)?;

    // Fetch the account balance in lamports
    let account = client.get_account(&pubkey)?;
    let account_type: AccountType;
    if account.executable {
        account_type = AccountType::Program
    } else if account.owner == system_program() {
        account_type = AccountType::Wallet
    } else if SplMintAccount::unpack(&account.data).is_ok() {
        let mint_data = SplMintAccount::unpack(&account.data)
            .map_err(|_| ReadTransactionError::DeserializeError)?;
        account_type = AccountType::Mint(mint_data)
    } else if SplAssociatedTokenAccount::unpack(&account.data).is_ok() {
        let associated_token_data = SplAssociatedTokenAccount::unpack(&account.data)
            .map_err(|_| ReadTransactionError::DeserializeError)?;
        account_type = AccountType::AssociatedToken(associated_token_data)
    } else if MetadataAccount::deserialize(&mut account.data.as_ref()).is_ok() {
        let metadata = MetadataAccount::deserialize(&mut account.data.as_ref())
            .map_err(|_| ReadTransactionError::DeserializeError)?;
        account_type = AccountType::Metadata(metadata)
    } else {
        account_type = AccountType::Others
    }

    Ok(Account { 
        pubkey: address.to_string(),
        sol_balance: account.lamports as f64 / LAMPORTS_PER_SOL as f64,
        account_type,
        data: account.data
     })
}

pub fn get_multiple_accounts(client: &RpcClient, addresses: Vec<&str>) -> Result<Vec<Account>, ReadTransactionError> {
    let pubkeys = addresses_to_pubkeys(addresses);
    let accounts = client.get_multiple_accounts(&pubkeys)?;

    let mut result: Vec<Account> = vec![];
    
    // Iterate over accounts and corresponding pubkeys
    for (account_option, pubkey) in accounts.iter().zip(pubkeys) {
        match account_option {
            Some(account) => {
                // Determine the account type based on its data
                let account_type = if account.executable {
                    AccountType::Program
                } else if account.owner == system_program() {
                    AccountType::Wallet
                } else if let Ok(mint_data) = SplMintAccount::unpack(&account.data) {
                    AccountType::Mint(mint_data)
                } else if let Ok(associated_token_data) = SplAssociatedTokenAccount::unpack(&account.data) {
                    AccountType::AssociatedToken(associated_token_data)
                } else if let Ok(metadata) = MetadataAccount::deserialize(&mut account.data.as_ref()) {
                    AccountType::Metadata(metadata)
                } else {
                    AccountType::Others
                };

                // Add the successfully processed account to the result vector
                result.push(Account {
                    pubkey: pubkey.to_string(),
                    sol_balance: account.lamports as f64 / LAMPORTS_PER_SOL as f64,
                    account_type,
                    data: account.data.clone(),
                });
            }
            None => {
                // Handle the case where an account is `None` (nonexistent or invalid account)
                return Err(ReadTransactionError::AccountNotFound);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::utils::create_rpc_client;

    use super::*;

    const WALLET_ADDRESS_1: &str = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
    const ASSOCIATED_ACT_ACCOUNT_ADDRESS: &str = "7geCZYWHtghvWj11sb7exvu4uMANfhvGvEvVRRZ8GmSd";
    const ACT_MINT_ADDRESS: &str = "ArDKWeAhQj3LDSo2XcxTUb5j68ZzWg21Awq97fBppump";
    const PNUT_METADATA_ADDRESS: &str = "9dUa9SeDsikxXtCYtXTNviTUKdatFbj38xg8EhujpDsQ";
    
    #[test]
    fn test_get_account() {
        let client = create_rpc_client("RPC_URL");
        let account = get_account(&client, PNUT_METADATA_ADDRESS)
            .expect("Unable to get account");
        match account.account_type {
            AccountType::Metadata(_) => {
                assert!(true)
            }
            _ => {
                assert!(false)
            }
        }
    }

    #[test]
    fn test_get_multiple_accounts() {
        let client = create_rpc_client("RPC_URL");
        let addresses = vec![WALLET_ADDRESS_1, ASSOCIATED_ACT_ACCOUNT_ADDRESS, ACT_MINT_ADDRESS, PNUT_METADATA_ADDRESS];
        let accounts = get_multiple_accounts(&client, addresses)
            .expect("Unable to get accounts");
        let does_not_contain_unknown_account_type = accounts.iter().all(|account| {
            match account.account_type {
                AccountType::Others => {
                    false
                }
                _ => {
                    true
                }
            }
        });
        assert!(does_not_contain_unknown_account_type)
    }

}