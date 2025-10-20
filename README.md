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
  - `parliament.fetch_mp_activity`
  - `parliament.fetch_mp_voting_record`
  - `parliament.lookup_constituency_offline`
  - `parliament.search_uk_law`
  - `research.run` – orchestrates the three data tools and returns an authored brief with advisories.
  - `utilities.current_datetime`

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
| `CACHE_TTL_ACTIVITY` | TTL for cached MP activity responses (seconds). | `21600` (6 hours) |
| `CACHE_TTL_VOTES` | TTL for cached voting record responses. | `21600` (6 hours) |
| `CACHE_TTL_CONSTITUENCY` | TTL for offline constituency lookups. | `86400` (24 hours) |
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

The server listens on `0.0.0.0:4100` by default. Health check: `curl http://localhost:4100/api/health`.

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
| `parliament.fetch_core_dataset` | Query legacy Linked Data datasets (members, divisions, debates, etc.). | `dataset` (required), `searchTerm`, `page`, `perPage`, `enableCache`, `fuzzyMatch`, `applyRelevance`, `relevanceThreshold` |
| `parliament.fetch_bills` | Search the versioned Bills API for current or past bills. | `searchTerm`, `house`, `session`, `parliamentNumber`, `enableCache`, `applyRelevance`, `relevanceThreshold` |
| `parliament.fetch_legislation` | Query legislation.gov.uk Atom feeds for matching acts/orders. | `title`, `year` (>= 1800), `type`, `enableCache`, `applyRelevance`, `relevanceThreshold` |
| `parliament.fetch_mp_activity` | Recent debates, questions and other activity for a specific MP. | `mpId` (required), `limit`, `enableCache` |
| `parliament.fetch_mp_voting_record` | Summarise votes cast by an MP, with optional date/bill filters. | `mpId` (required), `fromDate`, `toDate`, `billId`, `limit`, `enableCache` |
| `parliament.lookup_constituency_offline` | Resolve a postcode to its Westminster constituency and current MP (best effort). | `postcode` (required), `enableCache` |
| `parliament.search_uk_law` | Search UK primary/secondary legislation by title keywords. | `query` (required), `legislationType`, `limit`, `enableCache` |
| `research.run` | Retrieve bills, debates, legislation, votes, state-of-parties and compose a brief. Returns advisories when sources fail. | `topic` (required), `billKeywords`, `debateKeywords`, `mpId`, `includeStateOfParties`, `limit` |
| `utilities.current_datetime` | Return current UTC and Europe/London timestamps. | – |

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

This MCP server provides comprehensive UK Parliament data to OpenAI Deep Research, enabling AI-powered research on legislation, MPs, voting records, and parliamentary activity.

### Prerequisites

1. **Server Running**: The MCP server must be running (locally or remotely) with a known `MCP_API_KEY`
2. **Deep Research Access**: Access to OpenAI Deep Research (web UI or compatible client)
3. **Network Connectivity**: Deep Research must be able to reach your server

### Quick Setup (Web Interface - Recommended)

1. **Start Your Server**
   ```bash
   # Using Docker Compose (recommended)
   docker compose up -d
   
   # Or using Cargo
   cargo run
   ```

2. **Open Deep Research**
   - Go to [Deep Research](https://chat.openai.com/?model=gpt-4o-deep-research)
   - Sign in to your OpenAI account

3. **Add MCP Server**
   - Navigate to **Settings → Data sources → Add MCP Server**
   - **Name**: `UK Parliament Research`
   - **Base URL**: `http://localhost:4100/api/mcp`
   - **Headers**: Add header with:
     - **Key**: `x-api-key`
     - **Value**: `4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443`

4. **Test Connection**
   - Click **Test Connection**
   - Should show success with 9 available tools
   - Click **Save**

### CLI Configuration

For OpenAI CLI users, create/edit `~/.openai/config.json`:

```json
{
  "mcpServers": {
    "parliament": {
      "type": "http",
      "url": "http://localhost:4100/api/mcp",
      "headers": {
        "x-api-key": "4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443"
      }
    }
  }
}
```

Restart your OpenAI client after saving the configuration.

### Testing the Integration

#### Sample Research Queries

Try these queries in Deep Research to test the integration:

```
"Research recent climate change legislation in the UK Parliament"
"Find information about Boris Johnson's voting record"
"What bills are currently being debated in the House of Commons?"
"Look up constituency information for postcode SW1A 1AA"
"Analyze the current state of parties in Parliament"
```

#### Monitoring Server Activity

Watch your server logs to see Deep Research invoking tools:

```bash
# Monitor Docker logs
docker logs -f mp-writer-mcp-server-mcp-server-1

# Or monitor Cargo logs
cargo run
```

### Available Tools for Deep Research

| Tool | Purpose | Example Use Case |
|------|---------|------------------|
| `search` | Generic search across Parliament data | "Search climate legislation" |
| `fetch` | Generic fetch for Parliament records | "Fetch MP 4592 activity" |
| `parliament.fetch_core_dataset` | Query MPs, divisions, debates | "Find all Labour MPs" |
| `parliament.fetch_bills` | Search current/past bills | "What climate bills are active?" |
| `parliament.fetch_legislation` | UK legislation metadata | "Find Human Rights Act details" |
| `parliament.fetch_mp_activity` | MP's recent activity | "What has Caroline Johnson been doing?" |
| `parliament.fetch_mp_voting_record` | MP voting history | "How did Boris Johnson vote on Brexit?" |
| `parliament.lookup_constituency_offline` | Postcode to constituency | "What constituency is SW1A 1AA?" |
| `parliament.search_uk_law` | Search UK legislation | "Find all climate change laws" |
| `research.run` | Comprehensive research brief | "Research UK net zero policy" |
| `utilities.current_datetime` | Current time (UTC/London) | "What's the current time?" |

### Troubleshooting

#### Connection Issues

**Problem**: Connection test fails
- **Solution**: Verify server is running with `curl http://localhost:4100/api/health`
- **Check**: Docker container status with `docker ps`

**Problem**: Authentication errors
- **Solution**: Ensure header name is exactly `x-api-key` (case-sensitive)
- **Check**: API key matches the one in your `.env` file

**Problem**: Tools not appearing
- **Solution**: Test `list_tools` endpoint manually
- **Check**: Server logs for errors

#### Network Configuration

**Local Development**:
- Use `http://localhost:4100/api/mcp`
- Ensure Deep Research can reach localhost

**Remote Server**:
- Replace `localhost` with your server's IP/domain
- Consider HTTPS for production deployments

**Docker Networking**:
- Use `http://host.docker.internal:4100/api/mcp` if Deep Research runs in Docker
- Or expose port 4100 to host network

### Production Deployment

For production use with Deep Research:

1. **HTTPS Setup**
   ```bash
   # Use reverse proxy with SSL
   nginx -t && systemctl reload nginx
   ```

2. **API Key Management**
   ```bash
   # Generate new API key
   ./scripts/generate-api-key.sh
   
   # Update both server and client configs
   ```

3. **Monitoring**
   ```bash
   # Set up log monitoring
   docker logs --tail=100 -f mp-writer-mcp-server-mcp-server-1
   ```

### Advanced Usage

#### Custom Research Workflows

Deep Research can combine multiple tools for comprehensive analysis:

```
"Research the UK's approach to climate change by analyzing:
1. Current climate bills in Parliament
2. Recent voting records of key MPs
3. Existing climate legislation
4. Current party positions"
```

#### Integration with Other Tools

The MCP server works alongside other Deep Research data sources:
- Web search results
- Document analysis
- Other MCP servers

### Support

- **Server Issues**: Check [Troubleshooting](#troubleshooting) section
- **API Documentation**: See [Available Tools](#available-tools-for-deep-research)
- **Configuration**: Reference `.env.example` for all options

### Additional Documentation

- **[Deep Research Integration Guide](docs/DEEP_RESEARCH_INTEGRATION.md)** - Step-by-step setup instructions
- **[Configuration Examples](docs/CONFIGURATION_EXAMPLES.md)** - Various deployment scenarios
- **[API Testing Guide](docs/API_TESTING_GUIDE.md)** - Comprehensive endpoint testing
- **[Quick Reference](docs/QUICK_REFERENCE.md)** - Essential commands and settings

---

## Development & Testing

- Lint or format: `cargo fmt`, `cargo clippy`
- Unit / integration tests: `cargo test`
- Research service fixture test: `cargo test --test research_tests`

The repository includes a `scripts/` directory with helper utilities.

---

## Troubleshooting

- **401 / Unauthorized** – ensure `x-api-key` matches `MCP_API_KEY`.
- **404 from upstream APIs** – the server surfaces upstream URLs/status in `error.data`; adjust queries or review API changes.
- **Docker networking** – when running inside Docker, expose the container port and use `http://host.docker.internal:4100/api/mcp` from host clients.
- **Deep Research does not list the server** – re-run the connection test in settings and verify the server is reachable over HTTPS if accessed from the cloud.

---

## License

See [LICENSE](LICENSE).
