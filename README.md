# Turnkey Organization ID Query Script

This script queries your current organization ID using the Turnkey client.

## Setup

1. Create the examples directory and .env file:
   ```bash
   mkdir -p examples
   ```

2. Create `examples/.env` file with your Turnkey credentials:
   ```env
   # Your Turnkey organization ID
   TURNKEY_ORGANIZATION_ID=your_organization_id_here
   
   # Your Turnkey API public key 
   TURNKEY_API_PUBLIC_KEY=your_api_public_key_here
   
   # Your Turnkey API private key
   TURNKEY_API_PRIVATE_KEY=your_api_private_key_here
   ```

3. Run the script:
   ```bash
   cargo run
   ```

## What the script does

The script will:
- Load your API credentials from the environment
- Initialize the Turnkey client
- Display your current organization ID
- Show the current timestamp to verify the client connection

## Notes

The organization ID is typically stored as an environment variable rather than queried from the API, as it's required for most Turnkey API calls. 