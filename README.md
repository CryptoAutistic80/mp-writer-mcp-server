# Deep Research MCP Server

This project provides a Rust implementation of the Deep Research MCP server used to expose UK Parliament tools over JSON-RPC. The server exposes a health endpoint and a `/api/mcp` endpoint compatible with the OpenAI Deep Research agent.

## Features

- JSON-RPC 2.0 handling for `initialize`, `list_tools`, and `call_tool` requests.
- Four tools mapped to UK Parliament APIs:
  - `parliament.fetch_core_dataset`
  - `parliament.fetch_bills`
  - `parliament.fetch_historic_hansard`
  - `parliament.fetch_legislation`
- Configurable caching with per-tool TTLs and retry logic for unreliable upstream APIs.
- API key enforcement via the `x-api-key` header.

## Getting Started

1. Copy `.env.example` to `.env` and provide your own values.
2. Install Rust (1.75 or later) and run:

```bash
cargo run
```

The server listens on `0.0.0.0:<MCP_SERVER_PORT>` (default `4100`).

## JSON-RPC Usage

All MCP requests are sent to `POST /api/mcp` with the body encoded as JSON-RPC 2.0. The standard Deep Research sequence is supported:

1. `initialize`
2. `list_tools`
3. `call_tool`

Responses follow the format described in the Deep Research documentation. Tool results return the upstream JSON payload inside `result.content[0].json`.

## Development

- Run `cargo check` or `cargo run` to validate changes.
- The codebase is organized into feature modules per tool domain to match the project's development rules.
- No additional setup is required beyond populating the `.env` file.
