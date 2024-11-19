use solana_sdk::{
    pubkey::Pubkey,
    native_token::LAMPORTS_PER_SOL, 
};
use solana_client::rpc_response::RpcKeyedAccount;
use solana_account_decoder::UiAccountData;
use super::account::EasySolanaAccount;

/// Convenience interpretation of a Wallet account 
#[derive(Debug)]
pub struct WalletAccount {
    pub base: EasySolanaAccount,
    pub token_accounts: Vec<WalletToken>
}

#[derive(Debug)]
pub struct WalletToken {
    pub pubkey: Pubkey,
    pub sol_balance: f64,
    pub token_pubkey: Pubkey,
    pub owner_pubkey: Pubkey,
    pub token_balance: u64,
    pub token_decimals: u64,
    pub ui_amount: f64,
}

/// Used to parse a [`Vec<RpcKeyedAccount`] into a [`Vec<WalletToken`].
pub fn parse_token_accounts(token_accounts: Vec<RpcKeyedAccount>) -> Vec<WalletToken> {
    token_accounts
        .iter()
        .filter_map(|keyed_account| {
            // Parse the pubkey and SOL balance
            let pubkey = keyed_account.pubkey.parse::<Pubkey>().ok()?;
            let sol_balance = keyed_account.account.lamports as f64 / LAMPORTS_PER_SOL as f64;

            // Parse JSON data for token account
            if let UiAccountData::Json(parsed_data) = &keyed_account.account.data {
                // Ensure it's an SPL token account
                if parsed_data.program != "spl-token" {
                    return None;
                }

                // Extract and validate `info` fields
                let info = parsed_data.parsed.get("info")?;
                let token_pubkey = info.get("mint")?.as_str()?.parse::<Pubkey>().ok()?;
                let owner_pubkey = info.get("owner")?.as_str()?.parse::<Pubkey>().ok()?;
                let token_amount = info.get("tokenAmount")?;
                let token_balance = token_amount.get("amount")?.as_str()?.parse::<u64>().ok()?;
                let token_decimals = token_amount.get("decimals")?.as_u64().unwrap_or(0);
                let ui_amount = token_amount.get("uiAmount")?.as_f64().unwrap_or(0.0);

                // Construct WalletToken
                Some(WalletToken {
                    pubkey,
                    sol_balance,
                    token_pubkey,
                    owner_pubkey,
                    token_balance,
                    token_decimals,
                    ui_amount,
                })
            } else {
                None
            }
        })
        .collect()
}