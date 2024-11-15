use solana_sdk::pubkey::Pubkey;
use solana_client::rpc_client::RpcClient;
use borsh::{
    BorshDeserialize,
    BorshSerialize
};
use crate::{
    error::AccountReaderError, 
    solana_programs::metadata_program,
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

/// Fetches the metadata account given a token Pubkey, deserializing their data and returning `MetadataAccount`.
/// Invalid account, metadata that cannot be deserialized and if RPC client fails to fetch data, return a `AccountReaderError`.
pub fn get_metadata_of_token(client: &RpcClient, token_pubkey: &Pubkey) -> Result<MetadataAccount, AccountReaderError> {
    let metadata_program = metadata_program();
    // Get pubkey of the token's metadata account by deriving it from their seed
    let seed = &[b"metadata", metadata_program.as_ref(), token_pubkey.as_ref()];
    let (metadata_pubkey, _nonce) = Pubkey::find_program_address(seed, &metadata_program);

    // Fetch account data and handle errors
    let metadata_account = client
        .get_account(&metadata_pubkey)
        .map_err(AccountReaderError::from)?;

    // Deserialize account data
    let mut deserialized_metadata_account = 
        MetadataAccount::deserialize(&mut metadata_account.data.as_ref())
        .map_err(|_| AccountReaderError::DeserializeError)?;

    // Trim paddings
    deserialized_metadata_account.data.name = deserialized_metadata_account.data.name.trim_end_matches('\0').to_string();
    deserialized_metadata_account.data.symbol = deserialized_metadata_account.data.symbol.trim_end_matches('\0').to_string();
    deserialized_metadata_account.data.uri = deserialized_metadata_account.data.uri.trim_end_matches('\0').to_string();

    Ok(deserialized_metadata_account)
}

/// Fetches the metadata accounts given a slice of token Pubkeys, deserializing their data and returning `Vec<MetadataAccount>`.
/// Invalid accounts and metadata that cannot be deserialized are removed. If RPC client fails to fetch data, returns a `AccountReaderError`.
pub fn get_metadata_of_tokens(client: &RpcClient, token_pubkeys: &[Pubkey]) -> Result<Vec<MetadataAccount>, AccountReaderError> {
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
    let metadata_accounts_result = client.get_multiple_accounts(&pubkeys_of_metadata_account);

    // handle errors in fetching account
    let metadata_accounts = metadata_accounts_result.map_err(AccountReaderError::from)?;

    // deserialize accounts 
    let data_of_metadata_accounts: Vec<MetadataAccount> = metadata_accounts
        .into_iter()
        .filter_map(|account_option| account_option)
        .map(|account| {
            let mut metadata_account = MetadataAccount::deserialize(&mut account.data.as_ref()).ok()?;
            metadata_account.data.name = metadata_account.data.name.trim_end_matches('\0').to_string();
            metadata_account.data.symbol = metadata_account.data.symbol.trim_end_matches('\0').to_string();
            metadata_account.data.uri = metadata_account.data.uri.trim_end_matches('\0').to_string();
            Some(metadata_account)
        })
        .filter_map(|metadata_account|metadata_account)
        .collect();

    Ok(data_of_metadata_accounts)
}


#[cfg(test)]
mod tests {
    use crate::{
        create_rpc_client,
        addresses_to_pubkeys,
    };
    use super::*;

    const PNUT_TOKEN_ADDRESS: &str = "2qEHjDLDLbuBgRYvsxhc5D6uDWAivNFZGan56P1tpump";

    #[test]
    fn test_get_metadata_of_tokens() {
        let addresses: Vec<String> = 
        vec![
            PNUT_TOKEN_ADDRESS,
        ].iter_mut()
        .map(|address| address.to_string())
        .collect();

        let client = create_rpc_client("RPC_URL");
        let pubkeys = addresses_to_pubkeys(addresses);
        let metadata_of_tokens = get_metadata_of_tokens(&client, &pubkeys).expect("Failed to fetch accounts");
        assert!(metadata_of_tokens.len() == 1);
        let pnut_metadata = &metadata_of_tokens[0];
        assert!(pnut_metadata.mint.to_string() == PNUT_TOKEN_ADDRESS.to_string());
        assert!(pnut_metadata.data.name == "Peanut the Squirrel ".to_string());
        assert!(pnut_metadata.data.symbol == "Pnut ".to_string());
    }
}