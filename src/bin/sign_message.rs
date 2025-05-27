use std::env;
use std::error::Error;
use turnkey_client::TurnkeyClient;
use turnkey_client::generated::{
 SignRawPayloadIntentV2
};
use turnkey_client::generated::immutable::common::v1::{HashFunction, PayloadEncoding};
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

    let signature_result = client.sign_raw_payload(
        organization_id.clone(),
        client.current_timestamp(),
        SignRawPayloadIntentV2 {
            sign_with: "0xe3e439c29f35a519534342778486E423cA2431D3".to_string(),
            payload: "Hello, world!".to_string(),
            encoding: PayloadEncoding::TextUtf8,
            hash_function: HashFunction::Keccak256,
        },
    ).await?;

    println!(
        "Produced a new signature: r={}, s={}, v={}",
        signature_result.r, signature_result.s, signature_result.v,
    );

    Ok(())
}
