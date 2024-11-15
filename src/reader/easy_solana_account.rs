use solana_sdk::{
    clock::Epoch,
    program_pack::Pack, 
    pubkey::Pubkey,
    native_token::LAMPORTS_PER_SOL
};
use solana_client::rpc_client::RpcClient;
use spl_token::state::{ 
    Account as AssociatedTokenAccount,
    AccountState,
    Mint
};

use crate::{
    constants::solana_programs::{
        system_program,
        token_program
    }, error::AccountReaderError, get_metadata_of_token, get_metadata_of_tokens
};

/// Convenience interpretation of a basic solana account, containing all the important variables and information.
/// Use the relevant schemas to decode data variable if needed.
#[derive(Debug)]
pub struct EasySolanaAccount {
    pub pubkey: Pubkey,
    /// balance is in SOL, not lamports
    pub sol_balance: f64,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: Epoch,
    pub data: Vec<u8>,
    pub account_type: AccountType,
}

#[derive(Debug)]
pub enum AccountType {
    /// A program account is executable
    Program,
    /// A wallet account is owned by the system program
    Wallet,
    /// An associated token account
    AssociatedTokenAccount(AssociatedTokenAccountDetails),
    /// An mint account
    MintAccount(MintAccountDetails),
    /// Unknown account type
    Others
}

/// Convenience interpretation of a Associated Token account 
#[derive(Debug)]
pub struct AssociatedTokenAccountDetails {
    /// The mint associated with this account
    pub mint_address: Pubkey,
    /// amount of tokens this account holds, decimals unknown
    pub token_balance: u64, 
    /// Wallet owner of the associated account
    pub owner: Pubkey, 
    /// The account's state
    pub state: AccountState,
    /// Optional authority to close the account.
    pub close_authority: Option<Pubkey>,
}

/// Convenience interpretation of a Mint account 
#[derive(Debug)]
pub struct MintAccountDetails {
    pub token_name: String,
    pub token_ticker: String,
    pub token_uri: String,
    /// Total supply of tokens.
    pub supply: u64,
    /// Number of base 10 digits to the right of the decimal place.
    pub decimals: u8,
    /// Is `true` if this structure has been initialized
    pub is_initialized: bool,
    /// Optional authority to freeze token accounts.
    pub freeze_authority: Option<Pubkey>,
    // Optional authority to mint tokens
    pub mint_authority: Option<Pubkey>,
}

/// Fetches and parses an account given a pubkey, returning a EasySolanaAccount. 
/// Invalid account and if RPC client fails to fetch data, returns a `AccountReaderError`
pub fn get_easy_solana_account(client: &RpcClient, pubkey: &Pubkey) -> Result<EasySolanaAccount, AccountReaderError> {
    // Fetch account data 
    let account = client
        .get_account(pubkey)
        .map_err(AccountReaderError::from)?;

    // Determine the account type
    let account_type = if account.executable {
        AccountType::Program
    } else if account.owner == system_program() {
        AccountType::Wallet
    } else if account.owner == token_program() {
        if let Ok(associated_token_account) = AssociatedTokenAccount::unpack(&account.data) {
            AccountType::AssociatedTokenAccount(AssociatedTokenAccountDetails {
                mint_address: associated_token_account.mint,
                token_balance: associated_token_account.amount,
                owner: associated_token_account.owner,
                state: associated_token_account.state,
                close_authority: associated_token_account.close_authority.into()
            })
        } else if let Ok(mint_account) = Mint::unpack(&account.data) {
            let mut token_name = String::new();
            let mut token_ticker = String::new();
            let mut token_uri = String::new();
            if let Ok(token_metadata) = get_metadata_of_token(client, pubkey) {
                token_name = token_metadata.data.name;
                token_ticker = token_metadata.data.symbol;
                token_uri = token_metadata.data.uri;
            }
            AccountType::MintAccount(MintAccountDetails {
                token_name,
                token_ticker,
                token_uri,
                supply: mint_account.supply,
                decimals: mint_account.decimals,
                is_initialized: mint_account.is_initialized,
                freeze_authority: mint_account.freeze_authority.into(),
                mint_authority: mint_account.mint_authority.into()
            })
        } else {
            AccountType::Others
        }
    } else {
        AccountType::Others
    };

    Ok(EasySolanaAccount {
        pubkey: *pubkey,
        sol_balance: account.lamports as f64 / LAMPORTS_PER_SOL as f64, // Convert lamports to SOL
        owner: account.owner,
        executable: account.executable,
        rent_epoch: account.rent_epoch,
        data: account.data,
        account_type,
    })
}

/// Fetches and parses accounts given a slice of Pubkeys, returning a Vec<EasySolanaAccount>. 
/// Invalid accounts are removed. If RPC client fails to fetch data, returns a `AccountReaderError`
pub fn get_easy_solana_accounts(client: &RpcClient, pubkeys: &[Pubkey]) -> Result<Vec<EasySolanaAccount>, AccountReaderError> {
    // Step 1: Fetch multiple accounts
    let accounts_result = client.get_multiple_accounts(pubkeys);
    let accounts = accounts_result.map_err(AccountReaderError::from)?; // handle error

    // Step 2: Parse accounts into EasySolanaAccount
    let mut easy_accounts: Vec<EasySolanaAccount> = accounts
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
                    if let Ok(associated_token_account) = AssociatedTokenAccount::unpack(&account.data) {
                        AccountType::AssociatedTokenAccount(AssociatedTokenAccountDetails {
                            mint_address: associated_token_account.mint,
                            token_balance: associated_token_account.amount,
                            owner: associated_token_account.owner,
                            state: associated_token_account.state,
                            close_authority: associated_token_account.close_authority.into()
                        })
                    } else if let Ok(mint_account) = Mint::unpack(&account.data) {
                        AccountType::MintAccount(MintAccountDetails {
                            token_name: String::new(),
                            token_ticker: String::new(),
                            token_uri: String::new(),
                            supply: mint_account.supply,
                            decimals: mint_account.decimals,
                            is_initialized: mint_account.is_initialized,
                            freeze_authority: mint_account.freeze_authority.into(),
                            mint_authority: mint_account.mint_authority.into()
                        })
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

    // Step 3: Filter for Vector of mint pubkeys to query metadata in a bulk request
    let mint_pubkeys: Vec<Pubkey> = easy_accounts
    .iter()
    .filter_map(|account| match account.account_type {
        AccountType::MintAccount(_) => Some(account.pubkey),
        _ => None,
    })
    .collect();

    if !mint_pubkeys.is_empty() {
        let metadata_accounts = get_metadata_of_tokens(client, &mint_pubkeys)?;

        for metadata_account in metadata_accounts {
            if let Some(account) = easy_accounts.iter_mut().find(|acc| acc.pubkey == metadata_account.mint) {
                if let AccountType::MintAccount(details) = &mut account.account_type {
                    details.token_name = metadata_account.data.name.clone();
                    details.token_ticker = metadata_account.data.symbol.clone();
                    details.token_uri = metadata_account.data.uri.clone();
                }
            }
        }
    }

    Ok(easy_accounts)
}


#[cfg(test)]
mod tests {
    use crate::{
        addresses_to_pubkeys, 
        create_rpc_client,
    };

    use super::*;

    const WALLET_ADDRESS: &str = "joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg";
    const WALLET_ASSOCIATED_HAPPY_CAT_ADDRESS: &str = "4ZVBVjcaLUqUxVi3EHaVKp1pZ96AZoznyGWgWxKYZhsD";
    const HAPPY_CAT_MINT_ADDRESS: &str = "2Df5GphWffyegX9Xjwhjz96hozATStb8YBaouaYcpump";
    // Invalid Account
    const CLOSED_ACCOUNT_ADDRESS: &str = "7o2B9chozpRvHsLgm1Qp3UV9NrS7bx7NH3BZKSePtHEh";
    // Invalid Address
    const INVALID_ADDRESS: &str = "thisisaninvalidaddress";

    #[test]
    fn test_get_easy_solana_account() {
        let pubkey = HAPPY_CAT_MINT_ADDRESS.parse::<Pubkey>().unwrap();
        let client = create_rpc_client("RPC_URL");
        let easy_solana_account = get_easy_solana_account(&client, &pubkey).expect("Failed to fetch accounts");
        assert!(easy_solana_account.sol_balance > 0.0); // all accounts need lamports for rent
        // solana addresses are 32 bytes long
        assert!(easy_solana_account.pubkey.to_bytes().len() == 32); 
        assert!(easy_solana_account.owner.to_bytes().len() == 32);
        match easy_solana_account.account_type {
            AccountType::MintAccount(details) => {
                assert!(details.decimals == 6);
                assert!(details.token_name == "HAPPY");
                assert!(details.token_ticker == "Happy Cat");
            }
            _ => {}
        }
    }

    #[test]
    fn test_get_easy_solana_accounts() {
        let addresses: Vec<String> = 
        vec![ 
            HAPPY_CAT_MINT_ADDRESS, 
            WALLET_ADDRESS, 
            WALLET_ASSOCIATED_HAPPY_CAT_ADDRESS, 
            CLOSED_ACCOUNT_ADDRESS,
            INVALID_ADDRESS
        ].iter_mut()
        .map(|address| address.to_string())
        .collect();

        let client = create_rpc_client("RPC_URL");
        // Convert String addresses to Pubkeys
        let pubkeys = addresses_to_pubkeys(addresses);
        // 4 valid addresses
        assert!(pubkeys.len() == 4);
        let easy_solana_accounts = get_easy_solana_accounts(&client, &pubkeys).expect("Failed to fetch accounts");
        // 3 valid accounts
        assert!(easy_solana_accounts.len() == 3);
        for account in easy_solana_accounts {
            assert!(account.sol_balance > 0.0); // all accounts need lamports for rent
            // solana addresses are 32 bytes long
            assert!(account.pubkey.to_bytes().len() == 32); 
            assert!(account.owner.to_bytes().len() == 32);
            match account.account_type {
                AccountType::AssociatedTokenAccount(details) => {
                    assert!(details.mint_address.to_string() == HAPPY_CAT_MINT_ADDRESS.to_string());
                    assert!(details.owner.to_string() == WALLET_ADDRESS.to_string());
                    assert!(details.token_balance == 869439);
                    assert!(account.sol_balance == 0.00203928);
                }
                AccountType::MintAccount(details) => {
                    assert!(details.decimals == 6);
                    assert!(details.token_name == "HAPPY");
                    assert!(details.token_ticker == "Happy Cat");
                }
                _ => {}
            }
        }
    }

    #[test]
    fn failing_test_invalid_rpc_url() {
        let addresses: Vec<String> = 
        vec![
            WALLET_ADDRESS, 
        ].iter_mut()
        .map(|address| address.to_string())
        .collect();

        let client = create_rpc_client("INVALID_RPC_URL");
        let pubkeys = addresses_to_pubkeys(addresses);
        let fetch_and_parse_accounts_result = get_easy_solana_accounts(&client, &pubkeys);
        assert!(fetch_and_parse_accounts_result.is_err());
    }
}