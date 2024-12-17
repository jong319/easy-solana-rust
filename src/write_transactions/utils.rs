use solana_client::{
    rpc_response::RpcSimulateTransactionResult, 
    rpc_client::RpcClient, 
    rpc_config::{RpcSimulateTransactionConfig, RpcSendTransactionConfig}
};
use solana_sdk::{
    signature::Signature, transaction::Transaction, transaction::TransactionError
};
use solana_transaction_status_client_types::{UiInstruction, UiParsedInstruction};
use serde_json::{Value, Map};
use crate::error::{WriteTransactionError, SimulationError};

#[derive(Debug)]
pub struct SimulationResult {
    pub transaction_logs: Vec<String>,
    pub units_consumed: u32,
    pub instructions: Vec<ParsedInstruction>,
    pub error: Option<TransactionError>
}

#[derive(Debug)]
pub struct ParsedInstruction {
    pub program: String,
    pub program_id: String, 
    pub info: Map<String, Value>
}

pub fn simulate_transaction(client: &RpcClient, transaction: Transaction) -> Result<SimulationResult, SimulationError> {
    let simulation_result = client.simulate_transaction_with_config(
        &transaction, 
        RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: true,
            commitment: None,
            encoding: None,
            accounts: None,
            min_context_slot: None,
            inner_instructions: true
        }
    )?;
    
    parse_simulation_result(simulation_result.value)
}

fn parse_simulation_result(simulation_result: RpcSimulateTransactionResult) -> Result<SimulationResult, SimulationError> {
    let logs = &simulation_result.logs.ok_or(SimulationError::NoLogsAvailable)?;

    let units_consumed = simulation_result.units_consumed.ok_or(SimulationError::NoUnitsConsumedAvailable)?;
    
    let inner_instructions = &simulation_result.inner_instructions.ok_or(SimulationError::NoInnerInstructionsAvailable)?;

    let parsed_instructions : Vec<ParsedInstruction> = inner_instructions
    .iter()
    .flat_map(|inner_instruction| {
        inner_instruction.instructions.iter().filter_map(|instruction| {
            if let UiInstruction::Parsed(parsed) = instruction {
                if let UiParsedInstruction::Parsed(parsed_instruction) = parsed {
                    let program = parsed_instruction.program.clone();
                    let program_id = parsed_instruction.program_id.clone();

                    // Ensure parsed_instruction.parsed is an Object and contains "info"
                    if let Value::Object(info_object) = &parsed_instruction.parsed {
                        if let Some(Value::Object(info)) = info_object.get("info") {
                            return Some(ParsedInstruction {
                                program,
                                program_id,
                                info: info.clone(),
                            });
                        }
                    }
                }
            }
            None
        })
    })
    .collect();

    Ok(SimulationResult {
        transaction_logs: logs.to_vec(),
        units_consumed: units_consumed as u32,
        instructions: parsed_instructions,
        error: simulation_result.err
    })
}

pub fn send_transaction_unchecked(client: &RpcClient, transaction: Transaction) -> Result<Signature, WriteTransactionError> {
    let signature = client.send_transaction_with_config(
        &transaction,
        RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
            max_retries: None,
            min_context_slot: None
        }
    )?;
    
    Ok(signature)
}

pub fn send_and_confirm_transaction(client: &RpcClient, transaction: Transaction) -> Result<Signature, WriteTransactionError> {
    let signature = client.send_and_confirm_transaction(
        &transaction,
    )?;
    
    Ok(signature)
}