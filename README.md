# Deep Research MCP Server

Rust implementation of an [OpenAI MCP](https://openai.com/index/introducing-the-model-context-protocol/)–compatible service that exposes UK Parliament data and a research aggregator over JSON‑RPC. The server powers Deep Research workflows by providing up‑to‑date information about members, bills, divisions, and legislation.

---

## Overview

- **JSON‑RPC 2.0** endpoint at `/api/mcp` with `initialize`, `list_tools`, and `call_tool`.
- **Authentication** via mandatory `x-api-key` header.
- **Caching** backed by Sled (persisted) plus in‑memory request cache wrappers.
- **Tools**
  - `parliament.fetch_core_dataset`
  - `parliament.fetch_bills`
  - `parliament.fetch_legislation`
  - `research.run` – orchestrates the three data tools and returns an authored brief with advisories.

---

## Requirements

- Rust 1.75+ (only for local builds)
- Cargo (bundled with Rust)
- Docker / Docker Compose (optional, for containerised deployment)
- OpenAI Deep Research access (for integration)

---

## Configuration

All configuration is driven by environment variables (or a `.env` file). The quickest way to create one is:

```bash
cp .env.example .env
./scripts/generate-api-key.sh   # optional helper; updates MCP_API_KEY
```

| Variable | Description | Default |
| --- | --- | --- |
| `MCP_API_KEY` | **Required.** Shared secret presented in the `x-api-key` header. | – |
| `MCP_SERVER_PORT` | TCP port exposed by the HTTP server. | `4100` |
| `MCP_DISABLE_PROXY` | `true` disables outgoing proxy usage for Reqwest clients. | `false` |
| `CACHE_ENABLED` | Master switch for in-memory HTTP caching. | `true` |
| `CACHE_TTL_MEMBERS` | Cache TTL (seconds) for members dataset calls. | `3600` |
| `CACHE_TTL_BILLS` | Cache TTL for bills queries. | `1800` |
| `CACHE_TTL_LEGISLATION` | Cache TTL for legislation feed fetches. | `7200` |
| `CACHE_TTL_DATA` | Cache TTL for other Linked Data datasets (divisions, debates, etc.). | `1800` |
| `CACHE_TTL_RESEARCH` | TTL for persisted research briefs in Sled. | `604800` (7 days) |
| `RELEVANCE_THRESHOLD` | Default relevance score cut-off used by the aggregator. | `0.3` |
| `MCP_DB_PATH` | Folder that stores the Sled database. | `./data/db` |

> **Note:** Restart the server after changing configuration – values are read at start-up.

---

## Running Locally (Cargo)

```bash
# 1. Clone & configure
git clone https://github.com/<your-org>/mp-writer-mcp-server.git
cd mp-writer-mcp-server
cp .env.example .env          # or use scripts/generate-api-key.sh

# 2. Launch the server
cargo run
```

The server listens on `0.0.0.0:4100` by default. Health check: `curl http://localhost:4100/health`.

---

## Running with Docker

### Build

```bash
docker build -t deep-research-mcp .
```

### Run (stand-alone)

```bash
docker run --rm \
  -p 4100:4100 \
  --env-file .env \
  deep-research-mcp
```

### Run with Docker Compose

```bash
docker compose up --build
```

Edit `docker-compose.yml` or `.env` to customise port bindings or configuration. Use `docker compose down` to stop.

---

## Available Tools

| Tool | Purpose | Key Arguments |
| --- | --- | --- |
| `parliament.fetch_core_dataset` | Query legacy Linked Data datasets (members, divisions, debates, etc.). | `dataset`, `searchTerm`, pagination & relevance toggles |
| `parliament.fetch_bills` | Search the versioned Bills API for current or past bills. | `searchTerm`, `house`, `session`, `parliamentNumber`, relevance controls |
| `parliament.fetch_legislation` | Query legislation.gov.uk Atom feeds for matching acts/orders. | `title`, `year`, `type`, relevance controls |
| `research.run` | Retrieves bills, divisions, debates, legislation, state-of-parties, and composes a brief. Returns advisories when upstream sources fail. | `topic`, optional keyword overrides, `includeStateOfParties`, `limit` |

Each tool responds with the upstream JSON payload. `research.run` returns a structured DTO with `summary`, data vectors, and `advisories`.

---

## Testing the API with `curl`

Replace `YOUR_API_KEY` with the value from your `.env`.

```bash
# 1. List tools
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | jq

# 2. research.run example
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
        "jsonrpc":"2.0",
        "id":2,
        "method":"call_tool",
        "params":{
          "name":"research.run",
          "arguments":{
            "topic":"climate change",
            "billKeywords":["climate action"],
            "debateKeywords":["climate debate"],
            "includeStateOfParties":true,
            "limit":5
          }
        }
      }' | jq '.result.content[0].json'

# 3. Members lookup (Commons only)
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
        "jsonrpc":"2.0",
        "id":3,
        "method":"call_tool",
        "params":{
          "name":"parliament.fetch_core_dataset",
          "arguments":{
            "dataset":"commonsmembers",
            "searchTerm":"Johnson",
            "page":0,
            "perPage":10,
            "enableCache":true,
            "fuzzyMatch":false,
            "applyRelevance":false
          }
        }
      }' | jq '.result.content[0].json'

# 4. Bills search
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
        "jsonrpc":"2.0",
        "id":4,
        "method":"call_tool",
        "params":{
          "name":"parliament.fetch_bills",
          "arguments":{
            "searchTerm":"climate",
            "house":"commons",
            "enableCache":true,
            "applyRelevance":true,
            "relevanceThreshold":0.45
          }
        }
      }' | jq '.result.content[0].json'

# 5. Legislation metadata
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: YOUR_API_KEY" \
  -d '{
        "jsonrpc":"2.0",
        "id":5,
        "method":"call_tool",
        "params":{
          "name":"parliament.fetch_legislation",
          "arguments":{
            "title":"Human Rights",
            "year":1998,
            "type":"ukpga",
            "enableCache":true,
            "applyRelevance":true,
            "relevanceThreshold":0.3
          }
        }
      }' | jq '.result.content[0].json'
```

Error responses include useful `error.data` metadata (status, upstream URL, advisory text) to aid troubleshooting.

---

## Integrating with OpenAI Deep Research

Deep Research can connect to local MCP servers over HTTP. You will need:

1. The server running (locally or reachable over HTTPS) with a known `MCP_API_KEY`.
2. Access to OpenAI Deep Research (web UI or compatible client).

### Using the Web Interface

1. Open Deep Research and go to **Settings → Data sources → Add MCP Server**.
2. Enter a name (e.g., `UK Parliament Research`).
3. Set the base URL to `http://localhost:4100/api/mcp` (or your public URL).
4. Add a header `x-api-key` with the value from your `.env`.
5. Save and run the connection test – the server should respond with the tool catalogue.

### Using the OpenAI CLI / Config File

If you manage MCP servers through `~/.openai/config.json`, add an entry like:

```json
{
  "mcpServers": {
    "parliament": {
      "type": "http",
      "url": "http://localhost:4100/api/mcp",
      "headers": {
        "x-api-key": "YOUR_API_KEY"
      }
    }
  }
}
```

After saving the configuration, restart the OpenAI client or rerun Deep Research so it loads the new MCP definition.

### Verifying the Connection

- In Deep Research, start a new run and choose the MCP server as a data source.
- The agent should invoke the tools automatically; you can inspect invocation logs to confirm.
- If authentication fails, double-check the header name (`x-api-key`) and ensure the server is reachable from the Deep Research environment.

---

## Development & Testing

- Lint or format: `cargo fmt`, `cargo clippy`
- Unit / integration tests: `cargo test`
- Research service fixture test: `cargo test --test research_tests`

The repository includes a `scripts/` directory with helper utilities and a `docs/` folder containing the longer-term technical plan.

---

## Troubleshooting

- **401 / Unauthorized** – ensure `x-api-key` matches `MCP_API_KEY`.
- **404 from upstream APIs** – the server surfaces upstream URLs/status in `error.data`; adjust queries or review API changes.
- **Docker networking** – when running inside Docker, expose the container port and use `http://host.docker.internal:4100/api/mcp` from host clients.
- **Deep Research does not list the server** – re-run the connection test in settings and verify the server is reachable over HTTPS if accessed from the cloud.

---

## License

See [LICENSE](LICENSE).
