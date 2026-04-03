# Velum Documentation

This directory contains the generated Rust API documentation for Velum.

## Viewing Documentation

### Online

The latest documentation is automatically published to:
**https://tnl-o.github.io/velum/**

### Local

To generate documentation locally:

```bash
cd rust
cargo doc --no-deps --open
```

This will:
1. Build the documentation
2. Open it in your default web browser

## Documentation Structure

```
target/doc/
├── velum_ffi/          # Main library documentation
│   ├── all.html        # All items index
│   ├── index.html      # Crate root
│   ├── sidebar-items.js
│   └── ...
├── crates.js           # Crate index
└── index.html          # Redirect to main crate
```

## Documented Modules

| Module | Description |
|--------|-------------|
| `api` | HTTP API handlers (Axum), REST, GraphQL, WebSocket, MCP |
| `config` | Application configuration (HA, LDAP, OIDC) |
| `db` | Database layer (SQLx, PostgreSQL/MySQL/SQLite) |
| `kubernetes` | Kubernetes client (kube 0.98 + k8s-openapi 0.24) |
| `models` | Data models |
| `services` | Business logic (task runner, scheduler, notifications) |
| `utils` | Utilities (encryption, SSH, mailer) |
| `validators` | Validators (cron, YAML, schedules) |

## Code Examples

The documentation includes code examples marked with `no_run`:
- Server startup example
- Kubernetes client usage
- API handler examples

## Contributing

When adding new modules or functions, please follow Rust documentation conventions:

1. **Module-level docs** (`mod.rs`):
   ```rust
   //! Brief description of the module
   //!
   //! # Examples
   //!
   //! ```
   //! use velum::mymodule::MyStruct;
   //! ```
   ```

2. **Function-level docs**:
   ```rust
   /// Brief description
   ///
   /// # Arguments
   ///
   /// * `arg1` - Description of arg1
   ///
   /// # Returns
   ///
   /// Description of return value
   ///
   /// # Errors
   ///
   /// Returns error if...
   ///
   /// # Example
   ///
   /// ```
   /// let result = my_function(42);
   /// ```
   pub fn my_function(arg1: i32) -> Result<()> {
       // ...
   }
   ```

3. **Struct/Enum docs**:
   ```rust
   /// Represents a...
   ///
   /// # Fields
   ///
   /// * `field1` - Description
   /// * `field2` - Description
   ///
   /// # Example
   ///
   /// ```
   /// let s = MyStruct { field1: value };
   /// ```
   pub struct MyStruct {
       pub field1: Type,
       pub field2: Type,
   }
   ```

## Documentation Tests

To run documentation tests:

```bash
cd rust
cargo test --doc
```

## Documentation Linting

Check for documentation warnings:

```bash
cd rust
cargo doc --no-deps
```

Fix common issues:
- Use backticks for code: \`code\`
- Use angle brackets for URLs: \<https://example.com>
- Escape HTML tags in examples

## Publishing to crates.io

To publish the library to crates.io:

```bash
cd rust
cargo publish
```

**Note:** This requires a valid `CARGO_REGISTRY_TOKEN` environment variable.

## Version Compatibility

| Velum Version | Rust Edition | Documentation |
|---------------|--------------|---------------|
| 2.4.0+ | 2021 | Full module docs |
| 2.0.0 - 2.3.x | 2021 | Partial docs |

## Related Resources

- [GitHub Repository](https://github.com/tnl-o/velum)
- [User Guide](../docs/guides/)
- [API Documentation](../api-docs.yml)
- [Rust Documentation Guidelines](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
