use solana_sdk::{
    signature::Keypair,
    signer::Signer
};
use std::time::Instant;
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