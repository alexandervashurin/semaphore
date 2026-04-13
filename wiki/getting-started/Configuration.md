# Configuration

> All configuration options for Velum
>
> 📖 See also: [[Quick Start]], [[Docker Deployment]], [[Auth & Security]]

---

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `VELUM_DB_URL` | PostgreSQL connection string | `postgres://user:pass@host:5432/velum` |
| `VELUM_DB_DIALECT` | Database dialect | `postgres` |
| `VELUM_JWT_SECRET` | JWT signing secret (min 32 chars) | `your-secret-key-32-bytes-long!!` |
| `VELUM_WEB_PATH` | Path to web UI files | `/app/web/public` |

### Admin User (First Run)

| Variable | Description | Default |
|----------|-------------|---------|
| `VELUM_ADMIN` | Admin username | `admin` |
| `VELUM_ADMIN_PASSWORD` | Admin password | `admin123` |
| `VELUM_ADMIN_NAME` | Admin display name | `Administrator` |
| `VELUM_ADMIN_EMAIL` | Admin email | `admin@velum.local` |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `VELUM_LDAP_*` | LDAP configuration | — |
| `VELUM_OIDC_*` | OIDC providers | — |
| `RUST_LOG` | Logging level | `info` |
| `VELUM_HA_REDIS_HOST` | HA Redis host | — |
| `VELUM_HA_REDIS_PORT` | HA Redis port | — |

---

## Docker Configuration

See [[Docker Deployment]] for all compose variants.

---

## Advanced

### Redis Task Queue
```bash
VELUM_REDIS_URL=redis://localhost:6379
```

### Telegram Bot
```bash
VELUM_TELEGRAM_TOKEN=your-bot-token
VELUM_TELEGRAM_CHAT_ID=-1001234567890
```

### Logging
```bash
RUST_LOG=velum=debug,tower_http=debug
```

---

## Next Steps

- [[First Project]] — create your first project
- [[Auth & Security]] — configure LDAP, OIDC, TOTP
- [[Production Setup]] — harden for production
