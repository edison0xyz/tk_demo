# Turnkey Prototype

Sample repo to try and demonstrate different functionalities for turnkey in rust.

## Setup

* Copy the env from `.env.example` to `.env`

## Running the Programs

This project contains multiple executable programs organized in the `src/bin/` directory.

### Run the default program
```bash
cargo run
```

### Run specific programs
```bash
# Create a new wallet with a random 6-character name
cargo run --bin create
```

### Available Programs

- **create**: Creates a new Ethereum wallet with a randomly generated 6-character name

## Adding New Programs

To add more functionality as separate programs:

1. Create a new file in `src/bin/` (e.g., `src/bin/list_wallets.rs`)
2. Add your main function and import shared utilities:
   ```rust
   use turnkey_prototype::utils::load_api_key_from_env;
   
   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // Your code here
       Ok(())
   }
   ```
3. Run it with: `cargo run --bin <filename_without_extension>`