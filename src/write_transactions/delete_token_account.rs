use spl_token::instruction::{close_account, burn};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{
        Keypair, 
        Signer
    },
    transaction::Transaction,
    compute_budget::ComputeBudgetInstruction,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    error::WriteTransactionError, 
    read_transactions::associated_token_account::get_multiple_associated_token_accounts, 
    utils::{address_to_pubkey, addresses_to_pubkeys},
    solana_programs::token_program
};


/// Deletes a token account and burns any remaining token balance
pub fn construct_burn_and_delete_token_accounts_transaction(
        client: &RpcClient, 
        signer_keypair: &str, 
        token_addresses: Vec<&str>, 
        rent_recipient: &str, 
        force_delete: Option<bool>, 
        compute_units: u32
    ) -> Result<Transaction, WriteTransactionError> {
    let force_delete = force_delete.unwrap_or(false);
    // token mint accounts
    let token_pubkeys = addresses_to_pubkeys(token_addresses);
    if token_pubkeys.len() == 0 {
        return Err(WriteTransactionError::InvalidAddress(solana_sdk::pubkey::ParsePubkeyError::Invalid))
    }

    // signer
    let signer_keypair = Keypair::from_base58_string(signer_keypair);
    let signer_pubkey = signer_keypair.pubkey();

    // rent recipient
    let fee_account = address_to_pubkey(rent_recipient)?;

    // Derive and retrieve the associated token accounts
    let associated_token_addresses: Vec<String> = token_pubkeys
        .iter()
        .map(|pubkey| {
            let associated_token_pubkey = get_associated_token_address(&signer_pubkey, &pubkey);
            associated_token_pubkey.to_string()
        })  
        .collect();
    
    let associated_token_accounts = get_multiple_associated_token_accounts(
            client, 
            associated_token_addresses.iter().map(|x| x.as_str()).collect()
        )?;

    let mut instructions = vec![];

    // Compute Budget: SetComputeUnitLimit
    let set_compute_unit_limit = ComputeBudgetInstruction::set_compute_unit_limit(compute_units);
    instructions.push(set_compute_unit_limit);

    // Compute Budget: SetComputeUnitPrice
    let set_compute_unit_price = ComputeBudgetInstruction::set_compute_unit_price(333_333);
    instructions.push(set_compute_unit_price); 

    for account in associated_token_accounts {
        let balance = account.token_amount;

        // if force delete is false, token accounts are not closed if there are still token balances within
        if !force_delete {
            if balance > 0 {
                return Err(WriteTransactionError::DeleteTokenAccountError(format!("{:?} still have balances within", account.pubkey)))
            }
        }

        let pubkey = address_to_pubkey(&account.pubkey)?;
        let mint_pubkey = address_to_pubkey(&account.mint_pubkey)?;
        // If the balance is greater than zero, create a burn instruction
        if balance > 0 {
            let burn_instruction = burn(
                &token_program(),
                &pubkey,
                &mint_pubkey,
                &signer_pubkey,
                &[],
                balance,
            )?;
            instructions.push(burn_instruction);
        }
        // Create the close account instruction
        let close_ix = close_account(
            &spl_token::id(),
            &pubkey,
            &fee_account,
            &signer_pubkey,
            &[],
        )?;

        instructions.push(close_ix);
    }

    // Create a transaction
    let mut transaction = Transaction::new_with_payer(
        &instructions,
        Some(&signer_pubkey),
    );

    // Get a recent blockhash
    let recent_blockhash = client.get_latest_blockhash().unwrap();
    transaction.sign(&[&signer_keypair], recent_blockhash);

    Ok(transaction)
}


#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;
    use crate::{
        read_transactions::associated_token_account::get_all_token_accounts, utils::create_rpc_client, write_transactions::utils::simulate_transaction
    };

    const RECIPIENT_ADDRESS: &str = "BdWaphz89Sf91ZHmqWus98jSvuiLNahwWE7bErTTqWmU";
    
    #[test]
    fn test_simulate_burn_and_delete_token_accounts() {
        dotenv().ok();
        let private_key = env::var("PRIVATE_KEY").expect("Cannot find PRIVATE_KEY env var");
        let client = create_rpc_client("RPC_URL");

        let wallet_token_accounts = get_all_token_accounts(
            &client, 
            "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5"
        ).expect("Unable to get token accounts");

        let token_addresses: Vec<String> = wallet_token_accounts
            .iter()
            .map(|account| {
                account.mint_pubkey.clone()
            })
            .collect();

        let burn_and_delete_transaction = construct_burn_and_delete_token_accounts_transaction(
            &client, 
            &private_key, 
            token_addresses.iter().map(|x| x.as_str()).collect(), 
            RECIPIENT_ADDRESS, 
            Some(true),
            2_000_000
        ).expect("Unable to construct transaction: {:?}");

        let simulation_result = simulate_transaction(&client, burn_and_delete_transaction).expect("Failed to simulate transaction");
        assert!(simulation_result.error.is_none());
        let logs = &simulation_result.transaction_logs;
        let instructions = &simulation_result.instructions;
        println!("Compute Units consumed: {:?}", &simulation_result.units_consumed);
        for (index, log) in logs.iter().enumerate() {
            println!("{:?}: {:}", index, log);
        }
        for instruction in instructions {
            println!("{:?}", instruction);
        }
    }
}
