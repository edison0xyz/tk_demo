use std::env;
use std::error::Error;
use std::collections::HashMap;
use turnkey_client::TurnkeyClient;
use turnkey_client::generated::{
    SignRawPayloadIntentV2
};
use turnkey_client::generated::immutable::common::v1::{HashFunction, PayloadEncoding};
use turnkey_prototype::utils::load_api_key_from_env;
use serde_json::{json, Value};

// EIP-712 Domain Separator
#[derive(Debug)]
struct EIP712Domain {
    name: String,
    version: String,
    chain_id: u64,
    verifying_contract: String,
}

// EIP-2612 Permit structure for USDC
#[derive(Debug)]
struct Permit {
    owner: String,
    spender: String,
    value: u64,
    nonce: u64,
    deadline: u64,
}

// Simple EIP-712 hash implementation
fn encode_type(primary_type: &str, types: &HashMap<String, Vec<(String, String)>>) -> String {
    let mut result = format!("{}(", primary_type);
    
    if let Some(fields) = types.get(primary_type) {
        let field_strings: Vec<String> = fields.iter()
            .map(|(field_type, field_name)| format!("{} {}", field_type, field_name))
            .collect();
        result.push_str(&field_strings.join(","));
    }
    
    result.push(')');
    
    // Add referenced types
    let mut referenced_types = Vec::new();
    if let Some(fields) = types.get(primary_type) {
        for (field_type, _) in fields {
            if types.contains_key(field_type) && field_type != primary_type {
                referenced_types.push(field_type.clone());
            }
        }
    }
    
    referenced_types.sort();
    for ref_type in referenced_types {
        result.push_str(&encode_type(&ref_type, types));
    }
    
    result
}

fn hash_struct(struct_hash: &str, encoded_data: &str) -> String {
    use sha3::{Digest, Keccak256};
    
    let type_hash = Keccak256::digest(struct_hash.as_bytes());
    let combined = format!("{}{}", hex::encode(type_hash), encoded_data);
    let combined_bytes = hex::decode(combined).unwrap_or_default();
    let hash = Keccak256::digest(&combined_bytes);
    hex::encode(hash)
}

fn encode_data(data: &Value, struct_type: &str, types: &HashMap<String, Vec<(String, String)>>) -> String {
    let mut encoded = String::new();
    
    if let Some(fields) = types.get(struct_type) {
        for (field_type, field_name) in fields {
            if let Some(value) = data.get(field_name) {
                match field_type.as_str() {
                    "string" => {
                        if let Some(s) = value.as_str() {
                            use sha3::{Digest, Keccak256};
                            let hash = Keccak256::digest(s.as_bytes());
                            encoded.push_str(&hex::encode(hash));
                        }
                    },
                    "address" => {
                        if let Some(addr) = value.as_str() {
                            // Remove 0x prefix and pad to 32 bytes
                            let clean_addr = addr.trim_start_matches("0x");
                            encoded.push_str(&format!("{:0>64}", clean_addr));
                        }
                    },
                    "uint256" => {
                        if let Some(num) = value.as_u64() {
                            encoded.push_str(&format!("{:064x}", num));
                        }
                    },
                    _ => {
                        // Handle custom types (structs)
                        if types.contains_key(field_type) {
                            let nested_encoded = encode_data(value, field_type, types);
                            let nested_hash = hash_struct(&encode_type(field_type, types), &nested_encoded);
                            encoded.push_str(&nested_hash);
                        }
                    }
                }
            }
        }
    }
    
    encoded
}

fn create_eip712_hash(domain: &EIP712Domain, primary_type: &str, message: &Value) -> Result<String, Box<dyn Error>> {
    use sha3::{Digest, Keccak256};
    
    // Define the types
    let mut types = HashMap::new();
    
    // EIP712Domain type
    types.insert("EIP712Domain".to_string(), vec![
        ("string".to_string(), "name".to_string()),
        ("string".to_string(), "version".to_string()),
        ("uint256".to_string(), "chainId".to_string()),
        ("address".to_string(), "verifyingContract".to_string()),
    ]);
    
    // Permit type (EIP-2612)
    types.insert("Permit".to_string(), vec![
        ("address".to_string(), "owner".to_string()),
        ("address".to_string(), "spender".to_string()),
        ("uint256".to_string(), "value".to_string()),
        ("uint256".to_string(), "nonce".to_string()),
        ("uint256".to_string(), "deadline".to_string()),
    ]);
    
    // Create domain separator
    let domain_data = json!({
        "name": domain.name,
        "version": domain.version,
        "chainId": domain.chain_id,
        "verifyingContract": domain.verifying_contract
    });
    
    let domain_type_hash = encode_type("EIP712Domain", &types);
    let domain_encoded = encode_data(&domain_data, "EIP712Domain", &types);
    let domain_separator = hash_struct(&domain_type_hash, &domain_encoded);
    
    // Create struct hash
    let message_type_hash = encode_type(primary_type, &types);
    let message_encoded = encode_data(message, primary_type, &types);
    let struct_hash = hash_struct(&message_type_hash, &message_encoded);
    
    // Create final hash
    let prefix = "1901"; // EIP-191 prefix for structured data
    let final_data = format!("{}{}{}", prefix, domain_separator, struct_hash);
    let final_bytes = hex::decode(final_data)?;
    let final_hash = Keccak256::digest(&final_bytes);
    
    Ok(hex::encode(final_hash))
}

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

    // Check environment for the organization ID
    if let Ok(org_id) = env::var("TURNKEY_ORGANIZATION_ID") {
        println!("Current organization ID: {}", org_id);
    } else {
        println!("TURNKEY_ORGANIZATION_ID not found in environment variables");
        println!("Add it to your .env file or environment");
    }

    // Create EIP-712 domain for USDC (Ethereum mainnet)
    let domain = EIP712Domain {
        name: "USD Coin".to_string(),
        version: "2".to_string(),
        chain_id: 1,
        verifying_contract: "0xA0b86a33E6441E6C7D3E4C7C5C6C8C8C8C8C8C8C".to_string(), // Example USDC-like contract address
    };

    // Create permit data for 1000 USDC (6 decimals)
    let permit_amount = 1000_000_000u64; // 1000 USDC with 6 decimals
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let deadline = current_time + 3600; // 1 hour from now

    let message = json!({
        "owner": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826",
        "spender": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB",
        "value": permit_amount,
        "nonce": 0,
        "deadline": deadline
    });

    println!("\n=== EIP-2612 USDC Permit Example ===");
    println!("Domain: {:?}", domain);
    println!("Permit Details:");
    println!("  Owner: {}", message["owner"]);
    println!("  Spender: {}", message["spender"]);
    println!("  Amount: {} USDC", permit_amount as f64 / 1_000_000.0);
    println!("  Nonce: {}", message["nonce"]);
    println!("  Deadline: {} (Unix timestamp)", deadline);
    println!("\nMessage: {}", serde_json::to_string_pretty(&message)?);

    // Create EIP-712 hash
    let eip712_hash = create_eip712_hash(&domain, "Permit", &message)?;
    println!("\nEIP-712 Hash: 0x{}", eip712_hash);

    // Sign the EIP-712 hash using Turnkey
    // Format the hash with 0x prefix for Turnkey
    let hash_with_prefix = format!("0x{}", eip712_hash);
    
    let signature_result = client.sign_raw_payload(
        organization_id.clone(),
        client.current_timestamp(),
        SignRawPayloadIntentV2 {
            sign_with: "0xe3e439c29f35a519534342778486E423cA2431D3".to_string(),
            payload: hash_with_prefix,
            encoding: PayloadEncoding::TextUtf8,
            hash_function: HashFunction::Keccak256,
        },
    ).await?;

    println!(
        "\n=== Signature Result ===\nProduced EIP-2612 Permit signature: r={}, s={}, v={}",
        signature_result.r, signature_result.s, signature_result.v,
    );

    // Display the complete typed data structure that would be sent to a wallet
    let typed_data = json!({
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "version", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ],
            "Permit": [
                { "name": "owner", "type": "address" },
                { "name": "spender", "type": "address" },
                { "name": "value", "type": "uint256" },
                { "name": "nonce", "type": "uint256" },
                { "name": "deadline", "type": "uint256" }
            ]
        },
        "primaryType": "Permit",
        "domain": {
            "name": domain.name,
            "version": domain.version,
            "chainId": domain.chain_id,
            "verifyingContract": domain.verifying_contract
        },
        "message": message
    });

    println!("\n=== Complete EIP-2612 Typed Data ===");
    println!("{}", serde_json::to_string_pretty(&typed_data)?);

    Ok(())
} 