use crate::{
    constants::pumpfun_accounts::pumpfun_program, 
    utils::address_to_pubkey, 
    error::ReadTransactionError
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use borsh::{BorshDeserialize, BorshSerialize};

const PUMP_CURVE_TOKEN_DECIMALS: u8 = 6;

// Bonding curve account data
#[derive(BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct BondingCurveAccount {
    pub unkown_value: u64,
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub total_token_supply: u64,
    pub complete: bool,
}

pub fn calculate_token_price(curve_state: &BondingCurveAccount) -> Result<f64, ReadTransactionError> {
    if curve_state.virtual_token_reserves == 0 || curve_state.virtual_sol_reserves == 0 {
        return Err(ReadTransactionError::BondingCurveError);
    }
    // Bonding curve prices are calculated by virtual sol / virtual token
    let virtual_sol_reserves = curve_state.virtual_sol_reserves as f64 / LAMPORTS_PER_SOL as f64;
    let virtual_token_reserves = curve_state.virtual_token_reserves as f64 / 10_f64.powi(PUMP_CURVE_TOKEN_DECIMALS as i32);
    let token_price_in_sol = virtual_sol_reserves / virtual_token_reserves;

    Ok(token_price_in_sol)
}

pub fn get_bonding_curve_account(client: &RpcClient, token_address: &str) -> Option<(Pubkey, BondingCurveAccount)> {
    let bonding_curve_address = get_bonding_curve_address(token_address).ok()?;
    let bonding_curve_account = address_to_pubkey(&bonding_curve_address).ok()?;

    if let Ok(account_data) = client.get_account_data(&bonding_curve_account) {
        if let Ok(bonding_curve_data) = BondingCurveAccount::deserialize(&mut account_data.as_slice()) {
            return Some((bonding_curve_account, bonding_curve_data))
        }
    }
    return None
}

fn get_bonding_curve_address(token_address: &str) -> Result<String, ReadTransactionError> {
    let token_account = address_to_pubkey(token_address)?;
    // Get bonding curve data
    let seed = b"bonding-curve";
    let (bonding_curve_account, _bump_seed) = Pubkey::find_program_address(
        &[seed, &token_account.to_bytes()],
        &pumpfun_program()
    );
    Ok(bonding_curve_account.to_string())
} 



