# Contributing Guide

> How to contribute to Velum
>
> 📖 See also: [[Code of Conduct]], [[Security Policy]], [[Development Setup]]

---

## Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Run tests: `cargo test --all-features`
5. Run clippy: `cargo clippy --all-features -- -D warnings`
6. Commit and push
7. Open a Pull Request

---

## Code Style

- **Rust**: Follow `rustfmt` conventions
- **Commits**: Conventional commits (`feat:`, `fix:`, `docs:`, etc.)
- **Tests**: Every new function should have tests

---

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add entry to `CHANGELOG.md`
4. Request review from maintainers

---

## Branch Protection

- `main` — protected, requires PR review
- `develop` — integration branch
- `feat/*` — feature branches
- `fix/*` — bug fix branches

---

## CI/CD

Every PR triggers:
- Format check
- Build + Clippy
- Test suite (6551+ tests)
- Security audit

See [[Build Pipeline]] for details.

---

## Next Steps

- [[Code of Conduct]] — community guidelines
- [[Security Policy]] — report vulnerabilities
- [[Development Setup]] — local dev environment
