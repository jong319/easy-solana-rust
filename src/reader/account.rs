use solana_sdk::{
    clock::Epoch, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey
};
use solana_client::rpc_client::RpcClient;
use spl_token::state::{ 
    Account as AssociatedTokenAccount,
    Mint,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize
};
use crate::{ 
    constants::solana_programs::{
        system_program,
        token_program
    }, 
    error::AccountReaderError, 
    solana_programs::metadata_program,
};


/// Convenience interpretation of a basic solana account, containing all the important variables and information.
/// Use the relevant schemas to decode data variable if needed.
#[derive(Debug)]
pub struct EasySolanaAccount {
    pub pubkey: Pubkey,
    /// balance is in SOL, not lamports
    pub sol_balance: f64,
    pub account_type: AccountType,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: Epoch,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum AccountType {
    /// A program account is executable
    Program,
    /// A wallet account is owned by the system program
    Wallet,
    /// An associated token account
    AssociatedTokenAccount,
    /// An mint account
    MintAccount,
    /// Unknown account type
    Others
}

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

    /// Fetches the metadata accounts given a slice of Pubkeys, deserializing their data and returning `Vec<MetadataAccount>`.
    /// Invalid accounts and metadata that cannot be deserialized are removed. If RPC client fails to fetch data, returns a `AccountReaderError`.
    pub fn get_metadata_of_tokens(&self, token_pubkeys: &[Pubkey]) -> Result<Vec<MetadataAccount>, AccountReaderError> {
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
    
        // Fetch the metadata accounts and deserialize them
        let data_of_metadata_accounts: Vec<MetadataAccount> = self.client
            .get_multiple_accounts(&pubkeys_of_metadata_account)
            .map_err(AccountReaderError::from)?
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
}





#[cfg(test)]
mod tests {
    use crate::{
        create_rpc_client,
        addresses_to_pubkeys
    };
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
        // Convert String addresses to Pubkeys
        let pubkeys = addresses_to_pubkeys(addresses);
        let account_reader = AccountReader::new(client);
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
        let pubkeys = addresses_to_pubkeys(addresses);
        let account_reader = AccountReader::new(client);
        let fetch_and_parse_accounts_result = account_reader.get_easy_solana_accounts(&pubkeys);
        assert!(fetch_and_parse_accounts_result.is_err());
    }

    #[test]
    fn get_metadata_of_tokens() {
        let addresses: Vec<String> = 
        vec![
            PNUT_TOKEN_ADDRESS,
            WALLET_ADDRESS
        ].iter_mut()
        .map(|address| address.to_string())
        .collect();

        let client = create_rpc_client("RPC_URL");
        let pubkeys = addresses_to_pubkeys(addresses);
        let account_reader = AccountReader::new(client);
        let metadata_of_tokens = account_reader.get_metadata_of_tokens(&pubkeys).expect("Failed to fetch accounts");
        assert!(metadata_of_tokens.len() == 1);
        assert!(metadata_of_tokens[0].mint.to_string() == PNUT_TOKEN_ADDRESS.to_string())
    }
}