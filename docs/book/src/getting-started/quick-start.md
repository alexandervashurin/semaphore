# Quick Start

> **⏱️ 5 minutes** — from zero to running Velum instance
>
> 📖 See also: [[Configuration]], [[Docker Deployment]], [[First Project]]

---

## Prerequisites

- Docker & Docker Compose **or** Rust 1.80+
- PostgreSQL 15+ (if not using Docker)

---

## Option 1: Docker (Recommended)

### Demo Mode
```bash
docker compose -f docker-compose.demo.yml up -d
```
Open http://localhost:8088 — Login: `admin` / `admin123`

### Development Mode
```bash
docker compose -f docker-compose.dev.yml up -d
```
Open http://localhost:3000

---

## Option 2: From Source

```bash
# Clone
git clone https://github.com/alexandervashurin/semaphore.git
cd semaphore

# Build
cd rust && cargo build --release

# Run
VELUM_DB_DIALECT=postgres \
VELUM_DB_URL="postgres://user:pass@localhost:5432/velum" \
VELUM_JWT_SECRET="your-secret-key-32-bytes-long!!" \
VELUM_WEB_PATH="../web/public" \
VELUM_ADMIN=admin \
VELUM_ADMIN_PASSWORD=admin123 \
./target/release/velum server --host 0.0.0.0 --port 3000
```

---

## Verify Installation

```bash
# Health check
curl http://localhost:3000/healthz

# Ready check
curl http://localhost:3000/readyz

# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"admin123"}'
```

---

## Next Steps

1. [[Configuration]] — customize your setup
2. [[First Project]] — create a project and run your first task
3. [[Docker Deployment]] — production deployment options
4. [[API Reference]] — explore the full API

---

## Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| Linux | amd64, arm64 | ✅ Native |
| macOS | amd64 (Intel), arm64 (Apple Silicon) | ✅ Native |
| Docker | linux/amd64, linux/arm64 | ✅ Multi-arch |
| Kubernetes | amd64, arm64 | ✅ Helm + manifests |
