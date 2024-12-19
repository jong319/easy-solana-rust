use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_client::rpc_client::RpcClient;

use crate::{error::ReadTransactionError, get_associated_token_account, utils::address_to_pubkey};

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

pub struct SplTokenBalance {
    pub balance: u64, // balance without decimals
    pub token_decimals: u8, // token decimals
    pub ui_amount: f64 // ui balannce
}
/// Queries an account's token balance. Token decimals are unknown hence balance here is returned
/// in non ui format. 
pub fn get_token_balance(client: &RpcClient, associated_token_account_address: &str) -> Result<SplTokenBalance, ReadTransactionError> {
    let associated_token_account = get_associated_token_account(client, associated_token_account_address)?;
    Ok(SplTokenBalance {
        balance: associated_token_account.token_amount,
        token_decimals: associated_token_account.mint_decimals,
        ui_amount: associated_token_account.token_ui_amount
    })
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
                assert!(token_balance.balance == 869439);
                assert!(token_balance.ui_amount == 869439 as f64 / f64::powi(10.0, token_balance.token_decimals as i32))
            }
            Err(_) => assert!(false) // test fails
        }
    }

}