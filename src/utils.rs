use std::env;
use std::error::Error;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;

// Convenience function shared across examples to load a Turnkey API key from the local `examples/.env` file, or from env vars.
pub fn load_api_key_from_env() -> Result<TurnkeyP256ApiKey, Box<dyn Error>> {
    // Load .env file from the example folder (examples/.env)
    let env_path = env::current_dir()?.join(".env"); // load .env from current directory

    if env_path.exists() {
        dotenvy::from_path(&env_path)?;
    } else {
        println!("No .env file found at {:?}", env_path);
        println!("Continuing because env might already be populated with the right variables");
    }

    let api_public_key =
        env::var("TURNKEY_API_PUBLIC_KEY").expect("cannot load TURNKEY_API_PUBLIC_KEY");
    let api_private_key =
        env::var("TURNKEY_API_PRIVATE_KEY").expect("cannot load TURNKEY_API_PRIVATE_KEY");

    Ok(TurnkeyP256ApiKey::from_strings(
        api_private_key,
        Some(api_public_key),
    )?)
}