# IP Geolocation Service

A high-performance, standalone IP geolocation API service built in Rust. Provides IP-to-location and coordinates-to-timezone lookups with a simple REST API.

**Demo:** [geoip.vpetersson.com](https://geoip.vpetersson.com)

## Features

- **Fast**: Built with Axum and Tokio for high-concurrency async handling
- **Cacheable**: In-memory LRU cache + HTTP Cache-Control headers for proxy caching
- **Standalone**: All data bundled in the container (no external API dependencies)
- **Accurate**: Uses MaxMind GeoLite2 for IP geolocation, tzf-rs for timezone boundaries
- **LLM-Ready**: OpenAPI spec and llms.txt for AI agent integration
- **Protocol Buffers**: Optional protobuf responses for efficient binary serialization
- **HTTP/3 Ready**: Optional QUIC/HTTP/3 support for reduced latency

## API Endpoints

### Auto-detect Client IP

#### GET /

Returns geolocation data for the requesting client's IP address. Automatically detects the client IP from proxy headers (CF-Connecting-IP, X-Real-IP, X-Forwarded-For) or the direct connection.

**Example:**

```bash
curl "http://localhost:3000/"
```

**Response:**

```json
{
  "latitude": 51.5074,
  "longitude": -0.1278,
  "city": "London",
  "country_name": "United Kingdom",
  "time_zone": {
    "name": "Europe/London"
  },
  "languages": "en-GB,en"
}
```

---

### Simple Format (Backward Compatible)

#### GET /ipgeo

Returns basic geographic location data for a given IP address.

**Parameters:**

- `apiKey` (string, optional): API key (accepted but not validated)
- `ip` (string, required): IPv4 or IPv6 address to lookup

**Example:**

```bash
curl "http://localhost:3000/ipgeo?apiKey=test&ip=8.8.8.8"
```

**Response:**

```json
{
  "latitude": 37.751,
  "longitude": -97.822,
  "city": "Mountain View",
  "country_name": "United States",
  "time_zone": {
    "name": "America/Chicago"
  },
  "languages": "en-US,en"
}
```

#### GET /timezone

Returns timezone name for given geographic coordinates.

**Parameters:**

- `apiKey` (string, optional): API key (accepted but not validated)
- `lat` (float, required): Latitude coordinate
- `long` (float, required): Longitude coordinate

**Example:**

```bash
curl "http://localhost:3000/timezone?apiKey=test&lat=59.329504&long=18.069532"
```

**Response:**

```json
{
  "timezone": "Europe/Stockholm"
}
```

---

### Full Format (Extended)

#### GET /v1/ipgeo

Returns comprehensive location data with extended fields.

**Parameters:**

- `apiKey` (string, optional): API key (accepted but not validated)
- `ip` (string, required): IPv4 or IPv6 address to lookup

**Example:**

```bash
curl "http://localhost:3000/v1/ipgeo?apiKey=test&ip=8.8.8.8"
```

**Response:**

```json
{
  "ip": "8.8.8.8",
  "location": {
    "continent_code": "NA",
    "continent_name": "North America",
    "country_code2": "US",
    "country_code3": "USA",
    "country_name": "United States",
    "country_name_official": "United States of America",
    "country_capital": "Washington, D.C.",
    "state_prov": "California",
    "state_code": "US-CA",
    "city": "Mountain View",
    "zipcode": "94043",
    "latitude": "37.75100",
    "longitude": "-97.82200",
    "is_eu": false,
    "country_flag": "/static/flags/us.svg",
    "geoname_id": "5375480",
    "country_emoji": "🇺🇸"
  },
  "country_metadata": {
    "calling_code": "+1",
    "tld": ".us",
    "languages": ["en-US", "es-US"]
  },
  "currency": {
    "code": "USD",
    "name": "US Dollar",
    "symbol": "$"
  },
  "time_zone": {
    "name": "America/Los_Angeles",
    "offset": -8,
    "offset_with_dst": -8,
    "current_time": "2024-01-15 14:30:00.123-0800",
    "current_time_unix": 1705355400.123,
    "is_dst": false,
    "dst_savings": 1,
    "dst_exists": true
  }
}
```

#### GET /v1/timezone

Returns comprehensive timezone details for given coordinates.

**Parameters:**

- `apiKey` (string, optional): API key (accepted but not validated)
- `lat` (float, required): Latitude coordinate
- `long` (float, required): Longitude coordinate

**Example:**

```bash
curl "http://localhost:3000/v1/timezone?apiKey=test&lat=59.329504&long=18.069532"
```

**Response:**

```json
{
  "timezone": "Europe/Stockholm",
  "offset": 1,
  "offset_with_dst": 1,
  "current_time": "2024-01-15 23:30:00.123+0100",
  "current_time_unix": 1705355400.123,
  "is_dst": false,
  "dst_exists": true
}
```

---

### API Documentation

#### GET /openapi.yaml

Returns the OpenAPI 3.1 specification for the API. The spec is generated from the code using [utoipa](https://github.com/juhaku/utoipa), ensuring it's always in sync with the implementation.

**Example:**

```bash
curl "http://localhost:3000/openapi.yaml"
```

#### GET /llms.txt

Returns LLM-friendly documentation optimized for AI agent consumption. Includes API overview, use cases, quick reference, and examples.

**Example:**

```bash
curl "http://localhost:3000/llms.txt"
```

#### GET /sitemap.xml

Returns a standard sitemap for search engine and agent discovery.

**Example:**

```bash
curl "http://localhost:3000/sitemap.xml"
```

#### GET /robots.txt

Returns robots.txt for search engine crawlers.

**Example:**

```bash
curl "http://localhost:3000/robots.txt"
```

---

### Well-Known Discovery Endpoints

Standard discovery endpoints following web conventions:

#### GET /.well-known/openapi.yaml

OpenAPI specification at the standard well-known location.

#### GET /.well-known/ai-plugin.json

AI plugin manifest for ChatGPT-style agent discovery. Returns a JSON manifest describing the API capabilities and linking to the OpenAPI spec.

**Example Response:**

```json
{
  "schema_version": "v1",
  "name_for_human": "IP Geolocation API",
  "name_for_model": "ip_geolocation",
  "description_for_model": "Use this API to look up geographic location...",
  "api": {
    "type": "openapi",
    "url": "https://geoip.vpetersson.com/openapi.yaml"
  }
}
```

---

### Health Check

#### GET /health

Health check endpoint.

**Response:** `200 OK` with body `OK`

## Protocol Buffers (Protobuf) Support

All API endpoints support Protocol Buffer responses for efficient binary serialization. Use content negotiation via the `Accept` header.

### Request with Protobuf Response

```bash
curl -H "Accept: application/x-protobuf" "http://localhost:3000/ipgeo?ip=8.8.8.8" --output response.pb
```

### Protobuf Schemas

The `.proto` file is available at `proto/geolocation.proto` and includes:
- `IpGeoResponse` / `IpGeoResponseFull`
- `TimezoneResponse` / `TimezoneResponseFull`
- `ApiError`

### Benefits

- **~50% smaller** responses compared to JSON
- **Faster parsing** on the client side
- **Type-safe** with generated client libraries

---

## HTTP/3 (QUIC) Support

The API supports HTTP/3 over QUIC for reduced latency and improved connection performance. HTTP/3 requires TLS certificates.

### Enabling HTTP/3

Set the following environment variables:

```bash
HTTP3_ENABLED=true
HTTP3_BIND_ADDRESS=0.0.0.0:443
TLS_CERT_PATH=/path/to/cert.pem
TLS_KEY_PATH=/path/to/key.pem
```

### HTTP/3 Limitations

Due to the current HTTP/3 ecosystem in Rust, HTTP/3 currently supports these endpoints:
- `/health`
- `/openapi.yaml`
- `/llms.txt`
- `/.well-known/openapi.yaml`

For full API functionality (IP geolocation, timezone lookups), use HTTP/1.1 or HTTP/2.

### Alt-Svc Header

Responses include an `Alt-Svc` header to advertise HTTP/3 availability:
```
Alt-Svc: h3=":443"; ma=86400
```

---

## LLM and Agent Integration

This API is optimized for consumption by LLMs and AI agents:

- **OpenAPI Spec** (`/openapi.yaml`): Machine-readable API specification with full schemas, examples, and parameter documentation. Generated from code to ensure accuracy.
- **LLM Documentation** (`/llms.txt`): Human/AI readable context file explaining what the API does, when to use it, and how to call it.
- **MCP Server**: Native Model Context Protocol server for direct integration with Claude Desktop, Cursor, and other MCP clients.

### Example: Using with an AI Agent

```python
# Fetch the OpenAPI spec
import requests

spec = requests.get("https://geoip.vpetersson.com/openapi.yaml").text
# Parse and use with your agent framework (LangChain, AutoGPT, etc.)
```

---

## MCP Server (Model Context Protocol)

The IP Geolocation API includes native [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) support, enabling direct integration with Claude Desktop, Cursor, and other MCP-compatible clients.

### Unified Port Architecture

All protocols are served on the same port for simplicity:

| Protocol | Port | Transport | Endpoint |
|----------|------|-----------|----------|
| REST API (HTTP/1.1, HTTP/2) | 3000 | TCP | `/`, `/ipgeo`, `/v1/ipgeo`, etc. |
| MCP over HTTP | 3000 | TCP | `/mcp` (JSON-RPC), `/mcp/batch`, `/mcp/sse`, `/mcp/info` |
| HTTP/3 (QUIC) | 443 | UDP | Same endpoints (optional, requires TLS) |
| MCP over STDIO | N/A | STDIO | For local clients (Claude Desktop) |

### MCP Endpoints (HTTP)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/mcp` | POST | JSON-RPC 2.0 endpoint for MCP requests |
| `/mcp/batch` | POST | Batch JSON-RPC 2.0 endpoint for multiple requests |
| `/mcp/sse` | GET | Server-Sent Events for real-time notifications |
| `/mcp/info` | GET | Server capabilities and tool discovery |

### MCP Tools

| Tool | Description |
|------|-------------|
| `geoip_lookup` | Look up geographic location for an IP address. Returns city, country, coordinates, timezone, currency, and other metadata. |
| `geoip_bulk_lookup` | Look up multiple IP addresses (max 100). Returns results and errors separately. |
| `geoip_lookup_self` | Look up the caller's IP address. Available via HTTP transport. |
| `timezone_lookup` | Look up IANA timezone for coordinates. Returns timezone name, offset, DST info, and current time. |

### MCP Resources

| URI | Description |
|-----|-------------|
| `geoip://schema` | JSON Schema definitions for all response types |
| `geoip://data-source` | Information about MaxMind GeoLite2, tzf-rs, and other data sources |
| `geoip://limits` | API limits (bulk cap: 100, cache TTL, etc.) |
| `geoip://privacy` | Privacy practices: no IP logging, no PII retention, stateless lookups |

### Using MCP over HTTP

```bash
# Initialize the MCP session
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}'

# List available tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}'

# Call a tool
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "geoip_lookup", "arguments": {"ip": "8.8.8.8"}}}'

# Get server info
curl http://localhost:3000/mcp/info
```

### STDIO Transport (for Claude Desktop)

For local MCP clients, use the standalone STDIO server:

```bash
# Build the STDIO server
cargo build --release --bin mcp_server

# Run with STDIO transport
./target/release/mcp_server --transport stdio
```

### Claude Desktop Configuration

Add to your Claude Desktop configuration (`~/.config/claude/claude_desktop_config.json` on macOS/Linux):

```json
{
  "mcpServers": {
    "ip-geolocation": {
      "command": "/path/to/mcp_server",
      "args": ["--transport", "stdio"],
      "env": {
        "GEOIP_DB_PATH": "/path/to/GeoLite2-City.mmdb"
      }
    }
  }
}
```

### Cursor Configuration

Add to your Cursor MCP settings:

```json
{
  "mcp": {
    "servers": {
      "ip-geolocation": {
        "command": "/path/to/mcp_server",
        "args": ["--transport", "stdio"],
        "env": {
          "GEOIP_DB_PATH": "/path/to/GeoLite2-City.mmdb"
        }
      }
    }
  }
}
```

### Example Tool Usage

Once connected via MCP, you can use tools like:

**Look up an IP address:**
```
Use geoip_lookup to find the location of 8.8.8.8
```

**Look up timezone for coordinates:**
```
Use timezone_lookup to find the timezone for latitude 51.5074, longitude -0.1278
```

**Bulk lookup:**
```
Use geoip_bulk_lookup to find locations for these IPs: 8.8.8.8, 1.1.1.1, 9.9.9.9
```

## Configuration

Environment variables:

| Variable             | Default                          | Description                          |
| -------------------- | -------------------------------- | ------------------------------------ |
| `BIND_ADDRESS`       | `0.0.0.0:3000`                   | HTTP/1.1+2 server bind address       |
| `GEOIP_DB_PATH`      | `data/GeoLite2-City.mmdb`        | Path to MaxMind database             |
| `STATIC_DIR`         | `static`                         | Directory for static assets (flags)  |
| `CACHE_SIZE`         | `10000`                          | Max entries in IP lookup cache       |
| `CACHE_TTL_SECS`     | `3600`                           | Cache entry TTL in seconds           |
| `RUST_LOG`           | `ipgeolocation=info`             | Log level                            |
| `BASE_URL`           | `https://geoip.vpetersson.com`   | Base URL for OpenAPI, sitemap, etc.  |
| `HTTP3_ENABLED`      | `false`                          | Enable HTTP/3 server                 |
| `HTTP3_BIND_ADDRESS` | `0.0.0.0:443`                    | HTTP/3 server bind address (UDP)     |
| `TLS_CERT_PATH`      | `cert.pem`                       | Path to TLS certificate (PEM)        |
| `TLS_KEY_PATH`       | `key.pem`                        | Path to TLS private key (PEM)        |

## Building

### Prerequisites

1. **Rust toolchain**: Install via [rustup](https://rustup.rs/)
2. **MaxMind GeoLite2 database**: Free account required at [MaxMind](https://www.maxmind.com/en/geolite2/signup)

### Local Development

```bash
# Download GeoLite2-City.mmdb and place in data/ directory
mkdir -p data
# Download from MaxMind portal or use geoipupdate tool

# Build and run
cargo build --release
GEOIP_DB_PATH=data/GeoLite2-City.mmdb ./target/release/ipgeolocation
```

### Docker

The Dockerfile uses [Chainguard's secure container images](https://images.chainguard.dev/directory/image/rust/overview) for minimal attack surface and built-in security scanning via `cargo-auditable`.

```bash
# Build with MaxMind credentials (downloads database during build)
docker build \
  --build-arg MAXMIND_ACCOUNT_ID=your_account_id \
  --build-arg MAXMIND_LICENSE_KEY=your_license_key \
  -t ipgeolocation .

# Or build without credentials and mount database at runtime
docker build -t ipgeolocation .
docker run -v /path/to/GeoLite2-City.mmdb:/app/data/GeoLite2-City.mmdb -p 3000:3000 ipgeolocation

# Run
docker run -p 3000:3000 ipgeolocation
```

**Image contents:**

- GeoLite2-City database (downloaded during build if credentials provided)
- Country flag SVGs from [flag-icons](https://github.com/lipis/flag-icons) (MIT license)

**Image stack:**

- Build: `cgr.dev/chainguard/rust` (includes cargo-auditable for SBOM)
- Runtime: `cgr.dev/chainguard/glibc-dynamic` (minimal, CVE-free base)

## Testing

```bash
cargo test
```

## Data Sources

- **IP Geolocation**: [MaxMind GeoLite2-City](https://dev.maxmind.com/geoip/geolite2-free-geolocation-data) (CC BY-SA 4.0, requires free registration)
- **Timezone Boundaries**: [tzf-rs](https://crates.io/crates/tzf-rs) (MIT, compiled into binary)
- **Country Flags**: [flag-icons](https://github.com/lipis/flag-icons) (MIT, bundled SVGs)
- **Languages**: Embedded country-to-language mapping

## Attribution

This product includes GeoLite2 Data created by MaxMind, available from [https://www.maxmind.com](https://www.maxmind.com).

## License

MIT

The GeoLite2 database is licensed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
