# Configuration Examples

This directory contains example configurations for integrating the UK Parliament MCP Server with various clients and platforms.

## OpenAI Deep Research

### Web Interface Configuration

When adding the MCP server through the Deep Research web interface:

- **Name**: `UK Parliament Research`
- **Base URL**: `http://localhost:4100/api/mcp`
- **Headers**:
  - Key: `x-api-key`
  - Value: `4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443`

### CLI Configuration (`~/.openai/config.json`)

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

### Environment Variables

```bash
export OPENAI_MCP_SERVERS='{"parliament":{"type":"http","url":"http://localhost:4100/api/mcp","headers":{"x-api-key":"4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443"}}}'
```

## Production Configurations

### HTTPS with Reverse Proxy

For production deployments with HTTPS:

```json
{
  "mcpServers": {
    "parliament": {
      "type": "http",
      "url": "https://your-domain.com/api/mcp",
      "headers": {
        "x-api-key": "YOUR_PRODUCTION_API_KEY"
      }
    }
  }
}
```

### Docker Networking

If Deep Research runs in Docker:

```json
{
  "mcpServers": {
    "parliament": {
      "type": "http",
      "url": "http://host.docker.internal:4100/api/mcp",
      "headers": {
        "x-api-key": "4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443"
      }
    }
  }
}
```

### Remote Server

For remote server deployments:

```json
{
  "mcpServers": {
    "parliament": {
      "type": "http",
      "url": "http://your-server-ip:4100/api/mcp",
      "headers": {
        "x-api-key": "YOUR_REMOTE_API_KEY"
      }
    }
  }
}
```

## Testing Configurations

### Health Check

```bash
curl http://localhost:4100/api/health
```

### List Available Tools

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | jq
```

### Test Individual Tool

```bash
curl -sS http://localhost:4100/api/mcp \
  -H "Content-Type: application/json" \
  -H "x-api-key: 4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443" \
  -d '{"jsonrpc":"2.0","id":1,"method":"call_tool","params":{"name":"utilities.current_datetime","arguments":{}}}' | jq
```

## Security Considerations

### API Key Management

1. **Generate New Keys**: Use `./scripts/generate-api-key.sh`
2. **Rotate Keys**: Update both server and client configurations
3. **Secure Storage**: Never commit API keys to version control

### Network Security

1. **HTTPS**: Use SSL/TLS in production
2. **Firewall**: Restrict access to port 4100
3. **Authentication**: Always use API key authentication

### Environment Variables

```bash
# Production environment
export MCP_API_KEY="your-secure-production-key"
export MCP_SERVER_PORT="4100"
export MCP_DISABLE_PROXY="false"
```

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Connection refused | Check if server is running |
| Authentication failed | Verify API key matches |
| Tools not found | Test `list_tools` endpoint |
| Network timeout | Check firewall/network settings |

### Debug Commands

```bash
# Check server status
docker ps | grep mcp-server

# View server logs
docker logs mp-writer-mcp-server-mcp-server-1

# Test connectivity
telnet localhost 4100

# Verify API key
grep MCP_API_KEY .env
```

## Multiple Environments

### Development

```json
{
  "mcpServers": {
    "parliament-dev": {
      "type": "http",
      "url": "http://localhost:4100/api/mcp",
      "headers": {
        "x-api-key": "dev-api-key"
      }
    }
  }
}
```

### Staging

```json
{
  "mcpServers": {
    "parliament-staging": {
      "type": "http",
      "url": "http://staging-server:4100/api/mcp",
      "headers": {
        "x-api-key": "staging-api-key"
      }
    }
  }
}
```

### Production

```json
{
  "mcpServers": {
    "parliament-prod": {
      "type": "http",
      "url": "https://api.parliament-research.com/mcp",
      "headers": {
        "x-api-key": "production-api-key"
      }
    }
  }
}
```
