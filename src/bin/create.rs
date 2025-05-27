use std::env;
use std::error::Error;
use rand::{rng, Rng};
use turnkey_client::TurnkeyClient;
use turnkey_client::generated::{CreateWalletIntent, ExportWalletIntent};
use turnkey_client::generated::{
    immutable::common::v1::{AddressFormat, Curve, PathFormat},
    WalletAccountParams,
};
use turnkey_enclave_encrypt::{ExportClient, QuorumPublicKey};
use turnkey_prototype::utils::load_api_key_from_env;

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

    // Generate a random 6 character wallet name
    let random_wallet_name: String = (0..6)
        .map(|_| rng().random_range(b'A'..=b'Z') as char)
        .collect();
    
    println!("Creating wallet with name: {}", random_wallet_name);

    // Create a new wallet in the organization
    let create_wallet_result = client
        .create_wallet(
            organization_id.clone(),
            client.current_timestamp(),
            CreateWalletIntent {
                wallet_name: random_wallet_name,
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
    let wallet_id = create_wallet_result.wallet_id;
    println!("Ethereum address: {}", eth_address);
    println!("Wallet ID: {}", wallet_id);

    // Export Wallet
    let mut export_client = ExportClient::new(&QuorumPublicKey::production_signer());
    let export_wallet_result = client
        .export_wallet(
            organization_id.clone(),
            client.current_timestamp(),
            ExportWalletIntent {
                wallet_id: wallet_id.clone(),
                target_public_key: export_client.target_public_key()?,
                language: None,
            },
        )
        .await?;

    let export_bundle = export_wallet_result.export_bundle;
    let mnemonic_phrase =
        export_client.decrypt_wallet_mnemonic_phrase(export_bundle, organization_id.clone())?;

    assert_eq!(export_wallet_result.wallet_id, wallet_id);
    println!(
        "Wallet successfully exported: {} (Mnemonic phrase: {})",
        export_wallet_result.wallet_id,
        truncate_mnemonic_phrase(&mnemonic_phrase)
    );

    Ok(())
}

fn truncate_mnemonic_phrase(mnemonic_phrase: &str) -> String {
    let words : Vec<&str> = mnemonic_phrase.split_whitespace().collect();
    format!("{}...{}", words.first().unwrap(), words.last().unwrap())
}