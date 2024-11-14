use thiserror::Error;
use solana_client::client_error::ClientError as RpcClientError;

#[derive(Error, Debug)]
pub enum AccountReaderError {
    #[error("Failed to fetch multiple accounts: {0}")]
    RpcClientError(#[from] RpcClientError),
}