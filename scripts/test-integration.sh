#!/bin/bash

# UK Parliament MCP Server - Integration Test Script
# This script tests all endpoints to verify the server is working correctly

set -e

API_KEY="4da006fc4086f0ae7b93420d34b6b955d5f567805fc887531214ddfeaea7c443"
BASE_URL="http://localhost:4100/api/mcp"
HEALTH_URL="http://localhost:4100/api/health"

echo "🧪 Testing UK Parliament MCP Server Integration"
echo "=============================================="

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "❌ jq is required but not installed. Please install jq first."
    exit 1
fi

# Test 1: Health Check
echo "1. Testing health endpoint..."
if curl -s "$HEALTH_URL" | jq -e '.status == "ok"' > /dev/null; then
    echo "✅ Health check passed"
else
    echo "❌ Health check failed"
    exit 1
fi

# Test 2: List Tools
echo "2. Testing list_tools endpoint..."
TOOL_COUNT=$(curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":1,"method":"list_tools","params":{}}' | \
    jq '.result.tools | length')

if [ "$TOOL_COUNT" -eq 11 ]; then
    echo "✅ List tools passed (found $TOOL_COUNT tools)"
else
    echo "❌ List tools failed (expected 11, got $TOOL_COUNT)"
    exit 1
fi

# Test 3: Search - Generic Wrapper
echo "3. Testing search tool..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":2,"method":"call_tool","params":{"name":"search","arguments":{"target":"uk_law","query":"climate change","legislationType":"primary","limit":3,"enableCache":true}}}' | \
    jq -e '.result.structuredContent' > /dev/null; then
    echo "✅ Generic search tool passed"
else
    echo "❌ Generic search tool failed"
    exit 1
fi

# Test 4: Fetch - Generic Wrapper
echo "4. Testing fetch tool..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":3,"method":"call_tool","params":{"name":"fetch","arguments":{"target":"mp_activity","mpId":4592,"limit":3,"enableCache":true}}}' | \
    jq -e '.result.structuredContent' > /dev/null; then
    echo "✅ Generic fetch tool passed"
else
    echo "❌ Generic fetch tool failed"
    exit 1
fi

# Test 5: Utilities - Current DateTime
echo "5. Testing utilities.current_datetime..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":4,"method":"call_tool","params":{"name":"utilities.current_datetime","arguments":{}}}' | \
    jq -e '.result.structuredContent.utc' > /dev/null; then
    echo "✅ Current datetime tool passed"
else
    echo "❌ Current datetime tool failed"
    exit 1
fi

# Test 6: Parliament - Fetch Core Dataset
echo "6. Testing parliament.fetch_core_dataset..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":5,"method":"call_tool","params":{"name":"parliament.fetch_core_dataset","arguments":{"dataset":"commonsmembers","searchTerm":"Johnson","page":0,"perPage":2,"enableCache":true}}}' | \
    jq -e '.result.structuredContent.items' > /dev/null; then
    echo "✅ Core dataset tool passed"
else
    echo "❌ Core dataset tool failed"
    exit 1
fi

# Test 7: Parliament - Fetch Bills
echo "7. Testing parliament.fetch_bills..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":6,"method":"call_tool","params":{"name":"parliament.fetch_bills","arguments":{"searchTerm":"climate","house":"commons","enableCache":true}}}' | \
    jq -e '.result.structuredContent.items' > /dev/null; then
    echo "✅ Bills tool passed"
else
    echo "❌ Bills tool failed"
    exit 1
fi

# Test 8: Parliament - Fetch Legislation
echo "8. Testing parliament.fetch_legislation..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":7,"method":"call_tool","params":{"name":"parliament.fetch_legislation","arguments":{"title":"Human Rights","year":1998,"type":"ukpga","enableCache":true}}}' | \
    jq -e '.result.structuredContent.items' > /dev/null; then
    echo "✅ Legislation tool passed"
else
    echo "❌ Legislation tool failed"
    exit 1
fi

# Test 9: Parliament - Lookup Constituency
echo "9. Testing parliament.lookup_constituency_offline..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":8,"method":"call_tool","params":{"name":"parliament.lookup_constituency_offline","arguments":{"postcode":"SW1A 1AA","enableCache":true}}}' | \
    jq -e '.result.structuredContent.constituencyName' > /dev/null; then
    echo "✅ Constituency lookup tool passed"
else
    echo "❌ Constituency lookup tool failed"
    exit 1
fi

# Test 10: Parliament - Search UK Law
echo "10. Testing parliament.search_uk_law..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":9,"method":"call_tool","params":{"name":"parliament.search_uk_law","arguments":{"query":"climate change","legislationType":"primary","limit":3,"enableCache":true}}}' | \
    jq -e '.result.structuredContent' > /dev/null; then
    echo "✅ UK law search tool passed"
else
    echo "❌ UK law search tool failed"
    exit 1
fi

# Test 11: Research - Run
echo "11. Testing research.run..."
if curl -sS "$BASE_URL" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $API_KEY" \
    -d '{"jsonrpc":"2.0","id":10,"method":"call_tool","params":{"name":"research.run","arguments":{"topic":"climate change","billKeywords":["climate"],"includeStateOfParties":true,"limit":3}}}' | \
    jq -e '.result.structuredContent.summary' > /dev/null; then
    echo "✅ Research tool passed"
else
    echo "❌ Research tool failed"
    exit 1
fi

echo ""
echo "🎉 All tests passed! Your MCP server is ready for Deep Research integration."
echo ""
echo "Next steps:"
echo "1. Open Deep Research: https://chat.openai.com/?model=gpt-4o-deep-research"
echo "2. Go to Settings → Data sources → Add MCP Server"
echo "3. Configure:"
echo "   - Name: UK Parliament Research"
echo "   - Base URL: http://localhost:4100/api/mcp"
echo "   - Header: x-api-key = $API_KEY"
echo "4. Test the connection and start researching!"
echo ""
echo "📚 Documentation: docs/DEEP_RESEARCH_INTEGRATION.md"
