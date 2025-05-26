use std::env;
use std::error::Error;
use turnkey_client::TurnkeyClient;
use turnkey_client::generated::{CreateWalletIntent};
use turnkey_client::generated::{
    immutable::common::v1::{AddressFormat, Curve, PathFormat},
    WalletAccountParams,
};

mod utils;
use utils::load_api_key_from_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load the API key from environment
    let api_key = load_api_key_from_env()?;

    // Get organization ID from env
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");

    // Create a new Turnkey client
    let client = TurnkeyClient::builder().api_key(api_key).build()?;

    // Get the current timestamp (this proves the client works)
    let timestamp = client.current_timestamp();
    println!("Client initialized successfully!");
    println!("Current timestamp: {}", timestamp);

    // Note: The organization ID is typically passed in requests rather than queried
    // Check environment for the organization ID
    if let Ok(org_id) = env::var("TURNKEY_ORGANIZATION_ID") {
        println!("Current organization ID: {}", org_id);
    } else {
        println!("TURNKEY_ORGANIZATION_ID not found in environment variables");
        println!("Add it to your .env file or environment");
    }

    // Create a new wallet in the organization
    let create_wallet_result = client
        .create_wallet(
            organization_id.clone(),
            client.current_timestamp(),
            CreateWalletIntent {
                wallet_name: "New wallet".to_string(),
                accounts: vec![WalletAccountParams {
                    curve: Curve::Secp256k1,
                    path_format: PathFormat::Bip32,
                    path: "m/44'/60'/0'/0".to_string(),
                    address_format: AddressFormat::Ethereum,
                }],
                mnemonic_length: None, // Let that be the default
            },
        )
        .await?;
    assert_eq!(create_wallet_result.addresses.len(), 1);
    let eth_address = create_wallet_result.addresses.first().unwrap();
    println!("Ethereum address: {}", eth_address);

    Ok(())
}
