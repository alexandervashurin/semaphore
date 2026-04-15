# Troubleshooting

> Common issues and solutions
>
> 📖 See also: [[Debug Mode]], [[Migration Guide]], [[Configuration]]

---

## Common Issues

### Server won't start

**Check logs:**
```bash
RUST_LOG=debug ./target/release/velum server
```

**Common causes:**
- Database not accessible — check `VELUM_DB_URL`
- Port already in use — change `--port` flag
- Missing web files — verify `VELUM_WEB_PATH`

### Can't login

**Reset admin password:**
```bash
VELUM_ADMIN_PASSWORD=newpassword ./target/release/velum server
```

### Tasks fail

**Check runner logs:**
```bash
RUST_LOG=velum=debug ./target/release/velum server
```

**Common causes:**
- Missing Ansible/Terraform — install on runner host
- SSH key issues — verify access keys
- Repository not accessible — check network

---

## Debug Mode

Enable verbose logging:
```bash
RUST_LOG=velum=debug,tower_http=debug
```

---

## Database Issues

### Migration errors
```bash
# Check current schema
psql -U velum -d velum -c "\dt"

# Reset (WARNING: deletes all data)
psql -U velum -d velum -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
```

---

## Docker Issues

### Container won't start
```bash
docker logs velum-server
```

### Database not ready
```bash
docker compose -f docker-compose.yml up db -d
sleep 10  # wait for DB
docker compose -f docker-compose.yml up -d
```

---

## Next Steps

- [[Debug Mode]] — more debugging options
- [[Configuration]] — environment variables
- [[Development Setup]] — local dev environment
