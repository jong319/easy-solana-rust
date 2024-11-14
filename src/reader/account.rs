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
    pub fn addresses_to_pubkeys(&self, addresses: Vec<String>) -> Vec<Pubkey> {
        addresses
            .into_iter()
            .filter_map(|addr| addr.parse::<Pubkey>().ok())
            .collect()
    }

    /// Fetches and parses accounts given a slice of Pubkeys, returning a Vec<EasySolanaAccount>. 
    /// Invalid accounts are removed. If RPC client fails to fetch data, returns a `AccountReaderError`
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





#[cfg(test)]
mod tests {
    use crate::create_rpc_client;
    use super::*;

    #[test]
    fn read_valid_addresses() {
        // Valid addresses
        let pumpfun_address = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string();
        let pnut_token_address = "2qEHjDLDLbuBgRYvsxhc5D6uDWAivNFZGan56P1tpump".to_string();
        let wallet_address = "G2YxRa6wt1qePMwfJzdXZG62ej4qaTC7YURzuh2Lwd3t".to_string();
        let token_account_address = "9Ru8UbszAJnhJpxwXktroHuvfWTHpBN57NHgyLXMw1g".to_string();
        // Invalid account
        let closed_account_address = "7o2B9chozpRvHsLgm1Qp3UV9NrS7bx7NH3BZKSePtHEh".to_string();
        // Invalid addresses
        let invalid_address = "thisisaninvalidaddress".to_string();

        let addresses: Vec<String> = vec![
            pumpfun_address, 
            pnut_token_address, 
            wallet_address, 
            token_account_address, 
            closed_account_address,
            invalid_address
        ];

        let client = create_rpc_client("RPC_URL");
        let account_reader = AccountReader::new(client);
        let pubkeys = account_reader.addresses_to_pubkeys(addresses);
        // 5 valid addresses
        assert!(pubkeys.len() == 5);
        let easy_solana_accounts = account_reader.fetch_and_parse_accounts(&pubkeys).expect("Failed to fetch accounts");
        // 4 valid accounts
        assert!(easy_solana_accounts.len() == 4);
        for account in easy_solana_accounts {
            assert!(account.sol_balance > 0.0); // all accounts need lamports for rent
            // solana addresses are 32 bytes long
            assert!(account.pubkey.to_bytes().len() == 32); 
            assert!(account.owner.to_bytes().len() == 32);
        }
    }
}