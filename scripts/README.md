# Scripts

This folder contains helper scripts for the Deep Research MCP Server.

## Available Scripts

### `generate-api-key.sh`

Generates a secure random API key for the MCP server.

**Usage:**

```bash
./scripts/generate-api-key.sh
```

**Features:**

- Generates a cryptographically secure 64-character hex key using `openssl`
- Interactive prompts to automatically update your `.env` file
- Creates a backup of existing `.env` before making changes
- Can create a new `.env` file from `.env.example` if none exists
- Color-coded output for better readability

**Example Output:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Generated API Key:
4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Found .env file at: /path/to/project/.env

Do you want to update the .env file with this key? (y/n):
```

**Note:** After updating the API key, remember to restart the server with `cargo run` for changes to take effect.

