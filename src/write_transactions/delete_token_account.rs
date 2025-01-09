use spl_token_2022::instruction::{close_account, burn};
use solana_sdk::{pubkey::Pubkey, signature::Signer};
use crate::{
    error::TransactionBuilderError, 
    read_transactions::associated_token_account::derive_associated_token_account_address, 
    utils::address_to_pubkey
};

use super::transaction_builder::TransactionBuilder;

impl TransactionBuilder<'_> { 
    /// Adds a delete associated token account instruction into the transaction.
    /// This instruction will delete an associated token account for the payer keypair,
    /// and return the rent amount to the rent recipient. The balance of the token has to be
    /// 0 for the instruction to succeed, use the `burn_tokens` method first to remove all 
    /// outstanding balance.
    /// 
    /// ## Arguments
    /// 
    /// * `token_address` - Address of token for the associated token account
    /// * `target_account_address` - Address of the target account to create the associated token account for
    /// * `token_program` - Pubkey of the relevant token program (e.g Token2022) 
    /// 
    /// ## Errors
    /// 
    /// Invalid token address or target account address will throw a `TransactionBuilderError::InvalidAddress`
    /// 
    /// ## Example 
    /// ```
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
    /// let private_key = env::var("PRIVATE_KEY_2").expect("Cannot find PRIVATE_KEY env var");
    /// let client = create_rpc_client("RPC_URL");
    /// let keypair = Keypair::from_base58_string(&private_key);
    /// let close_account_transaction = TransactionBuilder::new(&client, &keypair)
    ///     .set_compute_units(50_000)
    ///     .set_compute_limit(1_000_000)
    ///     .delete_associated_token_account(USDC_TOKEN_ADDRESS, WALLET_ADDRESS_1, token_program())
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    /// let simulation_result = simulate_transaction(&client, close_account_transaction).unwrap();
    /// ```
    pub fn delete_associated_token_account(&mut self, token_address: &str, rent_recipient: &str, token_program: Pubkey) -> Result<&mut Self, TransactionBuilderError>  {
        // Payer account
        let payer_account = self.payer_keypair.pubkey();
        // Associated token account 
        let associated_token_account_address = derive_associated_token_account_address(
            &payer_account.to_string(), 
            token_address, 
            token_program
        )?;
        let associated_token_account = address_to_pubkey(&associated_token_account_address)?;
        // Rent Recipient 
        let rent_recipient_account = address_to_pubkey(rent_recipient)?;

        // Create the close account instruction
        let close_instruction = close_account(
            &token_program,
            &associated_token_account,
            &rent_recipient_account,
            &payer_account,
            &[],
        ).map_err(|err| TransactionBuilderError::InstructionError(err.to_string()))?;

        self.instructions.push(close_instruction);

        Ok(self)
    }

    pub fn burn_tokens(&mut self, token_address: &str, amount: u64, token_program: Pubkey) -> Result<&mut Self, TransactionBuilderError>  {
        // Payer account
        let payer_account = self.payer_keypair.pubkey();
        // Associated token account 
        let associated_token_account_address = derive_associated_token_account_address(
            &payer_account.to_string(), 
            token_address, 
            token_program
        )?;
        let associated_token_account = address_to_pubkey(&associated_token_account_address)?;
        // Token account
        let token_account = address_to_pubkey(token_address)?;

        let burn_instruction = burn(
            &token_program,
            &associated_token_account,
            &token_account,
            &payer_account,
            &[],
            amount,
        ).map_err(|err| TransactionBuilderError::InstructionError(err.to_string()))?;

        self.instructions.push(burn_instruction);

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signer::keypair::Keypair;
    use dotenv::dotenv;
    use std::env;
    use crate::{
        get_associated_token_account, 
        read_transactions::associated_token_account::get_all_token_accounts, 
        utils::create_rpc_client, 
        write_transactions::utils::simulate_transaction,
        constants::solana_programs::{token_2022_program, token_program}
    };

    const WALLET_ADDRESS_1: &str = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
    const WALLET_ADDRESS_2: &str = "joNASGVYc6ugNiUCsamrJ8i2PBoxFW9YvqNisNfFNXg";
    const SOL_KING_TOKEN_ADDRESS: &str = "CMo3SMFDgJBsnKPFy9rKSSGq7jQWCnt1SqRByT5Cpump";
    const USDC_TOKEN_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    // PYUSD is under the Token2022 program
    const PYUSD_TOKEN_ADDRESS: &str = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";   

    #[test]
    fn failing_test_close_token_account_with_balance() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY_2").expect("Cannot find PRIVATE_KEY env var");
        let client = create_rpc_client("RPC_URL");
        let keypair = Keypair::from_base58_string(&private_key);

        let close_account_transaction = TransactionBuilder::new(&client, &keypair)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            .delete_associated_token_account(SOL_KING_TOKEN_ADDRESS, WALLET_ADDRESS_1, token_program())
            .unwrap()
            .build()
            .unwrap();

        let simulation_result = simulate_transaction(&client, close_account_transaction).unwrap();
        assert!(simulation_result.error.is_some());
    }

    #[test]
    fn test_burn_and_close_token_account_with_balance() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY_2").expect("Cannot find PRIVATE_KEY env var");
        let client = create_rpc_client("RPC_URL");
        let keypair = Keypair::from_base58_string(&private_key);

        let associated_token_account_address = derive_associated_token_account_address(
            WALLET_ADDRESS_2, 
            SOL_KING_TOKEN_ADDRESS, 
            token_program()
        ).unwrap();
        let associated_token_account = get_associated_token_account(&client, &associated_token_account_address).unwrap();
        let balance = associated_token_account.token_amount;

        let close_account_transaction = TransactionBuilder::new(&client, &keypair)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            .burn_tokens(SOL_KING_TOKEN_ADDRESS, balance, token_program())
            .unwrap()
            .delete_associated_token_account(SOL_KING_TOKEN_ADDRESS, WALLET_ADDRESS_1, token_program())
            .unwrap()
            .build()
            .unwrap();

        let simulation_result = simulate_transaction(&client, close_account_transaction).unwrap();
        assert!(simulation_result.error.is_none());
    }

    #[test]
    fn test_close_token_account_with_no_balance() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY_2").expect("Cannot find PRIVATE_KEY env var");
        let client = create_rpc_client("RPC_URL");
        let keypair = Keypair::from_base58_string(&private_key);

        let close_account_transaction = TransactionBuilder::new(&client, &keypair)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            .delete_associated_token_account(USDC_TOKEN_ADDRESS, WALLET_ADDRESS_1, token_program())
            .unwrap()
            .build()
            .unwrap();

        let simulation_result = simulate_transaction(&client, close_account_transaction).unwrap();
        assert!(simulation_result.error.is_none())
    }

    #[test]
    fn test_close_token_2022_account_with_no_balance() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY_2").expect("Cannot find PRIVATE_KEY env var");
        let client = create_rpc_client("RPC_URL");
        let keypair = Keypair::from_base58_string(&private_key);

        let close_account_transaction = TransactionBuilder::new(&client, &keypair)
            .set_compute_units(50_000)
            .set_compute_limit(1_000_000)
            .delete_associated_token_account(PYUSD_TOKEN_ADDRESS, WALLET_ADDRESS_1, token_2022_program())
            .unwrap()
            .build()
            .unwrap();

        let simulation_result = simulate_transaction(&client, close_account_transaction).unwrap();
        assert!(simulation_result.error.is_none())
    }
    
    #[test]
    fn test_simulate_burn_and_delete_all_token_accounts() {
        dotenv().ok();
        let private_key_string = env::var("PRIVATE_KEY_2").expect("Cannot find PRIVATE_KEY env var");
        let payer_account_keypair = Keypair::from_base58_string(&private_key_string);
        let payer_account = payer_account_keypair.pubkey();

        let client = create_rpc_client("RPC_URL");

        let wallet_token_accounts = get_all_token_accounts(
            &client, 
            &payer_account.to_string()
        ).expect("Unable to get token accounts");

        let mut builder = TransactionBuilder::new(&client, &payer_account_keypair);
            
        builder.set_compute_units(50_000);
        builder.set_compute_limit(1_000_000);

        for token in wallet_token_accounts {
            let token_program = address_to_pubkey(&token.token_program).unwrap();
            if token.token_amount > 0 {
                let _ = builder.burn_tokens(&token.mint_pubkey.to_string(), token.token_amount, token_program).unwrap();
            }
            let _ = builder.delete_associated_token_account(&token.mint_pubkey, &payer_account.to_string(), token_program).unwrap();
        }

        let burn_and_delete_transaction = builder.build().unwrap();

        let simulation_result = simulate_transaction(&client, burn_and_delete_transaction).expect("Failed to simulate transaction");
        assert!(simulation_result.error.is_none());
    }
}
