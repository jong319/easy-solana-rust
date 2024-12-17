use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, native_token::LAMPORTS_PER_SOL, 
    signer::{
        keypair::Keypair,
        Signer
    }, transaction::Transaction
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
    error::WriteTransactionError, read_transactions::associated_token_account::derive_associated_token_account_address, utils::address_to_pubkey};
use super::bonding_curve::{get_bonding_curve_account, calculate_token_price_in_sol};

/// Bumps token by combining a buy and sell instruction within one transaction
/// IMPT: check if the associated token account exists first
pub async fn construct_bump_pump_token_transaction(
    client: &RpcClient, 
    base58_keypair: &str, 
    token_address: &str, 
    max_sol_cost: f64,
    compute_limit: u32,
    compute_units: u64,
) -> Result<Transaction, WriteTransactionError> {
    // Define accounts involved
    let token_account = address_to_pubkey(&token_address)?;
    let user_keypair = Keypair::from_base58_string(&base58_keypair);
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
    let (bonding_curve_account, bonding_state) = get_bonding_curve_account(&client, token_address).expect("Unable to get bonding curve addresses. Please try again");
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

    // Compute Budget: SetComputeUnitLimit
    let set_compute_unit_limit = ComputeBudgetInstruction::set_compute_unit_limit(compute_limit);

    // Compute Budget: SetComputeUnitPrice
    let set_compute_unit_price = ComputeBudgetInstruction::set_compute_unit_price(compute_units);

    // get latest bonding curve account data
    let cost_per_token = calculate_token_price_in_sol(&bonding_state)?;
    
    let amount: f64 = (max_sol_cost / cost_per_token) * 0.8;
    let multiplier = 10_u64.pow(PUMP_TOKEN_DECIMALS);
    let amount_in_decimals: u64 = (amount * multiplier as f64).round() as u64;
    let max_sol_cost_in_lamports = (max_sol_cost * LAMPORTS_PER_SOL as f64) as u64;

    let mut buy_instruction_data = buy_instruction_data();
    buy_instruction_data.extend_from_slice(&amount_in_decimals.to_le_bytes());
    buy_instruction_data.extend_from_slice(&max_sol_cost_in_lamports.to_le_bytes());

    let mut sell_instruction_data = sell_instruction_data();
    sell_instruction_data.extend_from_slice(&amount_in_decimals.to_le_bytes());
    sell_instruction_data.extend_from_slice(&(0 as u64).to_le_bytes());

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

    let mut transaction = Transaction::new_with_payer(
        &[
            set_compute_unit_limit.clone(),
            set_compute_unit_price.clone(),
            buy_instruction,
            sell_instruction,
        ],
        Some(&user_account),
    );
    let recent_blockhash = client.get_latest_blockhash()?;

    transaction.sign(&[&user_keypair], recent_blockhash);

    Ok(transaction)
}



#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;
    use crate::{
        utils::create_rpc_client,
        write_transactions::utils::simulate_transaction
    };

    const TOKEN_ADDRESS: &str = "ArDKWeAhQj3LDSo2XcxTUb5j68ZzWg21Awq97fBppump";
    
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]  // Multi-threaded runtime
    async fn test_bump_token() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY_1").unwrap();
        let client = create_rpc_client("RPC_URL");

        // associated token account must already be created
        let create_token_account_transaction = construct_bump_pump_token_transaction(
            &client, 
            &private_key, 
            TOKEN_ADDRESS, 
            0.02, 
            2_000_000,
            111_111
        )
        .await
        .expect("Failed to construct create_token_account transaction");

        let simulation_result = simulate_transaction(&client, create_token_account_transaction).expect("Failed to simulate transaction");
        let logs = &simulation_result.transaction_logs;
        let instructions = &simulation_result.instructions;
        // assert!(simulation_result.error.is_none());
        println!("{:?}", simulation_result.error);
        println!("Compute Units consumed: {:?}", &simulation_result.units_consumed);
        for (index, log) in logs.iter().enumerate() {
            println!("{:?}: {:}", index, log);
        }
        for instruction in instructions {
            println!("{:?}", instruction);
        }
    }
}
