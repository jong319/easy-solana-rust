use solana_sdk::{
    clock::Epoch, 
    native_token::LAMPORTS_PER_SOL, 
    pubkey::Pubkey
};
use solana_client::rpc_client::RpcClient;

use crate::error::AccountReaderError;

#[derive(Debug)]
pub struct EasySolanaAccount {
    pub pubkey: Pubkey,
    pub sol_balance: f64, // balance is in SOL, not lamports
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: Epoch,
    pub data: Vec<u8>,
}

pub struct AccountReader {
    client: RpcClient
}

impl AccountReader {
    // Create a new Account Reader
    pub fn new(client: RpcClient) -> Self {
        Self {
            client
        }
    }

    /// Reads a `Vec<String>` of addresses to `Vec<Pubkey>`, invalid addresses are removed.
    pub fn read_addresses(&self, addresses: Vec<String>) -> Vec<Pubkey> {
        addresses
            .into_iter()
            .filter_map(|addr| addr.parse::<Pubkey>().ok())
            .collect()
    }

    /// Fetches and parses accounts given a slice of Pubkeys, returning a Vec<EasySolanaAccount>. 
    /// If RPC client fails to fetch data, returns a `AccountReaderError`
    pub fn fetch_and_parse_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<EasySolanaAccount>, AccountReaderError> {
        // Fetch multiple accounts
        let accounts_result = self.client.get_multiple_accounts(pubkeys);

        // Handle errors in fetching accounts
        let accounts = accounts_result.map_err(AccountReaderError::from)?;

        // Parse accounts into EasySolanaAccount
        let easy_accounts: Vec<EasySolanaAccount> = accounts
            .into_iter()
            .zip(pubkeys)
            .filter_map(|(account_option, pubkey)| {
                account_option.map(|account| EasySolanaAccount {
                    pubkey: *pubkey,
                    sol_balance: account.lamports as f64 / LAMPORTS_PER_SOL as f64, // Convert lamports to SOL
                    owner: account.owner,
                    executable: account.executable,
                    rent_epoch: account.rent_epoch,
                    data: account.data,
                })
            })
            .collect();

        Ok(easy_accounts)
    }
}



