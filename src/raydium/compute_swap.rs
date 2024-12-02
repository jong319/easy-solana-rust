use reqwest::Error as ReqwestError;
use serde::Deserialize;
use thiserror::Error;

/// Custom error type for the function
#[derive(Error, Debug)]
pub enum RaydiumSwapError {
    #[error("Invalid Response: {0}")]
    InvalidResponse(String),
    #[error("Request Error: {0}")]
    RequestError(#[from] ReqwestError),
}

/// Response structure for the Raydium API
#[derive(Deserialize, Debug)]
#[allow(unused)]
struct RaydiumPriceResponse {
    id: String,
    success: bool,
    version: String, // 'V0' | 'V1'
    #[serde(rename = "openTime")]
    open_time: Option<String>,
    msg: Option<String>,
    data: Option<SwapData>,
}

/// Data structure for the swap details
#[derive(Deserialize, Debug)]
#[allow(unused)]
struct SwapData {
    #[serde(rename = "swapType")]
    swap_type: String, // 'BaseIn' | 'BaseOut'
    #[serde(rename = "inputMint")]
    input_mint: String,
    #[serde(rename = "inputAmount")]
    input_amount: String,
    #[serde(rename = "outputMint")]
    output_mint: String,
    #[serde(rename = "outputAmount")]
    output_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    other_amount_threshold: String,
    #[serde(rename = "slippageBps")]
    slippage_bps: i32,
    #[serde(rename = "priceImpactPct")]
    price_impact_pct: f64,
}

/// Gets the output amount of tokens from a Raydium swap.
pub async fn get_raydium_swap_output(
    input_mint: &str,
    input_mint_decimals: u32,
    input_amount: f64,
    output_mint: &str,
    output_mint_decimals: u32,
    slippage: f64,
) -> Result<f64, RaydiumSwapError> {
    // Compute input amount with decimals
    let input_amount_with_decimals = input_amount * 10_f64.powi(input_mint_decimals as i32);
    let slippage_bps = slippage * 100.0;

    // Construct URL
    let url = format!(
        "https://transaction-v1.raydium.io/compute/swap-base-in?inputMint={}&outputMint={}&amount={}&slippageBps={}&txVersion=V0",
        input_mint, output_mint, input_amount_with_decimals, slippage_bps
    );

    // Make HTTP request
    let response: RaydiumPriceResponse = reqwest::get(&url).await?.json().await?;

    // Validate response and extract output amount
    if let Some(data) = response.data {
        let output_amount = data.output_amount.parse::<f64>()
            .map_err(|_| RaydiumSwapError::InvalidResponse("Failed to parse output amount".to_string()))?;
        Ok(output_amount / 10_f64.powi(output_mint_decimals as i32))
    } else if let Some(msg) = response.msg {
        Err(RaydiumSwapError::InvalidResponse(msg))
    } else {
        Err(RaydiumSwapError::InvalidResponse("Unknown error".to_string()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const SOLANA_CONTRACT_ADDRESS: &str = "So11111111111111111111111111111111111111112";
    const USDC_TOKEN_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    #[tokio::test]
    async fn test_get_solana_price() {
        let solana_price = get_raydium_swap_output(
            SOLANA_CONTRACT_ADDRESS,
            9,
            1.0,
            USDC_TOKEN_ADDRESS,
            6,
            1.0
        ).await;
        println!("{:?}", solana_price)
    }
}
