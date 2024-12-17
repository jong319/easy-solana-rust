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

## Example

    use easy_solana::{
        create_rpc_client, 
        error::ReadTransactionError, 
        pumpfun::bonding_curve::{
            get_bonding_curve_account, 
            BondingCurveAccount, 
            calculate_token_price
        },
    }

    let client = create_rpc_client("https://api.mainnet-beta.solana.com");
    let pumpfun_token_address = "CzAdDkkbRJnPYYjuwZ8T6tUxtD2ouCpZMXkJD7Rhpump";
    let (bonding_curve_account, bonding_curve_data) = get_bonding_curve_account(&client, pumpfun_token_address).unwrap();
    let token_price_in_sol = calculate_token_price(&bonding_curve_data);


## License
EasySolana is licensed under MIT or Apache 2.0.
