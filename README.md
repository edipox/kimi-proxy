# Kimi Proxy

A lightweight HTTP proxy that forwards requests to the [Kimi API](https://api.kimi.com/coding/v1) with User-Agent spoofing for Zeroclaw compatibility.

## Quick Start

```bash
# 1. Clone the repo
git clone https://github.com/edipox/kimi-proxy.git
cd kimi-proxy

# 2. Run it
cargo run --release

# 3. Use it (in another terminal)
curl http://localhost:8787/models \
  -H "Authorization: Bearer YOUR_KIMI_API_KEY"
```

The proxy runs on port `8787` by default. Set `PORT` env var to change it.

## Zeroclaw Setup

```bash
export ANTHROPIC_API_URL="http://localhost:8787"
```

## How It Works

Forwards all requests to `https://api.kimi.com/coding/v1` while setting `User-Agent: claude-code/0.1.0`.
