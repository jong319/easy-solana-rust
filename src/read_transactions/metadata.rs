use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use borsh::{
    BorshDeserialize,
    BorshSerialize
};
use crate::{
    solana_programs::metadata_program, 
    utils::{address_to_pubkey, addresses_to_pubkeys},
    error::ReadTransactionError
};


 #[derive(BorshSerialize, BorshDeserialize, Debug)]
 pub struct MetadataAccount {
     pub key: u8,
     pub update_authority: Pubkey,
     pub mint: Pubkey,
     pub data: Metadata,
     pub primary_sale_happened: bool,
     pub is_mutable: bool,
 }
 
 #[derive(BorshSerialize, BorshDeserialize, Debug)]
 pub struct Metadata {
     pub name: String,
     pub symbol: String,
     pub uri: String,
 }


/// Fetches the metadata account given a token address, deserializing their data and returning `MetadataAccount`. 
/// Paddings in token name, symbol and uri are trimmed.
/// 
/// ### Arguments
pub fn get_metadata_of_token(client: &RpcClient, token_address: &str) -> Result<MetadataAccount, ReadTransactionError> {
    let token_pubkey = address_to_pubkey(token_address)?;
    let metadata_program = metadata_program();
    // Get pubkey of the token's metadata account by deriving it from their seed
    let seed = &[b"metadata", metadata_program.as_ref(), token_pubkey.as_ref()];
    let (metadata_pubkey, _nonce) = Pubkey::find_program_address(seed, &metadata_program);
    // Fetch account data
    let metadata_account = client.get_account(&metadata_pubkey)?;

    // Deserialize account data
    let mut deserialized_metadata_account = 
        MetadataAccount::deserialize(&mut metadata_account.data.as_ref())
        .map_err(|_| ReadTransactionError::DeserializeError)?;

    // Trim paddings
    deserialized_metadata_account.data.name = deserialized_metadata_account.data.name.trim_end_matches('\0').to_string();
    deserialized_metadata_account.data.symbol = deserialized_metadata_account.data.symbol.trim_end_matches('\0').to_string();
    deserialized_metadata_account.data.uri = deserialized_metadata_account.data.uri.trim_end_matches('\0').to_string();

    Ok(deserialized_metadata_account)
}

/// Fetches the metadata accounts given a multiple token Pubkeys, deserializing their data and returning [`Vec<MetadataAccount>`]. 
/// Paddings in token name, symbol and uri are trimmed.
/// ## Errors
/// If RPC client fails to fetch data, return a [`AccountReaderError::RpcClientError`].
/// Metadata accounts that cannot be deserialized or non existent accounts are filtered out.
pub fn get_metadata_of_tokens(client: &RpcClient, token_addresses: Vec<&str>) -> Result<Vec<MetadataAccount>, ReadTransactionError> {
    let token_pubkeys = addresses_to_pubkeys(token_addresses);
    let metadata_program = metadata_program();
    // Get the pubkeys of the token's metadata accounts by deriving it from their seed
    let pubkeys_of_metadata_account: Vec<Pubkey> = token_pubkeys
        .iter() 
        .map(|token_pubkey| {
            let seeds = &[b"metadata", metadata_program.as_ref(), token_pubkey.as_ref()];
            let (metadata_pubkey, _nonce) = Pubkey::find_program_address(seeds, &metadata_program);
            metadata_pubkey
        })
        .collect();

    // Fetch the metadata accounts
    let metadata_accounts = client.get_multiple_accounts(&pubkeys_of_metadata_account)?;

    // deserialize accounts 
    let data_of_metadata_accounts: Vec<MetadataAccount> = metadata_accounts
        .into_iter()
        .flatten()
        .filter_map(|account| {
            let mut metadata_account = MetadataAccount::deserialize(&mut account.data.as_ref()).ok()?;
            metadata_account.data.name = metadata_account.data.name.trim_end_matches('\0').to_string();
            metadata_account.data.symbol = metadata_account.data.symbol.trim_end_matches('\0').to_string();
            metadata_account.data.uri = metadata_account.data.uri.trim_end_matches('\0').to_string();
            Some(metadata_account)
        })
        .collect();

    Ok(data_of_metadata_accounts)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::create_rpc_client;

    const PNUT_TOKEN_ADDRESS: &str = "2qEHjDLDLbuBgRYvsxhc5D6uDWAivNFZGan56P1tpump";
    const ACT_MINT_ADDRESS: &str = "ArDKWeAhQj3LDSo2XcxTUb5j68ZzWg21Awq97fBppump";
    const MIRACOLI_MINT_ADDRESS: &str = "FafEz1HqZwzoNJ626HY8ZNBi2NwUYJE1tVn173rjpump";
    const WALLET_ADDRESS: &str = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
    
    #[test]
    fn test_get_metadata_of_token() {
        let client = create_rpc_client("RPC_URL");
        let pnut_metadata = get_metadata_of_token(&client, PNUT_TOKEN_ADDRESS).expect("Failed to fetch accounts");
        assert!(pnut_metadata.mint.to_string() == PNUT_TOKEN_ADDRESS.to_string());
        assert!(pnut_metadata.data.name == "Peanut the Squirrel ".to_string());
        assert!(pnut_metadata.data.symbol == "Pnut ".to_string());
    }

    #[test]
    fn failing_test_get_metadata_of_invalid_token() {
        let client = create_rpc_client("RPC_URL");
        let result = get_metadata_of_token(&client, WALLET_ADDRESS);
        // Check that it's a RpcForUserError
        match result {
            Err(ReadTransactionError::RpcForUserError(err)) => {
                println!("{:}", err);
                assert!(true);
            }
            Err(_) => {
                panic!("Expected RpcForUserError, but got a different error");
            }
            Ok(_) => {
                panic!("Expected an error, but got Ok");
            }
        }
    }

    #[test]
    fn test_get_metadata_of_tokens() {
        let client = create_rpc_client("RPC_URL");
        let metadata_of_tokens = get_metadata_of_tokens(&client, vec![PNUT_TOKEN_ADDRESS, MIRACOLI_MINT_ADDRESS, ACT_MINT_ADDRESS]).expect("Failed to fetch accounts");
        assert!(metadata_of_tokens.len() == 3);
        let is_pnut_token_found = metadata_of_tokens.iter().any(|token| token.data.name == "Peanut the Squirrel ".to_string());
        assert!(is_pnut_token_found);
    }
}