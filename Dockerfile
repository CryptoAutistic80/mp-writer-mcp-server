# syntax=docker/dockerfile:1.7

##
# Build stage - compile the Rust binary
##
FROM rust:1.87-slim AS builder

WORKDIR /app

# Install build dependencies (openssl headers needed by reqwest when TLS is enabled)
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cache dependency compilation
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY README.md LICENSE ./

RUN cargo build --release

##
# Runtime stage - minimal image containing the compiled binary
##
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mp-writer-mcp-server /usr/local/bin/mp-writer-mcp-server

# Do not set MCP_SERVER_PORT here so platforms like Cloud Run
# can inject their own PORT and the app will bind to it.
# Only keep a placeholder for MCP_API_KEY; override via env/secret at deploy time.
ENV MCP_API_KEY=change-me

# Documentative only; Cloud Run will ignore EXPOSE.
EXPOSE 8080

CMD ["mp-writer-mcp-server"]
