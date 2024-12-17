use solana_program::system_instruction;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{
        Keypair, 
        Signer
    }
};
use crate::{error::TransactionBuilderError, utils::address_to_pubkey};
use super::transaction_builder::TransactionBuilder;

impl<'a> TransactionBuilder<'a> {
    pub fn transfer_sol(&mut self, amount: f64, from_keypair: &'a Keypair, destination_address: &str) -> Result<&mut Self, TransactionBuilderError> {
        let destination_pubkey = address_to_pubkey(destination_address)?;
        let lamports = (amount * LAMPORTS_PER_SOL as f64) as u64;
        let instruction = system_instruction::transfer(&from_keypair.pubkey(), &destination_pubkey, lamports);
        self.instructions.push(instruction);
        
        // if from_keypair is not the payer_keypair, add it to signing keypairs
        if from_keypair.pubkey() != self.payer_keypair.pubkey() {
            self.signing_keypairs.push(&from_keypair);
        }
        Ok(self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use dotenv::dotenv;
    use std::env;
    use crate::{
        utils::create_rpc_client,
        write_transactions::utils::simulate_transaction
    };

    const WALLET_ADDRESS_1: &str = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
    const WALLET_ADDRESS_2: &str = "joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg";
    
    // #[tokio::test(flavor = "multi_thread", worker_threads = 2)]  // Multi-threaded runtime
    #[test]
    fn test_simulate_transfer_sol() {
        dotenv().ok();
        let private_key_string = env::var("PRIVATE_KEY_2").expect("Cannot find PRIVATE_KEY env var");
        let payer_account_keypair = Keypair::from_base58_string(&private_key_string);

        let client = create_rpc_client("RPC_URL");

        let transfer_sol_transaction = TransactionBuilder::new(&client, &payer_account_keypair)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            .transfer_sol(0.001, &payer_account_keypair, WALLET_ADDRESS_1)
            .unwrap()
            .build()
            .unwrap();

        let simulation_result = simulate_transaction(&client, transfer_sol_transaction).expect("Failed to simulate transaction");
        assert!(simulation_result.error.is_none());
    }

    #[test]
    fn test_transfer_all_sol() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY_1").expect("Cannot find PRIVATE_KEY env var");
        let client = create_rpc_client("RPC_URL");
        let keypair = Keypair::from_base58_string(&private_key);
        let simulated_transaction = TransactionBuilder::new(&client, &keypair)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            .transfer_sol(1_000_000.0, &keypair, WALLET_ADDRESS_2)
            .unwrap() // transaction builder error
            .build()
            .unwrap();
        let simulation_result = simulate_transaction(&client, simulated_transaction).unwrap();
        // 134359540.0
        let mut transfer_amount = 0.0;
        let re = Regex::new(r"Transfer: insufficient lamports (\d+), need \d+").unwrap();
        for log in simulation_result.transaction_logs {
            if let Some(caps) = re.captures(&log) {
                // Extract the first capture group and parse it as f64.
                if let Some(lamports_str) = caps.get(1) {
                    if let Ok(lamports) = lamports_str.as_str().parse::<f64>() {
                        transfer_amount = lamports;
                    }
                }
            }
        }
        let wallet_data_length = client.get_account_data(&keypair.pubkey()).unwrap().len();
        let minimum_sol_for_rent_exemption = client.get_minimum_balance_for_rent_exemption(wallet_data_length).unwrap();
        transfer_amount -= minimum_sol_for_rent_exemption as f64;
        
        let transfer_transaction = TransactionBuilder::new(&client, &keypair)
            .set_compute_units(50_000)
            .set_compute_limit(simulation_result.units_consumed)
            .transfer_sol(transfer_amount / LAMPORTS_PER_SOL as f64, &keypair, WALLET_ADDRESS_2)
            .unwrap() // transaction builder error
            .build()
            .unwrap();

        let new_simulation_result = simulate_transaction(&client, transfer_transaction).unwrap();
        assert!(new_simulation_result.error.is_none())
    }
}
