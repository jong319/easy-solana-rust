use solana_sdk::{
    commitment_config::CommitmentConfig, 
    pubkey::{ParsePubkeyError, Pubkey}, 
    signature::Keypair, 
    signer::Signer,
    bs58
};

use solana_client::rpc_client::RpcClient;

use std::time::Instant;
use dotenv::dotenv;
use std::env;
use regex::Regex;
use log::info;

use crate::error::KeypairError;

/// Generates a solana-sdk `Keypair` struct. 
/// Use optional starts_with and ends_with variables to generate a vanity address. 
pub fn generate_keypair(starts_with: Option<&str>, ends_with: Option<&str>) -> Result<Keypair, KeypairError> {
     // Define valid regex for Solana public key address characters
     let valid_chars_regex = Regex::new(r"^[1-9A-HJ-NP-Za-km-z]*$").unwrap();
     // Validate starts_with and ends_with patterns
     if let Some(prefix) = starts_with {
         if !valid_chars_regex.is_match(prefix) {
             return Err(KeypairError::InvalidPattern);
         }
     }
     if let Some(suffix) = ends_with {
        if !valid_chars_regex.is_match(suffix) {
            return Err(KeypairError::InvalidPattern);
        }
    }

    // Mark the start time and initialise attempts
    let start_time = Instant::now();
    let mut attempts: u64 = 0;
    // Begin keypair creation loop
    loop {
        attempts += 1;
        let keypair = Keypair::new();
        let public_address = keypair.pubkey().to_string();

        let starts_with_match = starts_with.map_or(true, |prefix| public_address.starts_with(prefix));
        let ends_with_match = ends_with.map_or(true, |suffix| public_address.ends_with(suffix));

        if starts_with_match && ends_with_match {
            let private_key = keypair.to_base58_string();
            info!("Wallet Created!");
            info!("Public Address: \n{}", &public_address);
            info!("Private Key: \n{}", &private_key);
            info!("Attempts: {:?}", attempts);
            info!("Time Taken: {:?}", start_time.elapsed());
            return Ok(keypair);
        }

        // Print progress every 10,000 attempts
        if attempts % 100000 == 0 {
            info!("Keypairs Created: {}, Time Elapsed: {:?}", attempts, start_time.elapsed());
        }
    }
}

/// Creates an Rpc Client, accepts an enviroment variable name or direct URL
pub fn create_rpc_client(rpc_input: &str) -> RpcClient {
    // Load environment variables from .env file if present
    dotenv().ok();

    // Check if rpc_input is an environment variable name or a direct URL
    let rpc_url = env::var(rpc_input).unwrap_or_else(|_| rpc_input.to_string());

    RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed())
}

/// Reads a `Vec<String>` of addresses to `Vec<Pubkey>`, invalid addresses are removed.
pub fn addresses_to_pubkeys(addresses: Vec<&str>) -> Vec<Pubkey> {
    addresses
        .into_iter()
        .filter_map(|addr| addr.parse::<Pubkey>().ok())
        .collect()
}

pub fn address_to_pubkey(address: &str) -> Result<Pubkey, ParsePubkeyError> {
    address.parse::<Pubkey>()
}

pub fn base58_to_keypair(keypair_string: &str) -> Result<Keypair, KeypairError> {
    let keypair_bytes = bs58::decode(keypair_string)
    .into_vec()
    .map_err(|_| KeypairError::Base58DecodeError)?;

    Keypair::from_bytes(&keypair_bytes).map_err(|_| KeypairError::InvalidKeypairBytes)
}

#[cfg(test)]
mod tests {
    use solana_sdk::signer::Signer;
    use super::*;

    #[test]
    fn test_generate_invalid_keypair() {
        let invalid_base58_keypair = "asd";
        let keypair = base58_to_keypair(&invalid_base58_keypair);
        println!("{:?}", keypair);
    }

    #[test]
    fn test_generate_keypair_that_starts_with_ab() {
        let ab_keypair = generate_keypair(Some("ab"), None).unwrap();
        assert!(ab_keypair.pubkey().to_string().starts_with("ab"))
    }

    #[test]
    fn test_generate_keypair_that_ends_with_yz() {
        let yz_keypair = generate_keypair(None, Some("yz")).unwrap();
        assert!(yz_keypair.pubkey().to_string().ends_with("yz"))
    }

    #[test]
    fn test_generate_keypair_that_starts_with_a_ends_with_z() {
        let az_keypair = generate_keypair(Some("a"), Some("z")).unwrap();
        assert!(az_keypair.pubkey().to_string().starts_with("a"));
        assert!(az_keypair.pubkey().to_string().ends_with("z"));
    }

    #[test]
    fn test_generate_keypair_with_invalid_pattern() {
        let invalid_keypair = generate_keypair(Some("i"), Some("0"));
        assert!(invalid_keypair.is_err());
    }
}
