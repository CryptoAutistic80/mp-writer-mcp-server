# Deep Research MCP Server

This project provides a Rust implementation of the Deep Research MCP server used to expose UK Parliament tools over JSON-RPC. The server exposes a health endpoint and a `/api/mcp` endpoint compatible with the OpenAI Deep Research agent.

## Features

- JSON-RPC 2.0 handling for `initialize`, `list_tools`, and `call_tool` requests.
- Four tools mapped to UK Parliament APIs:
  - `parliament.fetch_core_dataset` - Access core datasets (members, constituencies, etc.)
  - `parliament.fetch_bills` - Search for UK Parliament bills
  - `parliament.fetch_historic_hansard` - Retrieve historic Hansard debate transcripts
  - `parliament.fetch_legislation` - Retrieve legislation metadata from legislation.gov.uk
- Configurable caching with per-tool TTLs and retry logic for unreliable upstream APIs.
- API key enforcement via the `x-api-key` header for all MCP endpoints.

## Getting Started

### 1. Configure Environment

**Option A: Use the helper script (recommended)**

Run the included script to generate and configure your API key automatically:

```bash
./scripts/generate-api-key.sh
```

This script will:
- Generate a secure random API key
- Optionally update your `.env` file with the new key
- Create a backup of your existing `.env` file

**Option B: Manual setup**

Copy `.env.example` to `.env` and set your API key:

```bash
cp .env.example .env
```

Generate a secure API key:

```bash
openssl rand -hex 32
```

Update the `MCP_API_KEY` in your `.env` file with the generated key.

### 2. Install and Run

Install Rust (1.75 or later) and start the server:

```bash
cargo run
```

The server listens on `0.0.0.0:<MCP_SERVER_PORT>` (default `4100`).

**Note:** Restart the server after changing `.env` values for changes to take effect.

## Integrating with OpenAI Deep Search

1. **Expose the MCP endpoint**
   - Ensure the process running Deep Search can reach `http://<host>:<port>/api/mcp`.
   - Keep the API key from your `.env` file handy; Deep Search must provide it in the `x-api-key` header.

2. **Register the connector inside Deep Search**
   - In your application's connector configuration, declare an HTTP MCP server entry similar to:
     ```json
     {
       "url": "http://<host>:4100/api/mcp",
       "api_key": "YOUR_MCP_API_KEY"
     }
     ```
   - If your app supports environment inheritance, you can instead supply `DEEP_RESEARCH_MCP_PORT` and `DEEP_RESEARCH_API_KEY`; the server reads both as fallbacks.

3. **Verify the handshake**
   - Trigger Deep Search to connect; it will call `initialize`, then `list_tools`, followed by `call_tool` as needed.
   - You can emulate the first two steps manually with:
     ```bash
     curl -X POST http://localhost:4100/api/mcp \
       -H "Content-Type: application/json" \
       -H "x-api-key: YOUR_API_KEY" \
       -d '{
         "jsonrpc": "2.0",
         "id": 1,
         "method": "initialize",
         "params": {
           "protocolVersion": "1.0.0",
           "clientInfo": { "name": "smoke-test", "version": "0.1.0" },
           "capabilities": {}
         }
       }' | jq
     ```
   - A successful response includes the server metadata and advertised capabilities:
     ```json
     {
       "jsonrpc": "2.0",
       "id": 1,
       "result": {
         "serverInfo": { "name": "mp-writer-mcp-server", "version": "0.1.0" },
         "capabilities": { "tools": { "listChanged": false } }
       }
     }
     ```
   - Follow up with `list_tools` (see below) to confirm the Deep Search runtime will receive tool definitions.

## API Endpoints

### Health Check (No authentication required)

```bash
curl http://localhost:4100/api/health | jq
```

Response:
```json
{
  "status": "ok"
}
```

### MCP Endpoint (Authentication required)

All MCP requests require the `x-api-key` header and are sent to `POST /api/mcp` with JSON-RPC 2.0 format.

## Usage Examples

### 1. Initialize Session

```bash
curl -X POST http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "1.0.0",
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      },
      "capabilities": {}
    }
  }' | jq
```

### 2. List Available Tools

```bash
curl -X POST http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "list_tools",
    "params": {}
  }' | jq
```

### 3. Call Tool: Fetch Core Dataset

Search for members:

```bash
curl -X POST http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "call_tool",
    "params": {
      "name": "parliament.fetch_core_dataset",
      "arguments": {
        "dataset": "members",
        "searchTerm": "Johnson",
        "page": 0,
        "perPage": 10,
        "enableCache": true,
        "fuzzyMatch": true,
        "applyRelevance": false
      }
    }
  }' | jq
```

### 4. Call Tool: Fetch Bills

Search for bills by keyword:

```bash
curl -X POST http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "call_tool",
    "params": {
      "name": "parliament.fetch_bills",
      "arguments": {
        "searchTerm": "climate",
        "house": "commons",
        "enableCache": true,
        "applyRelevance": true,
        "relevanceThreshold": 0.5
      }
    }
  }' | jq
```

### 5. Call Tool: Fetch Historic Hansard

Retrieve debate transcripts:

```bash
curl -X POST http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
    "jsonrpc": "2.0",
    "id": 5,
    "method": "call_tool",
    "params": {
      "name": "parliament.fetch_historic_hansard",
      "arguments": {
        "house": "commons",
        "path": "1803/jun/20/war-message-from-the-throne",
        "enableCache": true
      }
    }
  }' | jq
```

### 6. Call Tool: Fetch Legislation

Search for legislation:

```bash
curl -X POST http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
    "jsonrpc": "2.0",
    "id": 6,
    "method": "call_tool",
    "params": {
      "name": "parliament.fetch_legislation",
      "arguments": {
        "title": "Human Rights",
        "year": 1998,
        "type": "ukpga",
        "enableCache": true,
        "applyRelevance": true,
        "relevanceThreshold": 0.3
      }
    }
  }' | jq
```

## JSON-RPC Response Format

Tool results return the upstream JSON payload inside `result.content[0].json`:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "json",
        "json": { /* upstream API response */ }
      }
    ]
  }
}
```

## Configuration

The `.env` file supports the following variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `MCP_API_KEY` | API key for authentication (required) | - |
| `MCP_SERVER_PORT` | Server port | `4100` |
| `MCP_DISABLE_PROXY` | Disable proxy for upstream requests | `false` |
| `CACHE_ENABLED` | Enable response caching | `true` |
| `CACHE_TTL_MEMBERS` | Cache TTL for members (seconds) | `3600` |
| `CACHE_TTL_BILLS` | Cache TTL for bills (seconds) | `1800` |
| `CACHE_TTL_LEGISLATION` | Cache TTL for legislation (seconds) | `7200` |
| `CACHE_TTL_HANSARD` | Cache TTL for Hansard (seconds) | `3600` |
| `CACHE_TTL_DATA` | Cache TTL for core datasets (seconds) | `1800` |
| `RELEVANCE_THRESHOLD` | Default relevance score threshold | `0.3` |

## Development

- Run `cargo check` to validate changes without building.
- Run `cargo run` to start the development server.
- The codebase follows modular organization:
  - `src/config/` - Configuration loading
  - `src/core/` - Shared utilities (cache, error handling)
  - `src/features/mcp/` - JSON-RPC handler and service
  - `src/features/parliament/` - UK Parliament API integration
  - `src/server/` - HTTP server setup and middleware

## Error Handling

The server returns standard JSON-RPC 2.0 error responses:

| Code | Description |
|------|-------------|
| `-32700` | Parse error |
| `-32600` | Invalid request |
| `-32601` | Method not found |
| `-32602` | Invalid params |
| `-32000` | Internal error |
| `-32002` | Upstream API error |

HTTP status codes:
- `200` - Success (with JSON-RPC response)
- `401` - Unauthorized (missing or invalid API key)
