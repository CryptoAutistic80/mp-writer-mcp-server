# OpenAI Deep Research Integration Guide

This guide provides step-by-step instructions for integrating the UK Parliament MCP Server with OpenAI Deep Research.

## 🚀 Quick Start

### 1. Start the Server

```bash
# Using Docker Compose (recommended)
docker compose up -d

# Verify it's running
curl http://localhost:4100/api/health
# Should return: {"status":"ok"}
```

### 2. Configure Deep Research

#### Option A: Web Interface (Recommended)

1. Open [Deep Research](https://chat.openai.com/?model=gpt-4o-deep-research)
2. Go to **Settings → Data sources → Add MCP Server**
3. Configure:
   - **Name**: `UK Parliament Research`
   - **Base URL**: `http://localhost:4100/api/mcp`
   - **Headers**: 
     - Key: `x-api-key`
     - Value: `4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443`
4. Click **Test Connection** → Should show 9 tools
5. Click **Save**

#### Option B: CLI Configuration

Create/edit `~/.openai/config.json`:

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

Restart your OpenAI client.

## 🧪 Testing the Integration

### Sample Queries

Try these queries in Deep Research:

```
"Research recent climate change legislation in the UK Parliament"
"Find information about Boris Johnson's voting record"
"What bills are currently being debated in the House of Commons?"
"Look up constituency information for postcode SW1A 1AA"
"Analyze the current state of parties in Parliament"
```

### Monitor Server Activity

```bash
# Watch server logs
docker logs -f mp-writer-mcp-server-mcp-server-1

# Test individual tools
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | jq
```

## 🔧 Troubleshooting

### Connection Issues

| Problem | Solution |
|---------|----------|
| Connection test fails | Check server: `curl http://localhost:4100/api/health` |
| Authentication errors | Verify header name is exactly `x-api-key` |
| Tools not appearing | Test `list_tools` endpoint manually |

### Network Configuration

- **Local**: Use `http://localhost:4100/api/mcp`
- **Remote**: Replace `localhost` with server IP/domain
- **Docker**: Use `http://host.docker.internal:4100/api/mcp`

## 📊 Available Tools

| Tool | Purpose | Example |
|------|---------|---------|
| `parliament.fetch_core_dataset` | Query MPs, divisions, debates | "Find all Labour MPs" |
| `parliament.fetch_bills` | Search bills | "What climate bills are active?" |
| `parliament.fetch_legislation` | UK legislation | "Find Human Rights Act" |
| `parliament.fetch_mp_activity` | MP activity | "What has Caroline Johnson been doing?" |
| `parliament.fetch_mp_voting_record` | Voting history | "How did Boris Johnson vote?" |
| `parliament.lookup_constituency_offline` | Postcode lookup | "What constituency is SW1A 1AA?" |
| `parliament.search_uk_law` | Search laws | "Find climate change laws" |
| `research.run` | Research brief | "Research UK net zero policy" |
| `utilities.current_datetime` | Current time | "What's the current time?" |

## 🎯 Advanced Usage

### Complex Research Queries

```
"Research the UK's approach to climate change by analyzing:
1. Current climate bills in Parliament
2. Recent voting records of key MPs
3. Existing climate legislation
4. Current party positions"
```

### Integration with Other Sources

The MCP server works alongside:
- Web search results
- Document analysis
- Other MCP servers

## 🚀 Production Deployment

### HTTPS Setup

```bash
# Use reverse proxy with SSL
nginx -t && systemctl reload nginx
```

### API Key Management

```bash
# Generate new API key
./scripts/generate-api-key.sh

# Update both server and client configs
```

### Monitoring

```bash
# Set up log monitoring
docker logs --tail=100 -f mp-writer-mcp-server-mcp-server-1
```

## 📞 Support

- **Server Issues**: Check troubleshooting section
- **API Documentation**: See README.md
- **Configuration**: Reference `.env.example`
