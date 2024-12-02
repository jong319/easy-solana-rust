use thiserror::Error;
use solana_client::{
    client_error::ClientError as RpcClientError,
    client_error::ClientErrorKind as RpcClientErrorKind
};
use solana_sdk::{program_error::ProgramError, pubkey::ParsePubkeyError};

#[derive(Error, Debug)]
pub enum ReadTransactionError {
    #[error("Invalid Address")]
    InvalidAddress(#[from]ParsePubkeyError),
    #[error("RpcError")]
    RpcError(String),
    #[error("Failed to fetch data: {0}")]
    RpcForUserError(String),
    #[error("Unable to deserialize account data according to schema")]
    DeserializeError,
    #[error("Account does not exist")]
    AccountNotFound,
    #[error("Token has migrated or not from pumpfun")]
    BondingCurveError,
}

impl From<RpcClientError> for ReadTransactionError {
    fn from(err: RpcClientError) -> Self {
        match err.kind {
            RpcClientErrorKind::RpcError(solana_client::rpc_request::RpcError::ForUser(err)) => ReadTransactionError::RpcForUserError(err.to_string()) ,
            _ => ReadTransactionError::RpcError(err.to_string()), // Default fallback
        }
    }
}

#[derive(Error, Debug)]
pub enum WriteTransactionError {
    #[error("Invalid Address")]
    InvalidAddress(#[from]ParsePubkeyError),
    #[error("Error reading data: {0}")]
    QueryError(#[from]ReadTransactionError),
    #[error("Error: Token Account already created")]
    CreateTokenAccountError,
    #[error("Error: {0}")]
    DeleteTokenAccountError(String),
    #[error("Client Error: {0}")]
    RpcClientError(#[from]RpcClientError),
    #[error("Error interacting with Program: {0}")]
    ProgramError(#[from]ProgramError),
}

#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Client Error: {0}")]
    RpcClientError(#[from]RpcClientError),
    #[error("Logs unavailable")]
    NoLogsAvailable,
    #[error("Units consumed unavailable.")]
    NoUnitsConsumedAvailable,
    #[error("Inner Instructions unavailable")]
    NoInnerInstructionsAvailable,
}


#[derive(Error, Debug)]
pub enum KeypairGenerationError {
    #[error("Solana addresses should only contain characters: 1-9,A-H,J-N,P-Z,a-k,m-z")]
    InvalidPattern
}

