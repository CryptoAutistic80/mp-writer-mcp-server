#!/bin/bash

# Generate API Key Script for Deep Research MCP Server
# This script generates a secure random API key and optionally updates the .env file

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Generate a secure random API key
API_KEY=$(openssl rand -hex 32)

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Generated API Key:${NC}"
echo -e "${YELLOW}${API_KEY}${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Get the project root directory (parent of scripts folder)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ENV_FILE="${PROJECT_ROOT}/.env"

# Check if .env file exists
if [ -f "$ENV_FILE" ]; then
    echo "Found .env file at: $ENV_FILE"
    echo ""
    read -p "Do you want to update the .env file with this key? (y/n): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Backup the original .env file
        cp "$ENV_FILE" "${ENV_FILE}.backup"
        echo -e "${GREEN}✓${NC} Backed up .env to .env.backup"
        
        # Update the MCP_API_KEY in .env
        if grep -q "^MCP_API_KEY=" "$ENV_FILE"; then
            # Key exists, replace it
            sed -i "s|^MCP_API_KEY=.*|MCP_API_KEY=${API_KEY}|" "$ENV_FILE"
            echo -e "${GREEN}✓${NC} Updated MCP_API_KEY in .env"
        else
            # Key doesn't exist, append it
            echo "MCP_API_KEY=${API_KEY}" >> "$ENV_FILE"
            echo -e "${GREEN}✓${NC} Added MCP_API_KEY to .env"
        fi
        
        echo ""
        echo -e "${YELLOW}⚠ Important:${NC} Restart the server for the new API key to take effect!"
        echo -e "   Run: ${BLUE}cargo run${NC}"
    else
        echo ""
        echo "Skipped updating .env file."
        echo "To use this key, manually update MCP_API_KEY in your .env file:"
        echo -e "  ${BLUE}MCP_API_KEY=${API_KEY}${NC}"
    fi
else
    echo -e "${YELLOW}⚠ No .env file found at: $ENV_FILE${NC}"
    echo ""
    read -p "Do you want to create a new .env file? (y/n): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [ -f "${PROJECT_ROOT}/.env.example" ]; then
            cp "${PROJECT_ROOT}/.env.example" "$ENV_FILE"
            sed -i "s|^MCP_API_KEY=.*|MCP_API_KEY=${API_KEY}|" "$ENV_FILE"
            echo -e "${GREEN}✓${NC} Created .env from .env.example"
            echo -e "${GREEN}✓${NC} Set MCP_API_KEY in .env"
        else
            echo "MCP_API_KEY=${API_KEY}" > "$ENV_FILE"
            echo "MCP_SERVER_PORT=4100" >> "$ENV_FILE"
            echo "CACHE_ENABLED=true" >> "$ENV_FILE"
            echo -e "${GREEN}✓${NC} Created new .env file with basic configuration"
        fi
    else
        echo ""
        echo "No .env file created. To use this key, create a .env file with:"
        echo -e "  ${BLUE}MCP_API_KEY=${API_KEY}${NC}"
    fi
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Done!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

