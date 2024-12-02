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
    // Display the transaction logs
    let logs = &simulation_result.logs.ok_or(SimulationError::NoLogsAvailable)?;
    // println!("Transaction Logs:");
    // for log in logs {
    //     println!("  - {}", log);
    // }

    // Display the compute units consumed
    let units_consumed = simulation_result.units_consumed.ok_or(SimulationError::NoUnitsConsumedAvailable)?;
    // println!("\nCompute Units Consumed: {}", units_consumed);
    
    let inner_instructions = &simulation_result.inner_instructions.ok_or(SimulationError::NoInnerInstructionsAvailable)?;
        // println!("\nInner Instructions:");

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

    // for (index, inner_instruction) in inner_instructions.iter().enumerate() {
    //     println!("\n  Inner Instruction {}:", index + 1);

    //     for instruction in &inner_instruction.instructions {
    //         match instruction {
    //             // Handle Compiled Instructions
    //             UiInstruction::Compiled(compiled) => {
    //                 println!("    Compiled Instruction: {:?}", compiled);
    //                 // Access compiled fields, e.g., compiled.program, compiled.data, etc.
    //             }

    //             // Handle Parsed Instructions (UiParsedInstruction)
    //             UiInstruction::Parsed(parsed) => {
    //                 println!("    Parsed Instruction:");
                    
    //                 // Match on the variants inside UiParsedInstruction
    //                 match parsed {
    //                     // Handle the Parsed variant of UiParsedInstruction
    //                     UiParsedInstruction::Parsed(parsed_instruction) => {
    //                         let program = &parsed_instruction.program;
    //                         let program_id = &parsed_instruction.program_id;
    //                         let stack_height = parsed_instruction.stack_height.unwrap_or(0);
    //                         println!("    Program: {}", program);
    //                         println!("    Program ID: {}", program_id);
    //                         println!("    Stack Height: {}", stack_height);

    //                         // Handle specific parsed data (e.g., parsed JSON)
    //                         if let Value::Object(info) = &parsed_instruction.parsed {
    //                             if let Some(instruction_type) = info.get("type") {
    //                                 let instruction_type = instruction_type.as_str().unwrap_or("Unknown");
    //                                 println!("      Instruction Type: {}", instruction_type);

    //                                 // Handle specific instruction types based on parsed data
    //                                 match instruction_type {
    //                                     "getAccountDataSize" => {
    //                                         if let Some(info) = info.get("info") {
    //                                             let mint = info.get("mint").unwrap_or(&Value::Null).to_string();
    //                                             println!("      Mint: {}", mint);
    //                                         }
    //                                     }
    //                                     "createAccount" => {
    //                                         if let Some(info) = info.get("info") {
    //                                             let new_account = info.get("newAccount").unwrap_or(&Value::Null).to_string();
    //                                             let lamports = info.get("lamports").unwrap_or(&Value::Null).to_string();
    //                                             let space = info.get("space").unwrap_or(&Value::Null).to_string();
    //                                             let owner = info.get("owner").unwrap_or(&Value::Null).to_string();
    //                                             println!("      New Account: {}", new_account);
    //                                             println!("      Lamports: {}", lamports);
    //                                             println!("      Space: {}", space);
    //                                             println!("      Owner: {}", owner);
    //                                         }
    //                                     }
    //                                     "initializeImmutableOwner" | "initializeAccount3" => {
    //                                         if let Some(info) = info.get("info") {
    //                                             let account = info.get("account").unwrap_or(&Value::Null).to_string();
    //                                             let mint = info.get("mint").unwrap_or(&Value::Null).to_string();
    //                                             let owner = info.get("owner").unwrap_or(&Value::Null).to_string();
    //                                             println!("      Account: {}", account);
    //                                             println!("      Mint: {}", mint);
    //                                             println!("      Owner: {}", owner);
    //                                         }
    //                                     }
    //                                     _ => {
    //                                         println!("      Unknown instruction type.");
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }

    //                     // Handle the PartiallyDecoded variant of UiParsedInstruction
    //                     UiParsedInstruction::PartiallyDecoded(partially_decoded) => {
    //                         println!("    Partially Decoded Instruction: {:?}", partially_decoded);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    // Handle any potential errors
    // if let Some(err) = &simulation_result.err {
    //     // println!("\nError in Simulation: {}", err);
    // };

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

pub fn send_transaction_checked(client: &RpcClient, transaction: Transaction) -> Result<Signature, WriteTransactionError> {
    let signature = client.send_and_confirm_transaction(
        &transaction,
    )?;
    
    Ok(signature)
}

pub fn extract_simulation_error(logs: &[String]) -> Option<String> {
    for i in 1..logs.len() {
        if logs[i].contains("failed") {
            return Some(logs[i - 1].clone());
        }
    }
    None
}