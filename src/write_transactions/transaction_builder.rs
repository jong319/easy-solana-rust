use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, signer::{
        keypair::Keypair,
        Signer
    }, transaction::Transaction, instruction::Instruction
};

use crate::error::TransactionBuilderError;


pub struct TransactionBuilder<'a> {
    pub client: &'a RpcClient,
    pub payer_keypair: &'a Keypair,
    pub instructions: Vec<Instruction>,
    pub signing_keypairs: Vec<&'a Keypair>,
}

impl<'a> TransactionBuilder<'a> {
    pub fn new(client: &'a RpcClient, payer_keypair: &'a Keypair) -> Self {
        Self {
            client,
            payer_keypair,
            instructions: Vec::new(),
            signing_keypairs: Vec::new(),
        }
    }

    pub fn set_compute_limit(&mut self, limit: u32) -> &mut Self {
        let instruction = ComputeBudgetInstruction::set_compute_unit_limit(limit);
        self.instructions.push(instruction);
        self
    }

    pub fn set_compute_units(&mut self, units: u64) -> &mut Self {
        let instruction = ComputeBudgetInstruction::set_compute_unit_price(units);
        self.instructions.push(instruction);
        self
    }

    pub fn build(&self) -> Result<Transaction, TransactionBuilderError> {
        let mut transaction = Transaction::new_with_payer(&self.instructions, Some(&self.payer_keypair.pubkey()));
        let recent_blockhash = self.client.get_latest_blockhash().map_err(|_| TransactionBuilderError::LatestBlockhashError)?;
        let mut all_keypairs: Vec<&'a Keypair> = vec![self.payer_keypair];
        all_keypairs.append(&mut self.signing_keypairs.clone());
        transaction.sign(&all_keypairs, recent_blockhash);
        Ok(transaction)
    }
}