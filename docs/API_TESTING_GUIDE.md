# API Testing Guide

This guide provides comprehensive testing instructions for the UK Parliament MCP Server endpoints.

## Prerequisites

- Server running on `http://localhost:4100`
- API key: `4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443`
- `curl` and `jq` installed
- Include the header `MCP-Protocol-Version: 2025-03-26` on every MCP HTTP request (including notifications).

## Health Check

```bash
curl http://localhost:4100/api/health
```

**Expected Response:**
```json
{"status":"ok"}
```

## MCP Protocol Endpoints

### List Available Tools

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | jq
```

**Expected Response:** JSON object with 9 tools listed

### Initialize (Optional)

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | jq
```

## Tool Testing

### 1. Utilities: Current DateTime

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"utilities.current_datetime","arguments":{}}}' | jq
```

**Expected Response:** Current UTC and London time

### 2. Parliament: Fetch Core Dataset

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":2,"method":"call_tool","params":{"name":"parliament.fetch_core_dataset","arguments":{"dataset":"commonsmembers","searchTerm":"Johnson","page":0,"perPage":5,"enableCache":true,"fuzzyMatch":false,"applyRelevance":false}}}' | jq
```

**Expected Response:** List of MPs named "Johnson"

### 3. Parliament: Fetch Bills

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":3,"method":"call_tool","params":{"name":"parliament.fetch_bills","arguments":{"searchTerm":"climate","house":"commons","enableCache":true,"applyRelevance":true,"relevanceThreshold":0.45}}}' | jq
```

**Expected Response:** List of climate-related bills

### 4. Parliament: Fetch Legislation

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":4,"method":"call_tool","params":{"name":"parliament.fetch_legislation","arguments":{"title":"Human Rights","year":1998,"type":"ukpga","enableCache":true,"applyRelevance":true,"relevanceThreshold":0.3}}}' | jq
```

**Expected Response:** Human Rights Act 1998 details

### 5. Parliament: Fetch MP Activity

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":5,"method":"call_tool","params":{"name":"parliament.fetch_mp_activity","arguments":{"mpId":4592,"limit":5,"enableCache":true}}}' | jq
```

**Expected Response:** Recent activity for MP ID 4592

### 6. Parliament: Fetch MP Voting Record

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":6,"method":"call_tool","params":{"name":"parliament.fetch_mp_voting_record","arguments":{"mpId":4592,"limit":5,"enableCache":true}}}' | jq
```

**Expected Response:** Recent voting record for MP ID 4592

### 7. Parliament: Lookup Constituency

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":7,"method":"call_tool","params":{"name":"parliament.lookup_constituency_offline","arguments":{"postcode":"SW1A 1AA","enableCache":true}}}' | jq
```

**Expected Response:** Constituency information for SW1A 1AA

### 8. Parliament: Search UK Law

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":8,"method":"call_tool","params":{"name":"parliament.search_uk_law","arguments":{"query":"climate change","legislationType":"primary","limit":5,"enableCache":true}}}' | jq
```

**Expected Response:** Climate change legislation

### 9. Research: Run

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":9,"method":"call_tool","params":{"name":"research.run","arguments":{"topic":"climate change","billKeywords":["climate action"],"debateKeywords":["climate debate"],"includeStateOfParties":true,"limit":5}}}' | jq
```

**Expected Response:** Comprehensive research brief on climate change

## Automated Testing Script

Create a test script to run all endpoints:

```bash
#!/bin/bash

API_KEY="4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443"
BASE_URL="http://localhost:4100/api/mcp"
PROTOCOL_VERSION="2025-03-26"

echo "Testing UK Parliament MCP Server..."

# Health check
echo "1. Health check..."
curl -s http://localhost:4100/api/health | jq

# List tools
echo "2. List tools..."
curl -sS "$BASE_URL" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $API_KEY" \
  -H "MCP-Protocol-Version: $PROTOCOL_VERSION" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | jq '.result.tools | length'

# Test each tool
echo "3. Testing utilities.current_datetime..."
curl -sS "$BASE_URL" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $API_KEY" \
  -H "MCP-Protocol-Version: $PROTOCOL_VERSION" \
  -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"utilities.current_datetime","arguments":{}}}' | jq '.result.content[0].json'

echo "4. Testing parliament.fetch_core_dataset..."
curl -sS "$BASE_URL" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $API_KEY" \
  -H "MCP-Protocol-Version: $PROTOCOL_VERSION" \
  -d '{"jsonrpc":"2.0","id":2,"method":"call_tool","params":{"name":"parliament.fetch_core_dataset","arguments":{"dataset":"commonsmembers","searchTerm":"Johnson","page":0,"perPage":2,"enableCache":true}}}' | jq '.result.content[0].json.items | length'

echo "5. Testing parliament.fetch_bills..."
curl -sS "$BASE_URL" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $API_KEY" \
  -H "MCP-Protocol-Version: $PROTOCOL_VERSION" \
  -d '{"jsonrpc":"2.0","id":3,"method":"call_tool","params":{"name":"parliament.fetch_bills","arguments":{"searchTerm":"climate","house":"commons","enableCache":true}}}' | jq '.result.content[0].json.items | length'

echo "All tests completed!"
```

## Error Testing

### Invalid API Key

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: invalid-key" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | jq
```

**Expected Response:** 401 Unauthorized

### Missing API Key

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | jq
```

**Expected Response:** 401 Unauthorized

### Invalid Tool Name

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"invalid.tool","arguments":{}}}' | jq
```

**Expected Response:** Error with tool not found

### Invalid Arguments

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"parliament.fetch_mp_activity","arguments":{"mpId":"invalid"}}}' | jq
```

**Expected Response:** Error with invalid arguments

## Performance Testing

### Load Testing

```bash
# Test with multiple concurrent requests
for i in {1..10}; do
  curl -sS http://localhost:4100/api/mcp \
    -H "Content-Type: application/json" \
    -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
    -H "MCP-Protocol-Version: 2025-03-26" \
    -d '{"jsonrpc":"2.0","id":'$i',"method":"call_tool","params":{"name":"utilities.current_datetime","arguments":{}}}' &
done
wait
```

### Cache Testing

```bash
# First request (cache miss)
time curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"parliament.fetch_core_dataset","arguments":{"dataset":"commonsmembers","searchTerm":"Johnson","enableCache":true}}}' > /dev/null

# Second request (cache hit)
time curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -H "MCP-Protocol-Version: 2025-03-26" \
  -d '{"jsonrpc":"2.0","id":2,"method":"call_tool","params":{"name":"parliament.fetch_core_dataset","arguments":{"dataset":"commonsmembers","searchTerm":"Johnson","enableCache":true}}}' > /dev/null
```

## Monitoring

### Server Logs

```bash
# Monitor Docker logs
docker logs -f mp-writer-mcp-server-mcp-server-1

# Monitor with timestamps
docker logs -f --timestamps mp-writer-mcp-server-mcp-server-1
```

### Health Monitoring

```bash
# Continuous health check
while true; do
  curl -s http://localhost:4100/api/health || echo "Server down at $(date)"
  sleep 30
done
```

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Connection refused | Check if server is running |
| 401 Unauthorized | Verify API key |
| 404 Not Found | Check endpoint URL |
| Timeout | Check network connectivity |
| Invalid JSON | Verify request format |

### Debug Commands

```bash
# Check server status
docker ps | grep mcp-server

# Check port binding
netstat -tlnp | grep 4100

# Test basic connectivity
telnet localhost 4100

# Check API key
grep MCP_API_KEY .env
```
