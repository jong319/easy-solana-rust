use solana_sdk::program_pack::Pack;
use solana_client::rpc_client::RpcClient;
use spl_token::state::Mint as SplMintAccount;

use crate::{
    utils::{address_to_pubkey, addresses_to_pubkeys},
    error::ReadTransactionError
};


pub fn get_mint_account(client: &RpcClient, token_address: &str) -> Result<SplMintAccount, ReadTransactionError> {
    let token_pubkey = address_to_pubkey(token_address)?;
    let token_account = client.get_account(&token_pubkey)?;
    let mint_data = SplMintAccount::unpack(&token_account.data)
        .map_err(|_| ReadTransactionError::DeserializeError)?; 
    
    Ok(mint_data)
}

pub fn get_multiple_mint_accounts(client: &RpcClient, token_addresses: Vec<&str>) -> Result<Vec<SplMintAccount>, ReadTransactionError> {
    let token_pubkeys = addresses_to_pubkeys(token_addresses);
    let mut token_accounts = client.get_multiple_accounts(&token_pubkeys)?;
    let token_accounts_data = token_accounts
        .iter_mut()
        .filter_map(|account_option| {
            // Check if the account is Some and try to unpack it
            account_option.as_mut().and_then(|account| SplMintAccount::unpack(&account.data).ok())
        })
        .collect();
    
    Ok(token_accounts_data)
}