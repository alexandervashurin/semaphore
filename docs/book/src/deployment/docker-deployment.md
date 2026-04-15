# Docker Deployment

> All Docker Compose deployment options
>
> 📖 See also: [[Quick Start]], [[Configuration]], [[Kubernetes Deployment]], [[Production Setup]]

---

## Available Compose Files

| File | Purpose | Port |
|------|---------|------|
| `docker-compose.demo.yml` | Quick demo with demo data | 8088 |
| `docker-compose.dev.yml` | Development with hot reload | 3000 |
| `docker-compose.yml` | Default production setup | 3000 |
| `docker-compose.prod.yml` | Production hardening | 3000 |
| `docker-compose.postgres.yml` | PostgreSQL only | — |

---

## Demo Mode

Fastest way to try Velum:

```bash
docker compose -f docker-compose.demo.yml up -d
```

Opens at http://localhost:8088 — Login: `admin` / `admin123`

---

## Development Mode

With hot code reload:

```bash
docker compose -f docker-compose.dev.yml up -d
```

---

## Production Mode

```bash
docker compose -f docker-compose.prod.yml up -d
```

Includes:
- Health checks
- Resource limits
- Restart policies
- Volume persistence

---

## Docker Image

The optimized image is **~23MB** (FROM scratch + shared libraries):

```bash
docker pull ghcr.io/alexandervashurin/semaphore:latest
```

### Multi-arch Support

| Platform | Tag |
|----------|-----|
| Linux amd64 | `latest`, `linux-amd64` |
| Linux arm64 | `latest`, `linux-arm64` |

---

## Custom Build

```bash
docker build -t velum:local .
```

---

## Next Steps

- [[Kubernetes Deployment]] — deploy to K8s
- [[Production Setup]] — harden for production
- [[Configuration]] — environment variables
