//! # Associated Token Account
//!
//! This module contains functions and structures for querying and 
//! deriving associated token accounts.

use solana_sdk::{native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::{ParsePubkeyError, Pubkey}};
use solana_client::{rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use spl_token::state::{
    Account as SplTokenAccount,
    Mint as SplMintAccount,
};
use solana_account_decoder::UiAccountData;
use serde_json::Value;

use crate::{
    constants::solana_programs::{associated_token_account_program, token_program}, error::ReadTransactionError, utils::{address_to_pubkey, addresses_to_pubkeys}
};


/// The associated token account is the account that holds the specific token 
/// balance and data belonging to the wallet address. All spl tokens belonging
/// to a wallet will have a relevant associated token account.
#[derive(Debug)]
pub struct AssociatedTokenAccount {
    pub pubkey: String, // pubkey of this account
    pub owner_pubkey: String, // pubkey of owner of this account
    pub mint_pubkey: String, // pubkey of token 
    pub mint_supply: u64, // current supply of token 
    pub mint_decimals: u8, // decimals of token 
    pub token_amount: u64, // amount of token this account holds
    pub token_ui_amount: f64, // ui amount of token this account holds
    pub mint_authority: Option<Pubkey> // mint authority of the token
}

/// Derives the associated token account address from the wallet address and mint address. 
/// 
/// # Arguments
/// 
/// * `wallet_address` - address of wallet holding the token.
/// * `mint_address` - address of the target token.
/// 
/// # Returns
/// 
/// `Result<String, ReadTransactionError>` - Returns a string address of the associated
/// token account on success, or an error if parsing the input addresses to pubkeys fails.
/// This function returns the address regardless if the account exists on the blockchain or not.
/// 
/// # Example
/// 
/// ```rust
/// use easy_solana::read_transactions::associated_token_account::derive_associated_token_account_address;
/// 
/// let wallet_address = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
/// let mint_address = "5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp";
/// let result = derive_associated_token_account_address(wallet_address, mint_address);
/// match result {
///     Ok(address) => println!("Associated Token Account Address: {:?}", address),
///     Err(err) => println!("Invalid wallet or mint address: {:?}", err)
/// }
/// ```
pub fn derive_associated_token_account_address(wallet_address: &str, mint_address: &str) -> Result<String, ParsePubkeyError> {
    let addresses = vec![wallet_address, mint_address];
    let pubkeys = addresses_to_pubkeys(addresses);
    // checks that pubkeys len == 2 else input wallet / mint address is invalid. 
    if pubkeys.len() != 2 {
        return Err(ParsePubkeyError::Invalid)
    }
    let (associated_token_account_pubkey, _nonce) = Pubkey::find_program_address(
        &[
            &pubkeys[0].to_bytes(),
            &token_program().to_bytes(),
            &pubkeys[1].to_bytes(),
        ],
        &associated_token_account_program(),
    );
    Ok(associated_token_account_pubkey.to_string())
}

/// Gets the associated token account details, including details of the token it holds. 
/// 
/// # Arguments
/// 
/// * `client` - An instance of the RPC client used to communicate with the blockchain.
/// * `associated_token_account_address` - address of the associated token account.
/// 
/// # Returns
/// 
/// `Result<AssociatedTokenAccount, ReadTransactionError>` - Returns the `AssociatedTokenAccount` 
/// struct on success, or an error if invalid address, non existent account or invalid account data.
/// 
/// # Example
/// 
/// ```rust
/// use easy_solana::{
///     read_transactions::associated_token_account::{
///         derive_associated_token_account_address,
///         get_associated_token_account
///     },
///     utils::create_rpc_client
/// };
/// 
/// let wallet_address = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
/// let mint_address = "5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp";
/// let result = derive_associated_token_account_address(wallet_address, mint_address);
/// match result {
///     Ok(address) => {
///         let client = create_rpc_client("https://api.mainnet-beta.solana.com");
///         let account = get_associated_token_account(&client, &address);
///         println!("associated token account: {:?}", account);
///     },
///     Err(err) => println!("Invalid wallet or mint address: {:?}", err)
/// }
/// ```
pub fn get_associated_token_account(client: &RpcClient, associated_token_account_address: &str) -> Result<AssociatedTokenAccount, ReadTransactionError> {
    // Parse the public address into a Pubkey
    let associated_token_account_pubkey = address_to_pubkey(associated_token_account_address)?;

    let token_account_data = client.get_account_data(&associated_token_account_pubkey)?;
    let token_account: SplTokenAccount = SplTokenAccount::unpack(&token_account_data)
        .map_err(|_| ReadTransactionError::DeserializeError)?;
    let mint_account_data = client.get_account_data(&token_account.mint)?;
    let mint_account: SplMintAccount = SplMintAccount::unpack(&mint_account_data)
        .map_err(|_| ReadTransactionError::DeserializeError)?;

    Ok(AssociatedTokenAccount {
        pubkey: associated_token_account_pubkey.to_string(),
        owner_pubkey: token_account.owner.to_string(),
        mint_pubkey: token_account.mint.to_string(),
        mint_supply: mint_account.supply,
        mint_decimals: mint_account.decimals,
        token_amount: token_account.amount,
        token_ui_amount: token_account.amount as f64 / u64::pow(10, mint_account.decimals as u32) as f64,
        mint_authority: mint_account.mint_authority.into()
    })
}

/// Gets multiple associated token accounts, invalid associated token accounts
/// are filtered out of the results.
/// 
/// # Arguments
/// 
/// * `client` - An instance of the RPC client used to communicate with the blockchain.
/// * `associated_token_account_addresses` - Vec of strings containing account addresses
/// 
/// # Returns
/// 
/// `Result<Vec<AssociatedTokenAccount>, ReadTransactionError>` - Returns a vector of `AssociatedTokenAccount` 
/// struct on success, or an error if invalid address, non existent account or invalid account data.
pub fn get_multiple_associated_token_accounts(
    client: &RpcClient,
    associated_token_addresses: Vec<&str>,
) -> Result<Vec<AssociatedTokenAccount>, ReadTransactionError> {
    // Convert the addresses to Pubkey
    let associated_token_pubkeys: Vec<Pubkey> = addresses_to_pubkeys(associated_token_addresses);
    if associated_token_pubkeys.len() == 0 {
        return Err(ReadTransactionError::InvalidAddress(ParsePubkeyError::Invalid))
    }
    // Fetch all account data in a single batch
    let associated_token_accounts = client.get_multiple_accounts(&associated_token_pubkeys)?;
    // Unpack token accounts and collect mint public keys
    let mut mint_pubkeys = Vec::new();
    let mut token_accounts = Vec::new();

    for (pubkey, account_data) in associated_token_pubkeys.iter().zip(associated_token_accounts.into_iter()) {
        if let Some(account_data) = account_data {
            if let Ok(token_account) = SplTokenAccount::unpack(&account_data.data) {
                token_accounts.push((pubkey, token_account));
                mint_pubkeys.push(token_account.mint);
            } else {
                eprintln!("get_multiple_associated_token_accounts: Unable to parse SplTokenAccount data")
            }
        } else {
            println!("get_multiple_associated_token_accounts: Account not found")
        }
    }
    
    // Fetch mint accounts in a single batch
    let mint_accounts_data = client.get_multiple_accounts(&mint_pubkeys)?;

    // Map mint public keys to their corresponding mint accounts
    let mint_accounts: Vec<SplMintAccount> = mint_accounts_data
    .into_iter()
    .filter_map(|account_data| {
        account_data.and_then(|data| {
            SplMintAccount::unpack(&data.data).ok()
        })
    })
    .collect();

    // Build associated token account details by matching token and mint accounts
    let mut associated_token_accounts = Vec::new();

    for ((pubkey, token_account), mint_account) in token_accounts.into_iter().zip(mint_accounts.into_iter()) {
        associated_token_accounts.push(AssociatedTokenAccount {
            pubkey: pubkey.to_string(),
            owner_pubkey: token_account.owner.to_string(),
            mint_pubkey: token_account.mint.to_string(),
            mint_supply: mint_account.supply,
            mint_decimals: mint_account.decimals,
            token_amount: token_account.amount,
            token_ui_amount: token_account.amount as f64
                / u64::pow(10, mint_account.decimals as u32) as f64,
            mint_authority: mint_account.mint_authority.into(),
        });
    }

    Ok(associated_token_accounts)
}


#[derive(Debug)]
pub struct WalletTokenAccount {
    pub pubkey: String,
    pub sol_balance: f64,
    pub mint_pubkey: String,
    pub owner_pubkey: String,
    pub token_amount: u64,
    pub decimals: u8,
    pub ui_amount: f64,
}


/// Gets all the associated token accounts belonging to a wallet address.
/// 
/// # Arguments
/// 
/// * `client` - An instance of the RPC client used to communicate with the blockchain.
/// * `wallet_address` - address of target wallet
/// 
/// # Returns
/// 
/// `Result<Vec<WalletTokenAccount>, ReadTransactionError>` - Returns a vector of `WalletTokenAccount` 
/// struct on success.
pub fn get_all_token_accounts(
    client: &RpcClient,
    wallet_address: &str,
) -> Result<Vec<WalletTokenAccount>, ReadTransactionError> {
    // Convert wallet address to Pubkey
    let wallet_pubkey = address_to_pubkey(wallet_address)?;

    // Fetch all token accounts owned by the wallet
    let token_accounts = client.get_token_accounts_by_owner(
        &wallet_pubkey,
        TokenAccountsFilter::ProgramId(token_program()),
    )?;

    let mut wallet_tokens = Vec::new();

    // Iterate over each token account
    for keyed_account in token_accounts.iter() {
        let pubkey = address_to_pubkey(keyed_account.pubkey.as_str())?;
        let sol_balance = keyed_account.account.lamports as f64 / LAMPORTS_PER_SOL as f64;
        if let UiAccountData::Json(parsed_data) = &keyed_account.account.data {
            // Extract `info` field
            let info = parsed_data
                .parsed
                .get("info")
                .ok_or_else(|| ReadTransactionError::DeserializeError)?
                .as_object()
                .ok_or_else(|| ReadTransactionError::DeserializeError)?;

            // Extract fields from `info`
            let mint_pubkey = info
                .get("mint")
                .and_then(Value::as_str)
                .ok_or_else(|| ReadTransactionError::DeserializeError)?
                .parse::<Pubkey>()?;

            let owner_pubkey = info
                .get("owner")
                .and_then(Value::as_str)
                .ok_or_else(|| ReadTransactionError::DeserializeError)?
                .parse::<Pubkey>()?;

            let token_amount = info
                .get("tokenAmount")
                .and_then(Value::as_object)
                .ok_or_else(|| ReadTransactionError::DeserializeError)?;

            let token_balance = token_amount
                .get("amount")
                .and_then(Value::as_str)
                .ok_or_else(|| ReadTransactionError::DeserializeError)?
                .parse::<u64>()
                .map_err(|_| ReadTransactionError::DeserializeError)?;

            let token_decimals = token_amount
                .get("decimals")
                .and_then(Value::as_u64)
                .unwrap_or(0) as u8;

            let ui_amount = token_amount
                .get("uiAmount")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);

            // Add to the list
            wallet_tokens.push(WalletTokenAccount {
                pubkey: pubkey.to_string(),
                sol_balance,
                mint_pubkey: mint_pubkey.to_string(),
                owner_pubkey: owner_pubkey.to_string(),
                token_amount: token_balance,
                decimals: token_decimals,
                ui_amount,
            });
        }
    }

    Ok(wallet_tokens)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::create_rpc_client;

    const ASSOCIATED_MICHI_WALLET_ADDRESS: &str = "7geCZYWHtghvWj11sb7exvu4uMANfhvGvEvVRRZ8GmSd";
    const MICHI_MINT_ADDRESS: &str = "ArDKWeAhQj3LDSo2XcxTUb5j68ZzWg21Awq97fBppump";
    const WALLET_ADDRESS: &str = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
    
    #[test]
    fn test_get_associated_token_account() {
        let client = create_rpc_client("RPC_URL");
        let associated_token_account = get_associated_token_account(
                &client,
                ASSOCIATED_MICHI_WALLET_ADDRESS
        ).expect("Failed to get associated token account");
        assert!(associated_token_account.mint_pubkey == MICHI_MINT_ADDRESS.to_string());
        assert!(associated_token_account.owner_pubkey == WALLET_ADDRESS.to_string());
        assert!(associated_token_account.mint_authority.is_none());
        println!("{:?}", associated_token_account.mint_supply / 1_000_000);
    }   

    #[test]
    fn test_derive_associated_token_account_address() {
        let associated_token_account_address = derive_associated_token_account_address(WALLET_ADDRESS, MICHI_MINT_ADDRESS).unwrap();
        assert!(associated_token_account_address == ASSOCIATED_MICHI_WALLET_ADDRESS.to_string())
    }

    #[test]
    fn test_get_all_token_accounts() {
        let client = create_rpc_client("RPC_URL");
        let token_accounts = get_all_token_accounts(&client, WALLET_ADDRESS).expect("Failed to retrieve token accounts");
        for token in token_accounts {
            assert!(token.owner_pubkey == WALLET_ADDRESS.to_string());
        }
    }

}