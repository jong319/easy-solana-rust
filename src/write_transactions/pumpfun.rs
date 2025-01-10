use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, 
    signer::Signer
};

use solana_program::instruction::{AccountMeta, Instruction};

use crate::{
    constants::{
        pumpfun_accounts::{
            buy_instruction_data, pumpfun_event_authority_account, pumpfun_fee_account, pumpfun_global_account, pumpfun_program, sell_instruction_data, PUMP_TOKEN_DECIMALS
        },
        solana_programs::{
            associated_token_account_program, rent_program, system_program, token_program
        }
    },
    pumpfun::bonding_curve::{ get_bonding_curve_account, calculate_token_price_in_sol },
    error::TransactionBuilderError, 
    read_transactions::associated_token_account::derive_associated_token_account_address, 
    utils::address_to_pubkey
};
use super::transaction_builder::TransactionBuilder;

impl TransactionBuilder<'_> { 
    pub fn bump_pumpfun_token(&mut self, token_address: &str, max_sol_cost: f64) -> Result<&mut Self, TransactionBuilderError>  {
        // Define accounts involved
        let token_account = address_to_pubkey(token_address)?;
        let user_keypair = self.payer_keypair;
        let user_account = user_keypair.pubkey();
        let associated_user_address = derive_associated_token_account_address(
            &user_account.to_string(), 
            &token_account.to_string(),
            token_program()
        )?;
        let associated_user_account = address_to_pubkey(&associated_user_address)?;
        let global_account = pumpfun_global_account();
        let pumpfun_fee_account = pumpfun_fee_account();
        let system_program = system_program();
        let token_program = token_program();
        let associated_token_program = associated_token_account_program();
        let rent_program = rent_program();
        let event_authority_account = pumpfun_event_authority_account();
        let pumpfun_program = pumpfun_program();
        
        // Get bonding curve and associated bonding curve accounts
        let (bonding_curve_account, bonding_state) = get_bonding_curve_account(self.client, token_address).expect("Unable to get bonding curve addresses. Please try again");
        let associated_bonding_curve_address = derive_associated_token_account_address(
            &bonding_curve_account.to_string(), 
            &token_account.to_string(),
            token_program
        )?;
        let associated_bonding_curve_account = address_to_pubkey(&associated_bonding_curve_address)?;
        
        // define buy accounts
        let buy_accounts = vec![
            AccountMeta::new_readonly(global_account, false),
            AccountMeta::new(pumpfun_fee_account, false),
            AccountMeta::new_readonly(token_account, false),
            AccountMeta::new(bonding_curve_account, false),
            AccountMeta::new(associated_bonding_curve_account, false),
            AccountMeta::new(associated_user_account, false),
            AccountMeta::new(user_account, true),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(rent_program, false),
            AccountMeta::new_readonly(event_authority_account, false),
            AccountMeta::new_readonly(pumpfun_program, false),
        ];

        // define sell accounts
        let sell_accounts = vec![
            AccountMeta::new_readonly(global_account, false),
            AccountMeta::new(pumpfun_fee_account, false),
            AccountMeta::new_readonly(token_account, false),
            AccountMeta::new(bonding_curve_account, false),
            AccountMeta::new(associated_bonding_curve_account, false),
            AccountMeta::new(associated_user_account, false),
            AccountMeta::new(user_account, true),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(associated_token_program, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(event_authority_account, false),
            AccountMeta::new_readonly(pumpfun_program, false),
        ];
        
        // get latest bonding curve account data
        let cost_per_token = calculate_token_price_in_sol(&bonding_state)
            .map_err(|err| TransactionBuilderError::BlockchainQueryError(err))?;
        let amount: f64 = (max_sol_cost / cost_per_token) * 0.8;
        let multiplier = 10_u64.pow(PUMP_TOKEN_DECIMALS);
        let amount_in_decimals: u64 = (amount * multiplier as f64).round() as u64;
        let max_sol_cost_in_lamports = (max_sol_cost * LAMPORTS_PER_SOL as f64) as u64;

        let mut buy_instruction_data = buy_instruction_data();
        buy_instruction_data.extend_from_slice(&amount_in_decimals.to_le_bytes());
        buy_instruction_data.extend_from_slice(&max_sol_cost_in_lamports.to_le_bytes());

        let mut sell_instruction_data = sell_instruction_data();
        sell_instruction_data.extend_from_slice(&amount_in_decimals.to_le_bytes());
        sell_instruction_data.extend_from_slice(&(0_u64).to_le_bytes());

        let buy_instruction = Instruction {
            program_id: pumpfun_program,
            accounts: buy_accounts.clone(),
            data: buy_instruction_data,
        };

        let sell_instruction = Instruction {
            program_id: pumpfun_program,
            accounts: sell_accounts.clone(),
            data: sell_instruction_data,
        };

        self.instructions.push(buy_instruction);
        self.instructions.push(sell_instruction);

        Ok(self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;
    use crate::{
        utils::{base58_to_keypair, create_rpc_client}, write_transactions::utils::simulate_transaction
    };

    const TOKEN_ADDRESS: &str = "CzAdDkkbRJnPYYjuwZ8T6tUxtD2ouCpZMXkJD7Rhpump";
    
    #[test]
    fn test_bump_token() {
        dotenv().ok();
        let private_key_string = env::var("PRIVATE_KEY_1").unwrap();
        let private_key = base58_to_keypair(&private_key_string).unwrap();

        let client = create_rpc_client("RPC_URL");

        let bump_pump_token_transaction = TransactionBuilder::new(&client, &private_key)
            .set_compute_units(111_111)
            .set_compute_limit(1_000_000)
            .create_associated_token_account_for_payer(TOKEN_ADDRESS, token_program())
            .unwrap()
            .bump_pumpfun_token(TOKEN_ADDRESS, 0.03)
            .unwrap()
            .build()
            .unwrap();
        
        let simulation_result = simulate_transaction(&client, bump_pump_token_transaction).expect("Failed to simulate transaction");
        assert!(simulation_result.error.is_none())
    }

    #[test]
    fn failing_test_bump_token_without_creating_associated_token_account() {
        dotenv().ok();
        let private_key_string = env::var("PRIVATE_KEY_1").unwrap();
        let private_key = base58_to_keypair(&private_key_string).unwrap();

        let client = create_rpc_client("RPC_URL");

        let bump_pump_token_transaction = TransactionBuilder::new(&client, &private_key)
            .set_compute_units(111_111)
            .set_compute_limit(1_000_000)
            .bump_pumpfun_token(TOKEN_ADDRESS, 0.03)
            .unwrap()
            .build()
            .unwrap();
        
        let simulation_result = simulate_transaction(&client, bump_pump_token_transaction).expect("Failed to simulate transaction");
        assert!(simulation_result.error.is_some())
    }
}
