# IP Geolocation Service

A high-performance, standalone IP geolocation API service built in Rust. Provides IP-to-location and coordinates-to-timezone lookups with a simple REST API.

**Demo:** [geoip.vpetersson.com](https://geoip.vpetersson.com)

## Features

- **Fast**: Built with Axum and Tokio for high-concurrency async handling
- **Cacheable**: In-memory LRU cache + HTTP Cache-Control headers for proxy caching
- **Standalone**: All data bundled in the container (no external API dependencies)
- **Accurate**: Uses MaxMind GeoLite2 for IP geolocation, tzf-rs for timezone boundaries

## API Endpoints

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

### Health Check

#### GET /health

Health check endpoint.

**Response:** `200 OK` with body `OK`

## Configuration

Environment variables:

| Variable         | Default                   | Description                         |
| ---------------- | ------------------------- | ----------------------------------- |
| `BIND_ADDRESS`   | `0.0.0.0:3000`            | Server bind address                 |
| `GEOIP_DB_PATH`  | `data/GeoLite2-City.mmdb` | Path to MaxMind database            |
| `STATIC_DIR`     | `static`                  | Directory for static assets (flags) |
| `CACHE_SIZE`     | `10000`                   | Max entries in IP lookup cache      |
| `CACHE_TTL_SECS` | `3600`                    | Cache entry TTL in seconds          |
| `RUST_LOG`       | `ipgeolocation=info`      | Log level                           |

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
