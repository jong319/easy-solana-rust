use solana_sdk::{
    clock::Epoch, native_token::LAMPORTS_PER_SOL, program_pack::Pack, pubkey::Pubkey 
};
use solana_client::{rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use spl_token::state::{ 
    Account as SplTokenAccount,
    Mint as SplMintAccount
};

use crate::{
    utils::{
        address_to_pubkey,
        addresses_to_pubkeys
    },
    constants::solana_programs::{
        system_program,
        token_program
    }, 
    error::AccountReaderError,
    reader::metadata_account::get_metadata_of_token,
};

use super::{
    wallet::{WalletAccount, parse_token_accounts},
    mint::MintAccount,
    associated_token_account::AssociatedTokenAccount,
};

/// A basic easy solana account, containing all the important variables and information.
/// Use [`parse_account`] method to transform an EasySolanaAccount to a [`EasySolanaParsedAccount`] 
/// determined by its [`AccountType`]
#[derive(Debug)]
pub struct EasySolanaAccount {
    pub pubkey: Pubkey,
    /// balance is in SOL, not lamports
    pub sol_balance: f64,
    /// Program that owns the account, Wallet accounts are owned by the System Program
    /// Associated Token Accounts and Mint Accounts are owned by the Token Program.
    pub program_owner_pubkey: Pubkey,
    pub executable: bool,
    pub rent_epoch: Epoch,
    pub data: Vec<u8>,
    pub account_type: AccountType,
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

/// Result from using [`parse_account`] on an [`EasySolanaAccount`], parses based on [`AccountType`].
#[derive(Debug)]
pub enum EasySolanaParsedAccount {
    Wallet(WalletAccount),
    AssociatedTokenAccount(AssociatedTokenAccount),
    MintAccount(MintAccount),
    Others(EasySolanaAccount),
}

/// Results from using [`parse_multiple_accounts`] on a [`Vec<EasySolanaAccount>`].
pub struct EasySolanaParsedAccounts {
    pub wallets: Vec<WalletAccount>,
    pub token_accounts: Vec<AssociatedTokenAccount>,
    pub mint_accounts: Vec<MintAccount>,
    pub others: Vec<EasySolanaAccount>,
}

/// Fetches an account given a pubkey, returning a [`EasySolanaAccount`]. 
/// To fetch and parse multiple accounts, use the [`get_multiple_accounts`] method instead.
/// ## Errors
/// If the account does not exist or fails to fetch the data, returns a [`AccountReaderError`]
/// ## Example
/// ```
/// use easy_solana::{
///     utils::create_rpc_client,
///     reader::easy_solana_accounts::account::get_account,
/// };
/// 
/// let client = create_rpc_client("https://api.mainnet-beta.solana.com");
/// let address = "joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg".to_string();
/// let easy_solana_account = get_account(&client, address).unwrap();
/// ```
pub fn get_account(client: &RpcClient, address: String) -> Result<EasySolanaAccount, AccountReaderError> {
    let pubkey = address_to_pubkey(address)?;
    let account = client.get_account(&pubkey)
    .map_err(AccountReaderError::from)?;
    // convert sol balance from lamports to SOL
    let sol_balance = account.lamports as f64 / LAMPORTS_PER_SOL as f64;
    let program_owner_pubkey = account.owner;
    let executable = account.executable;
    let rent_epoch = account.rent_epoch;
    let data = account.data;
    // Determine the account type
    let account_type = if account.executable {
        AccountType::Program
    } else if account.owner == system_program() {
        AccountType::Wallet
    } else if account.owner == token_program() {
        if let Ok(_) = SplTokenAccount::unpack(&data) {
            AccountType::AssociatedTokenAccount
        } else if let Ok(_) = SplMintAccount::unpack(&data) {
            AccountType::MintAccount
        } else {
            AccountType::Others
        }
    } else {
        AccountType::Others
    };

    Ok(EasySolanaAccount {
        pubkey,
        sol_balance,
        program_owner_pubkey,
        executable,
        rent_epoch,
        data,
        account_type,
    })
}
    

/// Fetches multiple accounts given a slice of Pubkeys, returning a [`Vec<EasySolanaAccount>`]. 
/// Accounts are determined by their [`AccountType`].
/// /// ## Errors
/// If it fails to fetch the data, returns a [`AccountReaderError`]. Otherwise, a non existent 
/// account will just be filtered out. 
/// ## Example
/// ```
/// use easy_solana::{
///     utils::create_rpc_client,
///     reader::easy_solana_accounts::account::get_multiple_accounts,
/// };
/// let addresses = vec!["joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg".to_string()];
/// let client = create_rpc_client("https://api.mainnet-beta.solana.com");
/// let easy_solana_accounts = get_multiple_accounts(&client, addresses).unwrap();
/// ```
pub fn get_multiple_accounts(client: &RpcClient, addresses: Vec<String>) -> Result<Vec<EasySolanaAccount>, AccountReaderError> {
    let pubkeys = addresses_to_pubkeys(addresses);
    // Step 1: Fetch multiple accounts
    let accounts = client.get_multiple_accounts(&pubkeys)
        .map_err(AccountReaderError::from)?; // handle error

    // Step 2: Parse accounts into EasySolanaAccount
    let easy_accounts: Vec<EasySolanaAccount> = accounts
        .into_iter()
        .zip(pubkeys)
        .filter_map(|(account_option, pubkey)| {
            account_option.map(|account| {
                // convert sol balance from lamports to SOL
                let sol_balance = account.lamports as f64 / LAMPORTS_PER_SOL as f64;
                let program_owner_pubkey = account.owner;
                let executable = account.executable;
                let rent_epoch = account.rent_epoch;
                let data = account.data;
                // Determine the account type
                let account_type = if account.executable {
                    AccountType::Program
                } else if account.owner == system_program() {
                    AccountType::Wallet
                } else if account.owner == token_program() {
                    // Try to unpack as TokenAccount or MintAccount
                    if let Ok(_) = SplTokenAccount::unpack(&data) {
                        AccountType::AssociatedTokenAccount
                    } else if let Ok(_) = SplMintAccount::unpack(&data) {
                        AccountType::MintAccount
                    } else {
                        AccountType::Others
                    }
                } else {
                    AccountType::Others
                };

                EasySolanaAccount {
                    pubkey,
                    sol_balance,
                    program_owner_pubkey,
                    executable,
                    rent_epoch,
                    data,
                    account_type,
                }
            })
        })
        .collect();
    Ok(easy_accounts)
}

/// Used to parse an [`EasySolanaAccount`] into either a [`WalletAccount`],[`AssociatedTokenAccount`]
/// or [`MintAccount`], depending on its [`AccountType`].
/// ## Errors
/// Throws an [`AccountReaderError`] if RpcClient failed to retrieve data or if deserializing account data fails
pub fn parse_account(client: &RpcClient, account: EasySolanaAccount) -> Result<EasySolanaParsedAccount, AccountReaderError> {
    match account.account_type {
        AccountType::Wallet => {
            // Get all token accounts by wallet
            let token_account_filter = TokenAccountsFilter::ProgramId(token_program());
            let token_accounts = client.get_token_accounts_by_owner(&account.pubkey, token_account_filter)?;    
            let wallet_tokens = parse_token_accounts(token_accounts);
            Ok(EasySolanaParsedAccount::Wallet(WalletAccount {
                base: account,
                token_accounts: wallet_tokens,
            }))
        }
        AccountType::AssociatedTokenAccount => {
            let associated_token_account = SplTokenAccount::unpack(&account.data)
                .map_err(|_| AccountReaderError::DeserializeError)?;
            Ok(EasySolanaParsedAccount::AssociatedTokenAccount(AssociatedTokenAccount {
                base: account,
                mint_pubkey: associated_token_account.mint,
                token_balance: associated_token_account.amount,
                owner_pubkey: associated_token_account.owner,
                state: associated_token_account.state,
                close_authority: associated_token_account.close_authority.into()
            }))
        }
        AccountType::MintAccount => {
            let mint_account = SplMintAccount::unpack(&account.data)
                .map_err(|_| AccountReaderError::DeserializeError)?;
            let metadata_account = get_metadata_of_token(client, account.pubkey.to_string())?;
            Ok(EasySolanaParsedAccount::MintAccount(MintAccount {
                base: account,
                metadata: metadata_account,
                supply: mint_account.supply,
                decimals: mint_account.decimals,
                is_initialized: mint_account.is_initialized,
                freeze_authority: mint_account.freeze_authority.into(),
                mint_authority: mint_account.mint_authority.into(),
            }))
        }
        // Program Accounts and Others cannot be parsed without additional information
        _ => Ok(EasySolanaParsedAccount::Others(account))
    }
}




#[cfg(test)]
mod tests {
    use crate::create_rpc_client;

    use super::*;

    const WALLET_ADDRESS: &str = "joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg";
    const WALLET_ASSOCIATED_HAPPY_CAT_ADDRESS: &str = "4ZVBVjcaLUqUxVi3EHaVKp1pZ96AZoznyGWgWxKYZhsD";
    const HAPPY_CAT_MINT_ADDRESS: &str = "2Df5GphWffyegX9Xjwhjz96hozATStb8YBaouaYcpump";
    // Invalid Account
    const CLOSED_ACCOUNT_ADDRESS: &str = "7o2B9chozpRvHsLgm1Qp3UV9NrS7bx7NH3BZKSePtHEh";
    // Invalid Address
    const INVALID_ADDRESS: &str = "thisisaninvalidaddress";

    #[test]
    fn test_get_wallet_account() {
        let client = create_rpc_client("RPC_URL");
        let easy_solana_account = get_account(&client, WALLET_ADDRESS.to_string()).expect("Failed to fetch accounts");
        assert!(easy_solana_account.sol_balance > 0.0); // all accounts need lamports for rent
        // solana addresses are 32 bytes long
        assert!(easy_solana_account.pubkey.to_bytes().len() == 32); 
        assert!(easy_solana_account.program_owner_pubkey.to_bytes().len() == 32);
        assert!(easy_solana_account.account_type == AccountType::Wallet);
        let wallet_account = parse_account(&client, easy_solana_account).expect("Failed to parse account");
        match wallet_account {
            EasySolanaParsedAccount::Wallet(wallet) => {
                assert!(wallet.token_accounts.len() == 3);
            }
            // Fail the test if any other type
            _ => assert!(false)
        }
    }

    #[test]
    fn test_get_associated_token_account() {
        let client = create_rpc_client("RPC_URL");
        let easy_solana_account = get_account(&client, WALLET_ASSOCIATED_HAPPY_CAT_ADDRESS.to_string()).expect("Failed to fetch accounts");
        assert!(easy_solana_account.sol_balance > 0.0); // all accounts need lamports for rent
        // solana addresses are 32 bytes long
        assert!(easy_solana_account.pubkey.to_bytes().len() == 32); 
        assert!(easy_solana_account.program_owner_pubkey.to_bytes().len() == 32);
        assert!(easy_solana_account.account_type == AccountType::AssociatedTokenAccount);
        let associated_token_account = parse_account(&client, easy_solana_account).expect("Failed to parse account");
        match associated_token_account {
            EasySolanaParsedAccount::AssociatedTokenAccount(account) => {
                assert!(account.owner_pubkey == WALLET_ADDRESS.parse::<Pubkey>().unwrap());
                assert!(account.mint_pubkey == HAPPY_CAT_MINT_ADDRESS.parse::<Pubkey>().unwrap());
            }
            // Fail the test if any other type
            _ => assert!(false)
        }
    }

    #[test]
    fn test_get_mint_account() {
        let client = create_rpc_client("RPC_URL");
        let easy_solana_account = get_account(&client, HAPPY_CAT_MINT_ADDRESS.to_string()).expect("Failed to fetch accounts");
        assert!(easy_solana_account.sol_balance > 0.0); // all accounts need lamports for rent
        // solana addresses are 32 bytes long
        assert!(easy_solana_account.pubkey.to_bytes().len() == 32); 
        assert!(easy_solana_account.program_owner_pubkey.to_bytes().len() == 32);
        assert!(easy_solana_account.account_type == AccountType::MintAccount);
        let associated_token_account = parse_account(&client, easy_solana_account).expect("Failed to parse account");
        match associated_token_account {
            EasySolanaParsedAccount::MintAccount(mint) => {
                assert!(mint.metadata.data.name == "HAPPY".to_string());
                assert!(mint.metadata.data.symbol == "Happy Cat".to_string());
                assert!(mint.metadata.mint == HAPPY_CAT_MINT_ADDRESS.parse::<Pubkey>().unwrap());
            }
            // Fail the test if any other type
            _ => assert!(false)
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
        let easy_solana_accounts = get_multiple_accounts(&client, addresses).expect("Failed to fetch accounts");
        // 3 valid accounts
        assert!(easy_solana_accounts.len() == 3);
        for account in easy_solana_accounts {
            assert!(account.sol_balance > 0.0); // all accounts need lamports for rent
            // solana addresses are 32 bytes long
            assert!(account.pubkey.to_bytes().len() == 32); 
            assert!(account.program_owner_pubkey.to_bytes().len() == 32);
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
        let fetch_and_parse_accounts_result = get_multiple_accounts(&client, addresses);
        assert!(fetch_and_parse_accounts_result.is_err());
    }
}