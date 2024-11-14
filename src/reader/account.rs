use solana_sdk::{
    clock::Epoch, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey
};
use solana_client::rpc_client::RpcClient;
use spl_token::state::{ 
    Account as AssociatedTokenAccount,
    Mint
};
use crate::{ 
    error::AccountReaderError,
    constants::solana_programs::{
        system_program,
        token_program
    },
};


/// Convenience interpretation of a solana Account, containing all the important variables and information.
#[derive(Debug)]
pub struct EasySolanaAccount {
    pub pubkey: Pubkey,
    pub sol_balance: f64, // balance is in SOL, not lamports
    pub account_type: AccountType,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: Epoch,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum AccountType {
    Program,
    Wallet,
    AssociatedTokenAccount,
    MintAccount,
    Others
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
    pub fn get_easy_solana_accounts(&self, pubkeys: &[Pubkey]) -> Result<Vec<EasySolanaAccount>, AccountReaderError> {
        // Fetch multiple accounts
        let accounts_result = self.client.get_multiple_accounts(pubkeys);

        // Handle errors in fetching accounts
        let accounts = accounts_result.map_err(AccountReaderError::from)?;

        // Parse accounts into EasySolanaAccount
        let easy_accounts: Vec<EasySolanaAccount> = accounts
            .into_iter()
            .zip(pubkeys)
            .filter_map(|(account_option, pubkey)| {
                account_option.map(|account| {
                    // Determine the account type
                    let account_type = if account.executable {
                        AccountType::Program
                    } else if account.owner == system_program() {
                        AccountType::Wallet
                    } else if account.owner == token_program() {
                        // Try to unpack as TokenAccount or MintAccount
                        if let Ok(_) = AssociatedTokenAccount::unpack(&account.data) {
                            AccountType::AssociatedTokenAccount
                        } else if let Ok(_) = Mint::unpack(&account.data) {
                            AccountType::MintAccount
                        } else {
                            AccountType::Others
                        }
                    } else {
                        AccountType::Others
                    };

                    EasySolanaAccount {
                        pubkey: *pubkey,
                        sol_balance: account.lamports as f64 / LAMPORTS_PER_SOL as f64, // Convert lamports to SOL
                        account_type,
                        owner: account.owner,
                        executable: account.executable,
                        rent_epoch: account.rent_epoch,
                        data: account.data,
                    }
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

    // Valid Addresses
    const PUMPFUN_PROGRAM_ADDRESS: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
    const PNUT_TOKEN_ADDRESS: &str = "2qEHjDLDLbuBgRYvsxhc5D6uDWAivNFZGan56P1tpump";
    const WALLET_ADDRESS: &str = "G2YxRa6wt1qePMwfJzdXZG62ej4qaTC7YURzuh2Lwd3t";
    const TOKEN_ACCOUNT_ADDRESS: &str = "9Ru8UbszAJnhJpxwXktroHuvfWTHpBN57NHgyLXMw1g";
    // Invalid Account
    const CLOSED_ACCOUNT_ADDRESS: &str = "7o2B9chozpRvHsLgm1Qp3UV9NrS7bx7NH3BZKSePtHEh";
    // Invalid Address
    const INVALID_ADDRESS: &str = "thisisaninvalidaddress";

    #[test]
    fn get_easy_solana_accounts_from_addresses() {
        let addresses: Vec<String> = 
        vec![
            PUMPFUN_PROGRAM_ADDRESS, 
            PNUT_TOKEN_ADDRESS, 
            WALLET_ADDRESS, 
            TOKEN_ACCOUNT_ADDRESS, 
            CLOSED_ACCOUNT_ADDRESS,
            INVALID_ADDRESS
        ].iter_mut()
        .map(|address| address.to_string())
        .collect();

        let client = create_rpc_client("RPC_URL");
        let account_reader = AccountReader::new(client);
        let pubkeys = account_reader.addresses_to_pubkeys(addresses);
        // 5 valid addresses
        assert!(pubkeys.len() == 5);
        let easy_solana_accounts = account_reader.get_easy_solana_accounts(&pubkeys).expect("Failed to fetch accounts");
        // 4 valid accounts
        assert!(easy_solana_accounts.len() == 4);
        for account in easy_solana_accounts {
            assert!(account.sol_balance > 0.0); // all accounts need lamports for rent
            // solana addresses are 32 bytes long
            assert!(account.pubkey.to_bytes().len() == 32); 
            assert!(account.owner.to_bytes().len() == 32);
            if account.executable {
                assert!(account.account_type == AccountType::Program)
            }
            if account.owner == system_program() {
                assert!(account.account_type == AccountType::Wallet)
            } else if account.owner == token_program() {
                assert!(account.account_type == AccountType::MintAccount || account.account_type == AccountType::AssociatedTokenAccount)
            }
        }
    }

    #[test]
    fn invalid_rpc_url_should_fail() {
        let addresses: Vec<String> = 
        vec![
            PUMPFUN_PROGRAM_ADDRESS, 
            PNUT_TOKEN_ADDRESS, 
            WALLET_ADDRESS, 
            TOKEN_ACCOUNT_ADDRESS,
        ].iter_mut()
        .map(|address| address.to_string())
        .collect();

        let client = create_rpc_client("INVALID_RPC_URL");
        let account_reader = AccountReader::new(client);
        let pubkeys = account_reader.addresses_to_pubkeys(addresses);
        let fetch_and_parse_accounts_result = account_reader.get_easy_solana_accounts(&pubkeys);
        assert!(fetch_and_parse_accounts_result.is_err());
    }
}