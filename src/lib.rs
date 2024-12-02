//! # EasySolana
//!
//! EasySolana simplifies querying data and writing transactions on the Solana
//! blockchain network. It takes multistep procedures like fetching, deserializing,
//! simulating and signing transactions into simple functions for developers to utilize.
//! In particular, it integrates seamlessly with Pump.fun programs and allows price queries, 
//! buy, sell and creation transactions. 
//!
//! ## Features
//! - Querying account data 
//! - Querying token details 
//! - Simulating transaction
//! - Sending transaction
//! - Integration with Pump.fun methods
//!
//! ## Example
//! ```rust
//! use easy_solana::{
//!     create_rpc_client, 
//!     error::ReadTransactionError, 
//!     pumpfun::bonding_curve::{
//!         get_bonding_curve_account, 
//!         BondingCurveAccount, 
//!         calculate_token_price
//!     },
//! }
//! 
//! let client = create_rpc_client("https://api.mainnet-beta.solana.com");
//! let pumpfun_token_address = "CzAdDkkbRJnPYYjuwZ8T6tUxtD2ouCpZMXkJD7Rhpump";
//! let (bonding_curve_account, bonding_curve_data) = get_bonding_curve_account(&client, pumpfun_token_address).unwrap();
//! let token_price_in_sol = calculate_token_price(&bonding_curve_data);
//! ```
//!
//! ## License
//! EasySolana is licensed under MIT or Apache 2.0.



pub mod utils;
pub use utils::{
    generate_keypair,
    create_rpc_client
};

pub mod read_transactions;
pub use read_transactions::{
    metadata::{get_metadata_of_token, get_metadata_of_tokens},
    balances::{get_sol_balance, get_token_balance},
    associated_token_account::{AssociatedTokenAccount, get_associated_token_account}
};

pub mod constants;
pub use constants::{
    solana_programs,
    pumpfun_accounts
};

pub mod error;

pub mod pumpfun;
pub mod raydium;
pub mod write_transactions;