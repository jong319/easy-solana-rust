//! # Associated Token Account
//!
//! This module contains functions and structures for querying and 
//! deriving associated token accounts.

use solana_sdk::{program_pack::Pack, pubkey::{ParsePubkeyError, Pubkey}};
use solana_client::{rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use spl_token::state::{
    Account as SplTokenAccount,
    Mint as SplMintAccount,
};
use solana_account_decoder::UiAccountData;
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};
use crate::{
    constants::solana_programs::{associated_token_account_program, token_program}, error::ReadTransactionError, utils::{address_to_pubkey, addresses_to_pubkeys}
};


/// Represents an associated token account, which holds a specific token 
/// data for a wallet address. Each wallet will have an associated token account
/// for each token (mint) it holds.
/// 
/// This struct includes the key information about the associated token account,
/// such as the wallet's token balance, the token's mint, its supply, and the authority
/// responsible for minting tokens.
///
/// ### Fields
/// 
/// - `pubkey`: The public key of the associated token account.
/// - `owner_pubkey`: The public key of the wallet that owns the associated token account.
/// - `mint_pubkey`: The public key of the token's mint.
/// - `mint_supply`: The current supply of the token in circulation.
/// - `mint_decimals`: The number of decimals used by the token's mint.
/// - `token_amount`: The amount of the token held in the associated token account.
/// - `token_ui_amount`: The token amount in a user-friendly format (e.g., with decimals converted to f64).
/// - `mint_authority`: The authority responsible for minting the token (if any).
/// - `token_program`: The program that owns the token, typically "Token2022" or "Token" for SPL tokens.
#[derive(Debug)]
pub struct AssociatedTokenAccount {
    pub pubkey: String,
    pub owner_pubkey: String,
    pub mint_pubkey: String,
    pub mint_supply: u64,
    pub mint_decimals: u8,
    pub token_amount: u64, 
    pub token_ui_amount: f64, 
    pub mint_authority: Option<Pubkey>, 
    pub token_program: String 
}

/// Derives the associated token account address from the wallet address and mint address. 
/// NOTE: the associated account address differs across different token programs, e.g Token2022 tokens 
/// would have a different associated token account from the standard spl token. 
/// 
/// ### Arguments
/// 
/// * `wallet_address` - address of wallet holding the token.
/// * `mint_address` - address of the target token.
/// * `token_program` - token program that corresponds to the token (e.g token2022 program)
/// 
/// ### Returns
/// 
/// `Result<String, ReadTransactionError>` - Returns a string address of the associated
/// token account on success, or an error if parsing the input addresses to pubkeys fails.
/// This function returns the address regardless if the account exists on the blockchain or not.
/// 
/// ### Example
/// 
/// ```rust
/// use easy_solana::read_transactions::associated_token_account::derive_associated_token_account_address;
/// use easy_solana::constants::solana_programs::{token_2022_program, token_program};
/// 
/// let wallet_address = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
/// let mint_address = "5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp";
/// let result = derive_associated_token_account_address(wallet_address, mint_address, token_program());
/// match result {
///     Ok(address) => println!("Associated Token Account Address: {:?}", address),
///     Err(err) => println!("Invalid wallet or mint address: {:?}", err)
/// }
/// ```
pub fn derive_associated_token_account_address(
    wallet_address: &str, 
    mint_address: &str, 
    token_program: Pubkey
) -> Result<String, ParsePubkeyError> {
    let addresses = vec![wallet_address, mint_address];
    let pubkeys = addresses_to_pubkeys(addresses);
    // checks that pubkeys len == 2 else input wallet / mint address is invalid. 
    if pubkeys.len() != 2 {
        return Err(ParsePubkeyError::Invalid)
    }
    let (associated_token_account_pubkey, _nonce) = Pubkey::find_program_address(
        &[
            &pubkeys[0].to_bytes(),
            &token_program.to_bytes(),
            &pubkeys[1].to_bytes(),
        ],
        &associated_token_account_program(),
    );
    Ok(associated_token_account_pubkey.to_string())
}

// Function to derive associated token account addresses for multiple wallet-mint pairs
pub fn derive_multiple_associated_token_account_addresses(
    wallet_to_mints: &HashMap<String, Vec<String>>,
    token_program: Pubkey,
) -> Result<HashMap<String, Vec<String>>, ParsePubkeyError> {
    let mut result = HashMap::new();

    for (wallet_address, mint_addresses) in wallet_to_mints.iter() {
        let mut associated_token_accounts = Vec::new();

        // Convert wallet address to Pubkey
        let wallet_pubkey = match Pubkey::from_str(wallet_address) {
            Ok(pubkey) => pubkey,
            Err(_) => return Err(ParsePubkeyError::Invalid),
        };

        // Iterate through each mint address for the current wallet
        for mint_address in mint_addresses {
            // Convert mint address to Pubkey
            let mint_pubkey = match Pubkey::from_str(mint_address) {
                Ok(pubkey) => pubkey,
                Err(_) => return Err(ParsePubkeyError::Invalid),
            };

            // Derive the associated token account address
            let (associated_token_account_pubkey, _nonce) = Pubkey::find_program_address(
                &[
                    &wallet_pubkey.to_bytes(),
                    &token_program.to_bytes(),
                    &mint_pubkey.to_bytes(),
                ],
                &associated_token_account_program(),
            );

            // Add the derived associated token account address to the vector
            associated_token_accounts.push(associated_token_account_pubkey.to_string());
        }

        // Insert the list of associated token accounts into the result HashMap
        result.insert(wallet_address.clone(), associated_token_accounts);
    }

    Ok(result)
}


/// Gets the associated token account details, including details of the token it holds. 
/// 
/// # Arguments
/// 
/// * `client` - An instance of the RPC client used to communicate with the blockchain.
/// * `associated_token_account_address` - address of the associated token account. To get the address of an associated token account, use the `derive_associated_token_account_address` function.
/// 
/// # Returns
/// 
/// `Result<AssociatedTokenAccount, ReadTransactionError>` - Returns the `AssociatedTokenAccount` struct on success, or an error if invalid address, non existent account or invalid account data.
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
/// use easy_solana::constants::solana_programs::{token_2022_program, token_program};
/// 
/// let wallet_address = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
/// let mint_address = "5mbK36SZ7J19An8jFochhQS4of8g6BwUjbeCSxBSoWdp";
/// let result = derive_associated_token_account_address(wallet_address, mint_address, token_program());
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
    let associated_token_account_pubkey = address_to_pubkey(associated_token_account_address)?;

    let token_account_data = client.get_account_data(&associated_token_account_pubkey)?;
    let token_account: SplTokenAccount = SplTokenAccount::unpack(&token_account_data)
        .map_err(|_| ReadTransactionError::DeserializeError)?;
    let mint_account = client.get_account(&token_account.mint)?;
    let mint_account_data: SplMintAccount = SplMintAccount::unpack(&mint_account.data)
        .map_err(|_| ReadTransactionError::DeserializeError)?;

    Ok(AssociatedTokenAccount {
        pubkey: associated_token_account_pubkey.to_string(),
        owner_pubkey: token_account.owner.to_string(),
        mint_pubkey: token_account.mint.to_string(),
        mint_supply: mint_account_data.supply,
        mint_decimals: mint_account_data.decimals,
        token_amount: token_account.amount,
        token_ui_amount: token_account.amount as f64 / u64::pow(10, mint_account_data.decimals as u32) as f64,
        mint_authority: mint_account_data.mint_authority.into(),
        token_program: mint_account.owner.to_string()
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
    if associated_token_pubkeys.is_empty() {
        return Err(ReadTransactionError::InvalidAddress(ParsePubkeyError::Invalid))
    }

    // Fetch all account data in a single batch
    let associated_token_accounts = client.get_multiple_accounts(&associated_token_pubkeys)?;

    // Unpack token accounts and collect mint public keys
    let mut mint_pubkeys = Vec::new();
    let mut token_accounts = Vec::new();

    for (pubkey, account_option) in associated_token_pubkeys.iter().zip(associated_token_accounts.into_iter()) {
        if let Some(account) = account_option {
            if let Ok(token_account) = SplTokenAccount::unpack(&account.data) {
                token_accounts.push((pubkey, token_account));
                mint_pubkeys.push(token_account.mint);
            } else {
                eprintln!("get_multiple_associated_token_accounts: Unable to parse SplTokenAccount data for {}", pubkey)
            }
        } else {
            eprintln!("get_multiple_associated_token_accounts: Account not found")
        }
    }
    
    // Fetch mint accounts in a single batch
    let mint_accounts = client.get_multiple_accounts(&mint_pubkeys)?;

    // Deserialise mint accounts and get mint account owner
    let mint_accounts_data: Vec<(SplMintAccount, Pubkey)> = mint_accounts
        .into_iter()
        .filter_map(|account_option| {
            account_option.and_then(|account| {
                SplMintAccount::unpack(&account.data)
                    .ok()
                    .map(|mint_account| (mint_account, account.owner))
            })
        })
        .collect();

    // Build associated token account details by matching token and mint accounts
    let mut associated_token_accounts = Vec::new();

    for ((pubkey, token_account), (mint_account, token_program)) in token_accounts.into_iter().zip(mint_accounts_data.into_iter()) {
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
            token_program: token_program.to_string()
        });
    }

    Ok(associated_token_accounts)
}


#[derive(Debug)]
struct WalletTokenAccount {
    pub pubkey: String,
    pub mint_pubkey: String,
    pub owner_pubkey: String,
    pub token_amount: u64,
    pub ui_amount: f64,
    pub token_program: String
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
/// `Result<Vec<AssociatedTokenAccount>, ReadTransactionError>` - Returns a vector of `AssociatedTokenAccount` 
/// struct on success.
pub fn get_all_token_accounts(
    client: &RpcClient,
    wallet_address: &str,
) -> Result<Vec<AssociatedTokenAccount>, ReadTransactionError> {
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
        // let sol_balance = keyed_account.account.lamports as f64 / LAMPORTS_PER_SOL as f64;
        let token_program = &keyed_account.account.owner;
        if let UiAccountData::Json(parsed_data) = &keyed_account.account.data {
            // Extract `info` field
            let info = parsed_data
                .parsed
                .get("info")
                .ok_or(ReadTransactionError::DeserializeError)?
                .as_object()
                .ok_or(ReadTransactionError::DeserializeError)?;

            // Extract fields from `info`
            let mint_pubkey = info
                .get("mint")
                .and_then(Value::as_str)
                .ok_or(ReadTransactionError::DeserializeError)?
                .parse::<Pubkey>()?;

            let owner_pubkey = info
                .get("owner")
                .and_then(Value::as_str)
                .ok_or(ReadTransactionError::DeserializeError)?
                .parse::<Pubkey>()?;

            let token_amount = info
                .get("tokenAmount")
                .and_then(Value::as_object)
                .ok_or(ReadTransactionError::DeserializeError)?;

            let token_balance = token_amount
                .get("amount")
                .and_then(Value::as_str)
                .ok_or(ReadTransactionError::DeserializeError)?
                .parse::<u64>()
                .map_err(|_| ReadTransactionError::DeserializeError)?;

            // let token_decimals = token_amount
            //     .get("decimals")
            //     .and_then(Value::as_u64)
            //     .unwrap_or(0) as u8;

            let ui_amount = token_amount
                .get("uiAmount")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);

            // Add to the list
            wallet_tokens.push(WalletTokenAccount {
                pubkey: pubkey.to_string(),
                mint_pubkey: mint_pubkey.to_string(),
                owner_pubkey: owner_pubkey.to_string(),
                token_amount: token_balance,
                ui_amount,
                token_program: token_program.to_string()
            });
        }
    }

    let mint_addresses: Vec<&str> = wallet_tokens
        .iter()
        .map(|account| account.mint_pubkey.as_str())
        .collect();
    let mint_pubkeys: Vec<Pubkey> = addresses_to_pubkeys(mint_addresses.clone());

    // There cannot be invalid addresses
    if mint_addresses.len() != mint_pubkeys.len() {
        return Err(ReadTransactionError::InvalidAddress(ParsePubkeyError::Invalid))
    }

    // Fetch mint accounts in a single batch
    let mint_accounts = client.get_multiple_accounts(&mint_pubkeys)?;

    // Deserialise mint accounts and get mint pubkey
    let mint_accounts_data: Vec<SplMintAccount> = mint_accounts
        .into_iter()
        .filter_map(|account_option| {
            account_option.and_then(|account| {
                SplMintAccount::unpack(&account.data)
                    .ok()
            })
        })
        .collect();
    
    let mut associated_token_accounts: Vec<AssociatedTokenAccount> = Vec::new();
    for (wallet_token_account, mint_account) in wallet_tokens.into_iter().zip(mint_accounts_data.into_iter()) {
        associated_token_accounts.push(AssociatedTokenAccount {
            pubkey: wallet_token_account.pubkey,
            owner_pubkey: wallet_token_account.owner_pubkey,
            mint_pubkey: wallet_token_account.mint_pubkey,
            mint_supply: mint_account.supply,
            mint_decimals: mint_account.decimals,
            token_amount: wallet_token_account.token_amount,
            token_ui_amount: wallet_token_account.ui_amount,
            mint_authority: mint_account.mint_authority.into(),
            token_program: wallet_token_account.token_program
        })
    }

    Ok(associated_token_accounts)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::{solana_programs::token_2022_program, utils::create_rpc_client};

    const WALLET_ADDRESS_1: &str = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
    const ASSOCIATED_ACT_ACCOUNT_ADDRESS: &str = "7geCZYWHtghvWj11sb7exvu4uMANfhvGvEvVRRZ8GmSd";
    const ACT_MINT_ADDRESS: &str = "ArDKWeAhQj3LDSo2XcxTUb5j68ZzWg21Awq97fBppump";
    const ASSOCIATED_MIRACOLI_ACCOUNT_ADDRESS: &str = "4rD1Pk3F5R4pUY21vgU3vdnHTGRS97fgD2Y5ozv3FCRx";
    const MIRACOLI_MINT_ADDRESS: &str = "FafEz1HqZwzoNJ626HY8ZNBi2NwUYJE1tVn173rjpump";

    const WALLET_ADDRESS_2: &str = "joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg";
    const PYUSD_TOKEN_ADDRESS: &str = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";
    const ASSOCIATED_PYUSD_ACCOUNT_ADDRESS: &str = "DCk5LLcSw1eyvkU47xTMg7pqLQDsAAtW3PKvtXbgSZRB";
    
    #[test]
    fn test_get_associated_token_account() {
        let client = create_rpc_client("RPC_URL");
        let associated_token_account = get_associated_token_account(
                &client,
                ASSOCIATED_ACT_ACCOUNT_ADDRESS
        ).expect("Failed to get associated token account");
        assert!(associated_token_account.mint_pubkey == ACT_MINT_ADDRESS.to_string());
        assert!(associated_token_account.owner_pubkey == WALLET_ADDRESS_1.to_string());
        assert!(associated_token_account.mint_authority.is_none());
    }

    #[test]
    fn faling_test_get_invalid_associated_token_account() {
        let client = create_rpc_client("RPC_URL");
        // use a wallet address instead
        let associated_token_account_result = get_associated_token_account(
                &client,
                WALLET_ADDRESS_1
        );

        // Check that it's a DeserializeError
        match associated_token_account_result {
            Err(ReadTransactionError::DeserializeError) => {
                assert!(true);
            }
            Err(_) => {
                panic!("Expected DeserializeError, but got a different error");
            }
            Ok(_) => {
                panic!("Expected an error, but got Ok");
            }
        }
    }

    #[test]
    fn test_get_multiple_associated_token_accounts() {
        let client = create_rpc_client("RPC_URL");
        let associated_token_accounts = get_multiple_associated_token_accounts(
                &client,
                vec![ASSOCIATED_ACT_ACCOUNT_ADDRESS, ASSOCIATED_MIRACOLI_ACCOUNT_ADDRESS]   
        ).expect("Failed to get associated token accounts");
        let is_act_token_found = associated_token_accounts.iter().any(|account| account.mint_pubkey.to_string() == ACT_MINT_ADDRESS.to_string());
        let is_miracoli_token_found = associated_token_accounts.iter().any(|account| account.mint_pubkey.to_string() == MIRACOLI_MINT_ADDRESS.to_string());
        assert!(is_act_token_found);
        assert!(is_miracoli_token_found);
    }

    #[test]
    fn failing_test_get_multiple_invalid_associated_token_accounts() {
        let client = create_rpc_client("RPC_URL");
        let associated_token_accounts = get_multiple_associated_token_accounts(
                &client,
                vec![WALLET_ADDRESS_1, ACT_MINT_ADDRESS, MIRACOLI_MINT_ADDRESS]
        ).expect("Failed to get associated token accounts");
        assert!(associated_token_accounts.is_empty())
    }  

    #[test]
    fn test_derive_associated_token_account_address() {
        let associated_token_account_address = derive_associated_token_account_address(
            WALLET_ADDRESS_1, 
            ACT_MINT_ADDRESS, 
            token_program()
        ).unwrap();
        assert!(associated_token_account_address == ASSOCIATED_ACT_ACCOUNT_ADDRESS.to_string())
    }

    #[test]
    fn test_derive_associated_token_2022_account_address() {
        let associated_token_account_address = derive_associated_token_account_address(
            WALLET_ADDRESS_2, 
            PYUSD_TOKEN_ADDRESS, 
            token_2022_program()
        ).unwrap();
        assert!(associated_token_account_address == ASSOCIATED_PYUSD_ACCOUNT_ADDRESS.to_string())
    }

    #[test]
    fn test_derive_multiple_associated_token_accounts_address() {
        let mut wallet_token_mapping: HashMap<String, Vec<String>> = HashMap::new();
        wallet_token_mapping.entry(WALLET_ADDRESS_1.to_string())
            .or_insert_with(|| vec![ACT_MINT_ADDRESS.to_string(), MIRACOLI_MINT_ADDRESS.to_string()]);

        let wallet_associated_account_mapping = derive_multiple_associated_token_account_addresses(
            &wallet_token_mapping,
            token_program()
        ).unwrap();

        let associated_token_account_addresses = wallet_associated_account_mapping.get(WALLET_ADDRESS_1).expect("Wallet does not exist in mapping");
        let is_act_associated_address_found = associated_token_account_addresses.iter().any(|address| address == ASSOCIATED_ACT_ACCOUNT_ADDRESS);
        assert!(is_act_associated_address_found);
        let is_miracoli_associated_address_found = associated_token_account_addresses.iter().any(|address| address == ASSOCIATED_MIRACOLI_ACCOUNT_ADDRESS);
        assert!(is_miracoli_associated_address_found);
    }

    #[test]
    fn test_get_all_token_accounts() {
        let client = create_rpc_client("RPC_URL");
        let token_accounts = get_all_token_accounts(&client, WALLET_ADDRESS_1).expect("Failed to retrieve token accounts");
        let are_tokens_under_same_owner = token_accounts.iter().all(|account| account.owner_pubkey == WALLET_ADDRESS_1.to_string());
        assert!(are_tokens_under_same_owner);
        let is_act_in_token_accounts = token_accounts.iter().any(|account| account.mint_pubkey.to_string() == ACT_MINT_ADDRESS.to_string());
        let is_miracoli_in_token_accounts = token_accounts.iter().any(|account| account.mint_pubkey.to_string() == MIRACOLI_MINT_ADDRESS.to_string());
        assert!(is_act_in_token_accounts);
        assert!(is_miracoli_in_token_accounts);
    }
}