# Kimi Proxy

A lightweight HTTP proxy server written in Rust that forwards requests to the [Kimi API](https://api.kimi.com/coding/v1), with User-Agent spoofing to make it compatible with Zeroclaw.

## Overview

This is a Rust port of the original Python `kimi_proxy.py`, providing the same functionality with better performance and lower resource usage. It's designed to sit between Zeroclaw and the Kimi API, forwarding all requests while spoofing the User-Agent header to appear as `claude-code/0.1.0`.

## Features

- 🚀 **High Performance**: Built with Actix-web and Tokio for async I/O
- 🔄 **Full HTTP Support**: Handles GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- 📡 **Streaming**: Supports streaming responses for real-time API usage
- 🪶 **Lightweight**: Minimal memory footprint compared to Python equivalent
- 🔧 **Configurable**: Port can be configured via environment variable

## Prerequisites

- [Rust](https://rustup.rs/) (1.75 or later recommended)
- A valid Kimi API key (set via `Authorization` header in requests)

## Installation

```bash
cd kimi-proxy
cargo build --release
```

## Usage

### Run the proxy

```bash
# Default port (8787)
cargo run

# Custom port
PORT=3000 cargo run
```

### Or run the release binary directly

```bash
./target/release/kimi-proxy
```

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `PORT` | `8787` | The port the proxy server listens on |

## API Usage

Once the proxy is running, you can send requests to it as if it were the Kimi API:

```bash
# List models
curl http://localhost:8787/models \
  -H "Authorization: Bearer YOUR_KIMI_API_KEY"

# Chat completions (streaming)
curl http://localhost:8787/chat/completions \
  -H "Authorization: Bearer YOUR_KIMI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "kimi-k2",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'
```

## Zeroclaw Configuration

To use this proxy with Zeroclaw, set the API base URL to point to the proxy:

```bash
export ANTHROPIC_API_URL="http://localhost:8787"
```

Or configure Zeroclaw's settings to use `http://localhost:8787` as the API endpoint.

## How It Works

1. **Request Reception**: The proxy receives HTTP requests on all paths
2. **Header Processing**: 
   - Copies most headers from the original request
   - Removes the `Host` header (reqwest sets it automatically for upstream)
   - Sets `User-Agent: claude-code/0.1.0` to spoof the client
3. **Proxying**: Forwards the request to `https://api.kimi.com/coding/v1`
4. **Streaming Response**: Streams the response back to the client in chunks

## Architecture

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Zeroclaw   │ ──▶  │  Kimi Proxy │ ──▶  │  Kimi API   │
│  (Client)   │      │  (Rust)     │      │  (Upstream) │
└─────────────┘      └─────────────┘      └─────────────┘
                           │
                           ▼
                    User-Agent: claude-code/0.1.0
```

## Development

### Run in development mode with logs

```bash
RUST_LOG=debug cargo run
```

### Run tests

```bash
cargo test
```

## Comparison with Python Version

| Feature | Python (Flask) | Rust (Actix-web) |
|---------|---------------|------------------|
| Startup Time | ~1-2s | ~100ms |
| Memory Usage | ~50-100MB | ~10-20MB |
| Concurrency | Thread-based | Async/await |
| Throughput | Good | Excellent |
| Binary Size | N/A (interpreter) | ~5MB (static) |

## License

MIT

## Credits

- Original Python implementation: `kimi_proxy.py`
- Rust port using [Actix-web](https://actix.rs/) and [Reqwest](https://docs.rs/reqwest/)
