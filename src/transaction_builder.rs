use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, native_token::LAMPORTS_PER_SOL, signer::{
        keypair::Keypair,
        Signer
    }, system_instruction, transaction::Transaction, instruction::Instruction
};
use spl_associated_token_account::instruction::create_associated_token_account;

use crate::{
    constants::solana_programs::token_program, error::{TransactionBuilderError, WriteTransactionError}, read_transactions::associated_token_account::derive_associated_token_account_address, utils::address_to_pubkey
};


struct TransactionBuilder<'a> {
    client: &'a RpcClient,
    payer_keypair: &'a Keypair,
    instructions: Vec<Instruction>,
    signing_keypairs: Vec<&'a Keypair>,
}

impl<'a> TransactionBuilder<'a> {
    fn new(client: &'a RpcClient, payer_keypair: &'a Keypair) -> Self {
        Self {
            client,
            payer_keypair,
            instructions: Vec::new(),
            signing_keypairs: Vec::new(),
        }
    }

    fn set_compute_limit(&mut self, limit: u32) -> &mut Self {
        let instruction = ComputeBudgetInstruction::set_compute_unit_limit(limit);
        self.instructions.push(instruction);
        self
    }

    fn set_compute_units(&mut self, units: u64) -> &mut Self {
        let instruction = ComputeBudgetInstruction::set_compute_unit_price(units);
        self.instructions.push(instruction);
        self
    }

    fn transfer_sol(&mut self, amount: f64, from_keypair: &'a Keypair, destination_address: &str) -> Result<&mut Self, TransactionBuilderError> {
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

    fn create_associated_token_account_for_payer(&mut self, token_address: &str) -> Result<&mut Self, TransactionBuilderError> {
        // Payer account
        let payer_account = self.payer_keypair.pubkey();

        // associated user 
        let associated_payer_address = derive_associated_token_account_address(
            &payer_account.to_string(),
            token_address
        ).map_err(|err| TransactionBuilderError::InvalidAddress(err))?;
        let associated_user_account = address_to_pubkey(&associated_payer_address)?;

        // Token account
        let token_account = address_to_pubkey(token_address)?;

        // token program
        let token_program = token_program();

        let create_associated_token_account_instruction = create_associated_token_account(
            &payer_account,
            &payer_account,
            &token_account,
            &token_program,
        );
        self.instructions.push(create_associated_token_account_instruction);
        Ok(self)
    }

    fn build(&self) -> Result<Transaction, TransactionBuilderError> {
        let mut transaction = Transaction::new_with_payer(&self.instructions, Some(&self.payer_keypair.pubkey()));
        let recent_blockhash = self.client.get_latest_blockhash().map_err(|_| TransactionBuilderError::LatestBlockhashError)?;
        let mut all_keypairs: Vec<&'a Keypair> = vec![self.payer_keypair];
        all_keypairs.append(&mut self.signing_keypairs.clone());
        transaction.sign(&all_keypairs, recent_blockhash);
        Ok(transaction)
    }
}


