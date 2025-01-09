use solana_sdk::{pubkey::Pubkey, signer::Signer};
use spl_associated_token_account::instruction::create_associated_token_account;

use crate::{
    error::TransactionBuilderError, utils::address_to_pubkey
};

use super::transaction_builder::TransactionBuilder;

impl TransactionBuilder<'_> { 
    /// Adds a create associated token account instruction into the transaction.
    /// This instruction only creates an associated token account for the signing keypair.
    /// If you wish to create an associated token account for other accounts, use the 
    /// `create_associated_token_account_for_others` function instead. 
    /// 
    /// ## Arguments
    /// 
    /// * `token_address` - Address of token for the associated token account
    /// * `token_program` - Pubkey of the relevant token program (e.g Token2022) 
    /// 
    /// ## Errors
    /// 
    /// Invalid token address will throw a `TransactionBuilderError::InvalidAddress`
    /// 
    /// ## Example
    /// 
    /// ```rust
    /// use dotenv::dotenv;
    /// use std::env;
    /// use solana_sdk::signer::keypair::Keypair;
    /// use easy_solana::create_rpc_client;
    /// use easy_solana::write_transactions::transaction_builder::TransactionBuilder;
    /// use easy_solana::write_transactions::utils::simulate_transaction;
    /// use easy_solana::constants::solana_programs::{token_2022_program, token_program};
    /// 
    /// const PYUSD_TOKEN_ADDRESS: &str = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";
    /// 
    /// dotenv().ok();
    /// let private_key_string = env::var("PRIVATE_KEY_1").unwrap();
    /// let private_key = Keypair::from_base58_string(&private_key_string);
    /// let client = create_rpc_client("RPC_URL");
    /// let create_token_account_transaction = TransactionBuilder::new(&client, &private_key)
    ///     .set_compute_units(50_000)
    ///     .set_compute_limit(1_000_000)
    ///     .create_associated_token_account_for_payer(PYUSD_TOKEN_ADDRESS, token_2022_program())
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    /// let simulation_result = simulate_transaction(&client, create_token_account_transaction).expect("Failed to simulate transaction");
    /// ```
    pub fn create_associated_token_account_for_payer(&mut self, token_address: &str, token_program: Pubkey) -> Result<&mut Self, TransactionBuilderError> {
        // Payer account
        let payer_account = self.payer_keypair.pubkey();
        // Token account
        let token_account = address_to_pubkey(token_address)?;

        let create_associated_token_account_instruction = create_associated_token_account(
            &payer_account,
            &payer_account,
            &token_account,
            &token_program,
        );

        self.instructions.push(create_associated_token_account_instruction);

        Ok(self)
    }


    /// Adds a create associated token account instruction into the transaction.
    /// This instruction creates an associated token account for the target account. 
    /// The signing keypair will pay for the rent fee. 
    /// 
    /// ## Arguments
    /// 
    /// * `token_address` - Address of token for the associated token account
    /// * `target_account_address` - Address of the target account to create the associated token account for
    /// * `is_token_2022` - Whether the target token is under the Token 2022 program. 
    /// 
    /// ## Errors
    /// 
    /// Invalid token address or target account address will throw a `TransactionBuilderError::InvalidAddress`
    /// 
    /// ## Example
    /// 
    /// ```rust
    /// use dotenv::dotenv;
    /// use std::env;
    /// use solana_sdk::signer::keypair::Keypair;
    /// use easy_solana::create_rpc_client;
    /// use easy_solana::write_transactions::transaction_builder::TransactionBuilder;
    /// use easy_solana::write_transactions::utils::simulate_transaction;
    /// use easy_solana::constants::solana_programs::{token_2022_program, token_program};
    /// 
    /// const WALLET_ADDRESS_1: &str = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
    /// const USDC_TOKEN_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    /// 
    /// dotenv().ok();
    /// let private_key_string = env::var("PRIVATE_KEY_2").unwrap();
    /// let private_key = Keypair::from_base58_string(&private_key_string);
    /// let client = create_rpc_client("RPC_URL");
    /// let create_token_account_transaction = TransactionBuilder::new(&client, &private_key)
    ///     .set_compute_units(50_000)
    ///     .set_compute_limit(1_000_000)
    ///     .create_associated_token_account_for_others(USDC_TOKEN_ADDRESS, WALLET_ADDRESS_1, token_program())
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    /// let simulation_result = simulate_transaction(&client, create_token_account_transaction).expect("Failed to simulate transaction");
    /// ```
    pub fn create_associated_token_account_for_others(&mut self, token_address: &str, target_account_address: &str, token_program: Pubkey) -> Result<&mut Self, TransactionBuilderError> {
        // Payer account
        let payer_account = self.payer_keypair.pubkey();
        // Target Account 
        let target_account = address_to_pubkey(target_account_address)?;
        // Token account
        let token_account = address_to_pubkey(token_address)?;

        let create_associated_token_account_instruction = create_associated_token_account(
            &payer_account,
            &target_account,
            &token_account,
            &token_program,
        );

        self.instructions.push(create_associated_token_account_instruction);

        Ok(self)
    }
}


#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use solana_sdk::signature::Keypair;
    use std::env;
    use crate::{
        solana_programs::{token_2022_program, token_program}, utils::create_rpc_client, write_transactions::{transaction_builder::TransactionBuilder, utils::simulate_transaction}
    };

    const WALLET_ADDRESS_1: &str = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
    const WALLET_ADDRESS_2: &str = "joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg";
    const USDC_TOKEN_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    // PYUSD is under the Token2022 program
    const PYUSD_TOKEN_ADDRESS: &str = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";
    
    #[test]
    fn test_simulate_create_token_account_with_fee_accounts() {
        dotenv().ok();
        let private_key_string = env::var("PRIVATE_KEY_1").unwrap();
        let private_key = Keypair::from_base58_string(&private_key_string);

        let client = create_rpc_client("RPC_URL");

        let create_token_account_transaction = TransactionBuilder::new(&client, &private_key)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            // transfer to fee account
            .transfer_sol(0.018, &private_key, WALLET_ADDRESS_2)
            .unwrap()
            // transfer to referral account
            .transfer_sol(0.002, &private_key, WALLET_ADDRESS_2)
            .unwrap()
            .create_associated_token_account_for_payer(USDC_TOKEN_ADDRESS, token_program())
            .unwrap()
            .build()
            .unwrap();

        let simulation_result = simulate_transaction(&client, create_token_account_transaction).expect("Failed to simulate transaction");
        assert!(simulation_result.error.is_none());
    }

    #[test]
    fn test_simulate_create_token_2022_account() {
        dotenv().ok();
        let private_key_string = env::var("PRIVATE_KEY_1").unwrap();
        let private_key = Keypair::from_base58_string(&private_key_string);

        let client = create_rpc_client("RPC_URL");

        let create_token_account_transaction = TransactionBuilder::new(&client, &private_key)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            .create_associated_token_account_for_payer(PYUSD_TOKEN_ADDRESS, token_2022_program())
            .unwrap()
            .build()
            .unwrap();

        let simulation_result = simulate_transaction(&client, create_token_account_transaction).expect("Failed to simulate transaction");
        assert!(simulation_result.error.is_none());
    }

    #[test]
    fn test_simulate_create_token_account_for_others() {
        dotenv().ok();
        let private_key_string = env::var("PRIVATE_KEY_2").unwrap();
        let private_key = Keypair::from_base58_string(&private_key_string);

        let client = create_rpc_client("RPC_URL");

        let create_token_account_transaction = TransactionBuilder::new(&client, &private_key)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            .create_associated_token_account_for_others(USDC_TOKEN_ADDRESS, WALLET_ADDRESS_1, token_program())
            .unwrap()
            .build()
            .unwrap();

        let simulation_result = simulate_transaction(&client, create_token_account_transaction).expect("Failed to simulate transaction");
        assert!(simulation_result.error.is_none());
    }
}
