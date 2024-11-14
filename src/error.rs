use thiserror::Error;
use solana_client::client_error::ClientError as RpcClientError;

#[derive(Error, Debug)]
pub enum AccountReaderError {
    #[error("Failed to fetch multiple accounts: {0}")]
    RpcClientError(#[from] RpcClientError),
}

#[derive(Error, Debug)]
pub enum KeypairGenerationError {
    #[error("Solana addresses should only contain characters: 1-9,A-H,J-N,P-Z,a-k,m-z")]
    InvalidPattern
}

