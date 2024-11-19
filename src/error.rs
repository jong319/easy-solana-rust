use thiserror::Error;
use solana_client::client_error::ClientError as RpcClientError;
use solana_sdk::pubkey::ParsePubkeyError;

#[derive(Error, Debug)]
pub enum AccountReaderError {
    #[error("Invalid Address")]
    InvalidAddress(#[from]ParsePubkeyError),
    #[error("Failed to fetch account: {0}")]
    RpcClientError(#[from] RpcClientError),
    #[error("Unable to deserialize account data according to schema")]
    DeserializeError,
    #[error("Account does not exist")]
    AccountNotFound
}

#[derive(Error, Debug)]
pub enum KeypairGenerationError {
    #[error("Solana addresses should only contain characters: 1-9,A-H,J-N,P-Z,a-k,m-z")]
    InvalidPattern
}

