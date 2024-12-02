use solana_program::system_instruction;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    signature::{
        Keypair, 
        Signer
    },
    transaction::Transaction,
    compute_budget::ComputeBudgetInstruction,
};
use solana_client::rpc_client::RpcClient;

use crate::{error::WriteTransactionError, utils::address_to_pubkey};


pub fn construct_transfer_sol_transaction(client: &RpcClient, signer_keypair: &str, recipient_address: &str, amount: f64, compute_units: u32) -> Result<Transaction, WriteTransactionError> {
    let sender_keypair = Keypair::from_base58_string(signer_keypair);
    let sender_pubkey = sender_keypair.pubkey();
    let recipient_pubkey = address_to_pubkey(recipient_address)?;
    
    // Amount should be in lamports
    let amount_in_lamports = (amount * LAMPORTS_PER_SOL as f64) as u64;

    // Instructions
    let mut instructions = vec![];

    // Compute Budget: SetComputeUnitLimit
    let set_compute_unit_limit = ComputeBudgetInstruction::set_compute_unit_limit(compute_units);
    instructions.push(set_compute_unit_limit);

    // Compute Budget: SetComputeUnitPrice
    let set_compute_unit_price = ComputeBudgetInstruction::set_compute_unit_price(333_333);
    instructions.push(set_compute_unit_price);

    // Transfer sol instruction
    let transfer_instruction = system_instruction::transfer(&sender_pubkey, &recipient_pubkey, amount_in_lamports);
    instructions.push(transfer_instruction);

    // Putting the transfer sol instruction into a transaction
    let recent_blockhash = client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(&instructions, Some(&sender_pubkey), &[&sender_keypair], recent_blockhash);

    Ok(txn)
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

    const RECIPIENT_ADDRESS: &str = "BdWaphz89Sf91ZHmqWus98jSvuiLNahwWE7bErTTqWmU";
    
    // #[tokio::test(flavor = "multi_thread", worker_threads = 2)]  // Multi-threaded runtime
    #[test]
    fn test_simulate_transfer_sol() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY").expect("Cannot find PRIVATE_KEY env var");
        let client = create_rpc_client("RPC_URL");

        let transfer_sol_transaction = construct_transfer_sol_transaction(
            &client, 
            &private_key, 
            RECIPIENT_ADDRESS, 
            0.001, 
            450
        ).expect("Failed to construct create_token_account transaction");

        let simulation_result = simulate_transaction(&client, transfer_sol_transaction).expect("Failed to simulate transaction");
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
