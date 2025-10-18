#!/bin/bash

# Test script to verify MCP client behavior
# This simulates what your client should be doing

API_KEY="4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443"
NGROK_URL="https://f652de0ddd8f.ngrok-free.app"
LOCAL_URL="http://localhost:4100"
PROTOCOL_VERSION="2025-03-26"

echo "=== Testing MCP Server ==="
echo "API Key: ${API_KEY:0:8}..."
echo ""

# Test 1: Health check
echo "1. Testing health endpoint..."
curl -sS "${NGROK_URL}/api/health" \
  -H "ngrok-skip-browser-warning: true" | jq
echo ""

# Test 2: Initialize (with proper headers)
echo "2. Testing initialize with proper headers..."
INIT_RESPONSE=$(curl -sS "${NGROK_URL}/api/mcp" \
  -H "Content-Type: application/json" \
  -H "x-api-key: ${API_KEY}" \
  -H "MCP-Protocol-Version: ${PROTOCOL_VERSION}" \
  -H "ngrok-skip-browser-warning: true" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "'"${PROTOCOL_VERSION}"'",
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      },
      "capabilities": {}
    }
  }')

echo "$INIT_RESPONSE" | jq
echo ""

# Test 3: Initialized notification
echo "3. Testing initialized notification..."
curl -sS "${NGROK_URL}/api/mcp" \
  -H "Content-Type: application/json" \
  -H "x-api-key: ${API_KEY}" \
  -H "MCP-Protocol-Version: ${PROTOCOL_VERSION}" \
  -H "ngrok-skip-browser-warning: true" \
  -d '{
    "jsonrpc": "2.0",
    "method": "initialized",
    "params": {}
  }'
echo ""
echo ""

# Test 4: List tools
echo "4. Testing list_tools..."
curl -sS "${NGROK_URL}/api/mcp" \
  -H "Content-Type: application/json" \
  -H "x-api-key: ${API_KEY}" \
  -H "MCP-Protocol-Version: ${PROTOCOL_VERSION}" \
  -H "ngrok-skip-browser-warning: true" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "list_tools",
    "params": {}
  }' | jq '.result.tools | length'
echo ""

# Test 5: Test WITHOUT MCP-Protocol-Version header (should fail)
echo "5. Testing WITHOUT MCP-Protocol-Version header (should fail)..."
curl -sS "${NGROK_URL}/api/mcp" \
  -H "Content-Type: application/json" \
  -H "x-api-key: ${API_KEY}" \
  -H "ngrok-skip-browser-warning: true" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "list_tools",
    "params": {}
  }' | jq '.error'
echo ""

echo "=== Test Complete ==="
echo "If test 5 shows error code -32600, that's the issue your client is experiencing."
echo "Your client needs to send the MCP-Protocol-Version: ${PROTOCOL_VERSION} header."
