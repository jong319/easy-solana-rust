use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    commitment_config::CommitmentConfig
};
use solana_client::rpc_client::RpcClient;

use std::time::Instant;
use dotenv::dotenv;
use std::env;
use log::info;

/// Generates a solana-sdk `Keypair` struct. 
/// Use optional starts_with and ends_with variables to generate a vanity address. 
pub fn generate_keypair(starts_with: Option<&str>, ends_with: Option<&str>) -> Keypair {
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
            return keypair;
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


#[cfg(test)]
mod tests {
    use solana_sdk::signer::Signer;
    use super::*;

    #[test]
    fn generate_keypair_that_starts_with_ab() {
        let ab_keypair = generate_keypair(Some("ab"), None);
        assert!(ab_keypair.pubkey().to_string().starts_with("ab"))
    }

    #[test]
    fn generate_keypair_that_ends_with_yz() {
        let yz_keypair = generate_keypair(None, Some("yz"));
        assert!(yz_keypair.pubkey().to_string().ends_with("yz"))
    }

    #[test]
    fn generate_keypair_that_starts_with_a_ends_with_z() {
        let az_keypair = generate_keypair(Some("a"), Some("z"));
        assert!(az_keypair.pubkey().to_string().starts_with("a"));
        assert!(az_keypair.pubkey().to_string().ends_with("z"));
    }
}
