use solana_sdk::{native_token::LAMPORTS_PER_SOL, program_pack::Pack};
use solana_client::rpc_client::RpcClient;
use spl_token::state::Account as SplTokenAccount;

use crate::{utils::address_to_pubkey, error::ReadTransactionError};

/// Queries an account's solana balance, returning it in UI format 
/// instead of in Lamports.
/// 
/// Example: 0.02
pub fn get_sol_balance(client: &RpcClient, address: &str) -> Result<f64, ReadTransactionError> {
    // Parse the public address into a Pubkey
    let pubkey = address_to_pubkey(address)?;

    // Fetch the account balance in lamports
    let balance = client.get_balance(&pubkey)?;
    let ui_balance = balance as f64 / LAMPORTS_PER_SOL as f64;

    Ok(ui_balance)
}

/// Queries an account's token balance. Token decimals are unknown hence balance here is returned
/// in non ui format. 
pub fn get_token_balance(client: &RpcClient, associated_token_account_address: &str) -> Result<u64, ReadTransactionError> {
    // Parse the public address into a Pubkey
    let pubkey = address_to_pubkey(associated_token_account_address)?;

    let account_data = client.get_account_data(&pubkey)?;
    let token_account: SplTokenAccount = SplTokenAccount::unpack(&account_data)
        .map_err(|_| ReadTransactionError::DeserializeError)?;

    Ok(token_account.amount)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::create_rpc_client;

    const EMPTY_WALLET_ADDRESS: &str = "7o2B9chozpRvHsLgm1Qp3UV9NrS7bx7NH3BZKSePtHEh";
    const ASSOCIATED_HAPPY_CAT_WALLET_ADDRESS: &str = "4ZVBVjcaLUqUxVi3EHaVKp1pZ96AZoznyGWgWxKYZhsD";
    
    #[test]
    fn test_get_sol_balance() {
        let client = create_rpc_client("RPC_URL");
        match get_sol_balance(&client, EMPTY_WALLET_ADDRESS) {
            Ok(sol_balance) => {
                assert!(sol_balance == 0.0)
            }
            Err(err) => {
                println!("{:?}", err);
                assert!(false) // test fails
            }
        }
    }

    #[test]
    fn test_get_token_balance() {
        let client = create_rpc_client("RPC_URL");
        match get_token_balance(&client, ASSOCIATED_HAPPY_CAT_WALLET_ADDRESS) {
            Ok(token_balance) => {
                assert!(token_balance == 869439);
            }
            Err(_) => assert!(false) // test fails
        }
    }

}