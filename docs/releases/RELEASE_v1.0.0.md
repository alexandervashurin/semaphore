# Velum v1.0.0

Стабильный baseline релиз Rust-версии Velum с Kubernetes UI, WebSocket операциями и обновлённой документацией/CI.

## Highlights

- Kubernetes UI: inline YAML dry-run для Pods/Deployments
- OpenAPI: актуальные WS endpoint'ы (`exec`, `portforward`) и схемы workload list
- CI: покрытие тестов через `cargo-tarpaulin` (artifact `cobertura.xml`)
- Release automation: Linux binaries для `amd64` и `arm64`

## Assets

- `velum-linux-amd64`
- `velum-linux-arm64`

## Quick Start

```bash
chmod +x velum-linux-amd64
./velum-linux-amd64 server --host 0.0.0.0 --port 3000
```

## Notes

- В релиз включены только Linux binaries (amd64/arm64).
- Mobile K8s UI + WCAG improvements зафиксированы как optional backlog для v2.
