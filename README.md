# EasySolana

EasySolana simplifies querying data and writing transactions on the Solana
blockchain network. It takes multistep procedures like fetching, deserializing,
simulating and signing transactions into simple functions for developers to utilize.
In particular, it integrates seamlessly with Pump.fun programs and allows price queries, 
buy, sell and creation transactions. 

## Features

- Querying account data 
- Building custom transactions
- Simulating transaction
- Sending transaction
- Integration with Pump.fun methods

## Examples

### Creating RPC client
All read and write functions to solana require an RPC client. Use the `create_rpc_client` function to create a client. The function takes in either an RPC URL, or an environment variable with your personal RPC URL.
```
use dotenv::dotenv;
use easy_solana::create_rpc_client;

// Load environment variables
dotenv().ok();

let client_with_rpc_url = create_rpc_client("https://api.mainnet-beta.solana.com");
let client_with_env_var = create_rpc_client("ENV_VAR_RPC_URL");
```

### Querying account data
The below example shows how to derive an associated token account address from a wallet and token address. 

This can be used to query the associated token account for variables like token balance, token decimals, token supply, mint authority and more.

Token name, ticker and URI can also be queried using the `get_metadata_of_token` function. 
```
// Load environment variables
dotenv().ok();

// Create RPC client
let client = create_rpc_client("RPC_URL");

// Solana addresses
let wallet_address = "ACTC9k56rLB1Z6cUBKToptXrEXussVkiASJeh8p74Fa5";
let associated_token_account_address= "7geCZYWHtghvWj11sb7exvu4uMANfhvGvEvVRRZ8GmSd";
let token_address = "ArDKWeAhQj3LDSo2XcxTUb5j68ZzWg21Awq97fBppump";

// Derive the associated token account address using the wallet and token address.
let derived_associated_token_account_address = derive_associated_token_account_address(
    wallet_address, 
    token_address, 
    token_program()
).unwrap();
assert_eq!(associated_token_account_address.to_string(), derived_associated_token_account_address);

// Query associated token account details
let associated_token_account = get_associated_token_account(
    &client, 
    associated_token_account_address
).unwrap();
print!(
"Associated Token Account
Pubkey: {}
Wallet Owner: {}
Token Address: {}
Token Balance: {}
Token Supply: {}
", associated_token_account.pubkey, 
associated_token_account.owner_pubkey,
associated_token_account.mint_pubkey, 
associated_token_account.token_ui_amount, 
associated_token_account.mint_supply);

// Query token name and ticker
let token_metadata = get_metadata_of_token(&client, token_address).unwrap();
print!("Token Name: {}, Token Ticker: {}", token_metadata.data.name, token_metadata.data.symbol);
```

### Building Custom Transactions
It is vital for applications to be able to customise the transactions. The below shows an example of creating an associated token account and taking a small fee by transferring a fixed amount of SOL to a fee account and a referral account.
```
use dotenv::dotenv;
use std::env;
use easy_solana::write_transactions::{
    transaction_builder::TransactionBuilder,
    utils::{
        simulate_transaction,
        send_transaction_unchecked,
        send_and_confirm_transaction
    }
};

// Load environment variables
dotenv().ok();

// Get private key string via environment variables
let private_key_string = env::var("PRIVATE_KEY_1").unwrap();

// Create RPC client
let client = create_rpc_client("RPC_URL");

// Customised transaction
let create_token_account_simulation_transaction = TransactionBuilder::new(&client, &private_key_string)
    // set priority fee
    .set_compute_units(50_000) 
    // max compute limit in solana is ~2_000_000, recommended to set a higher limit before simulation 
    .set_compute_limit(1_000_000) 
    // transfer to fee account
    .transfer_sol(0.018, &private_key, "FEE_WALLET_ADDRESS") 
    .unwrap()
    // transfer to referral account
    .transfer_sol(0.002, &private_key, "REFERRAL_WALLET_ADDRESS") 
    .unwrap()
    // create associated token account
    .create_associated_token_account_for_payer ("test_token_address", token_program())
    .unwrap()
    .build()
    .unwrap();
```

### Simulate Transactions
```
// Always simulate transaction for compute limit and errors
let simulation_result = simulate_transaction(&client, create_token_account_simulation_transaction).expect("Failed to simulate transaction");

// Check for simulation errors
if simulation_result.error.is_some() {
    println!("{:?}", simulation_result.transaction_logs)
}

// Get compute limit, lower compute limits are prioritised in Solana
let simulated_compute_limit = simulation_result.units_consumed;
```

### Send Transactions
```
// Recreate transaction with simulated compute limit
let create_token_account_transaction = TransactionBuilder::new(&client, &private_key)
    .set_compute_units(50_000) 
    .set_compute_limit(simulated_compute_limit) 
    .transfer_sol(0.018, &private_key, "FEE_WALLET_ADDRESS") 
    .unwrap()
    .transfer_sol(0.002, &private_key, "REFERRAL_WALLET_ADDRESS") 
    .unwrap()
    .create_associated_token_account_for_payer ("USDC_TOKEN_ADDRESS", token_program())
    .unwrap()
    .build()
    .unwrap();

// Sends an unchecked transaction, skip preflight, faster confirmation but failed transactions can occur
let signature = send_transaction_unchecked(&client, create_token_account_transaction).unwrap();

// Sends and confirm transaction, long waiting function. 
let signature = send_and_confirm_transaction(&client, create_token_account_transaction).unwrap();
```


## License
EasySolana is licensed under MIT or Apache 2.0.
