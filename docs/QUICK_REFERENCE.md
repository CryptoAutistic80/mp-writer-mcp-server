# Quick Reference

## Server Configuration

| Setting | Value |
|---------|-------|
| **Port** | 4100 |
| **Health Check** | `http://localhost:4100/api/health` |
| **MCP Endpoint** | `http://localhost:4100/api/mcp` |
| **API Key** | `4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443` |
| **Header Name** | `x-api-key` |

## Available Tools

| Tool | Purpose |
|------|---------|
| `utilities.current_datetime` | Current time (UTC/London) |
| `parliament.fetch_core_dataset` | Query MPs, divisions, debates |
| `parliament.fetch_bills` | Search current/past bills |
| `parliament.fetch_legislation` | UK legislation metadata |
| `parliament.fetch_mp_activity` | MP's recent activity |
| `parliament.fetch_mp_voting_record` | MP voting history |
| `parliament.lookup_constituency_offline` | Postcode to constituency |
| `parliament.search_uk_law` | Search UK legislation |
| `research.run` | Comprehensive research brief |

## Deep Research Setup

### Web Interface
1. **Settings → Data sources → Add MCP Server**
2. **Name**: `UK Parliament Research`
3. **Base URL**: `http://localhost:4100/api/mcp`
4. **Headers**: `x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443`

### CLI Config (`~/.openai/config.json`)
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

## Quick Commands

### Start Server
```bash
docker compose up -d
```

### Test Health
```bash
curl http://localhost:4100/api/health
```

### List Tools
```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | jq
```

### Monitor Logs
```bash
docker logs -f mp-writer-mcp-server-mcp-server-1
```

## Sample Queries for Deep Research

```
"Research recent climate change legislation in the UK Parliament"
"Find information about Boris Johnson's voting record"
"What bills are currently being debated in the House of Commons?"
"Look up constituency information for postcode SW1A 1AA"
"Analyze the current state of parties in Parliament"
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Connection failed | `curl http://localhost:4100/api/health` |
| Auth error | Check header name is `x-api-key` |
| Tools missing | Test `list_tools` endpoint |
| Server down | `docker ps` and restart if needed |

## Documentation

- **Integration Guide**: [docs/DEEP_RESEARCH_INTEGRATION.md](docs/DEEP_RESEARCH_INTEGRATION.md)
- **Configuration Examples**: [docs/CONFIGURATION_EXAMPLES.md](docs/CONFIGURATION_EXAMPLES.md)
- **API Testing Guide**: [docs/API_TESTING_GUIDE.md](docs/API_TESTING_GUIDE.md)
