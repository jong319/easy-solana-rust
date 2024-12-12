use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, native_token::LAMPORTS_PER_SOL, signer::{
        keypair::Keypair,
        Signer
    }, system_instruction, transaction::Transaction
};
use spl_associated_token_account::instruction::create_associated_token_account;

use crate::{
    error::WriteTransactionError, 
    utils::address_to_pubkey, 
    read_transactions::associated_token_account::derive_associated_token_account_address,
    constants::solana_programs::token_program
};


pub fn construct_create_token_account_transaction(
        client: &RpcClient, 
        signer_keypair: &str, 
        token_address: &str, 
        fee: f64, 
        fee_account: &str, 
        referral_fee: f64, 
        referral_fee_account: &str, 
        compute_limit: u32,
        compute_units: u64
    ) -> Result<Transaction, WriteTransactionError> {
    // token mint account - mint 
    let token_account = address_to_pubkey(token_address)?;

    // user
    let user_keypair = Keypair::from_base58_string(&signer_keypair);
    let user_account = user_keypair.pubkey();

    // associated user 
    let associated_user_address = derive_associated_token_account_address(
        &user_account.to_string(),
        token_address
    )?;
    let associated_user_account = address_to_pubkey(&associated_user_address)?;

    // token program
    let token_program = token_program();

    // Instructions
    let mut instructions = vec![];

     // Compute Budget: SetComputeUnitLimit
     let set_compute_unit_limit = ComputeBudgetInstruction::set_compute_unit_limit(compute_limit);
     instructions.push(set_compute_unit_limit);
 
     // Compute Budget: SetComputeUnitPrice
     let set_compute_unit_price = ComputeBudgetInstruction::set_compute_unit_price(compute_units);
     instructions.push(set_compute_unit_price);

    if fee > 0.0 {
        // Balance is in lamports, 1 SOL = 1_000_000_000 lamports
        let fee_in_lamports = (fee * LAMPORTS_PER_SOL as f64) as u64;

        // Creating the transfer sol instruction
        let fee_account = address_to_pubkey(fee_account)?;
        let fee_instruction = system_instruction::transfer(&user_account, &fee_account, fee_in_lamports);
        instructions.push(fee_instruction);
    }

    if referral_fee > 0.0 {
        // Balance is in lamports, 1 SOL = 1_000_000_000 lamports
        let referral_fee_in_lamports = (referral_fee * LAMPORTS_PER_SOL as f64) as u64;

        // Creating the transfer sol instruction
        let referral_fee_account = address_to_pubkey(referral_fee_account)?;
        let referral_fee_instruction = system_instruction::transfer(&user_account, &referral_fee_account, referral_fee_in_lamports);
        instructions.push(referral_fee_instruction);
    }

    // Check if the associated token account exists
    let account_info = client.get_account(&associated_user_account);
    if account_info.is_err() {
        let create_associated_token_account_instruction = create_associated_token_account(
            &user_account,
            &user_account,
            &token_account,
            &token_program,
        );
        instructions.push(create_associated_token_account_instruction);
    }

    let mut transaction = Transaction::new_with_payer(
        &instructions,
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

    const PNUT_TOKEN_ADDRESS: &str = "2qEHjDLDLbuBgRYvsxhc5D6uDWAivNFZGan56P1tpump";
    
    #[test]
    fn test_simulate_create_token_account() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY").unwrap();
        let fee_account = "BdWaphz89Sf91ZHmqWus98jSvuiLNahwWE7bErTTqWmU";
        let referral_fee_account = "BdWaphz89Sf91ZHmqWus98jSvuiLNahwWE7bErTTqWmU";
        let client = create_rpc_client("RPC_URL");

        let create_token_account_transaction = construct_create_token_account_transaction(
            &client, 
            &private_key, 
            PNUT_TOKEN_ADDRESS, 
            0.001, 
            fee_account,
            0.0,
            referral_fee_account,
            2_000_000,
            111_111
        ).expect("Failed to construct create_token_account transaction");

        let simulation_result = simulate_transaction(&client, create_token_account_transaction).expect("Failed to simulate transaction");
        let logs = &simulation_result.transaction_logs;
        let instructions = &simulation_result.instructions;
        assert!(simulation_result.error.is_none());
        println!("Compute Units consumed: {:?}", &simulation_result.units_consumed);
        for (index, log) in logs.iter().enumerate() {
            println!("{:?}: {:}", index, log);
        }
        for instruction in instructions {
            println!("{:?}", instruction);
        }
    }
}
