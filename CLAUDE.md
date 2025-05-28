# FerrisDB Development Guidelines

## Project Overview
FerrisDB is a distributed, transactional key-value database inspired by FoundationDB, implemented in Rust.

## Development Standards

### Code Style
- Follow Rust standard formatting with `rustfmt`
- Use `clippy` for linting
- Maximum line length: 100 characters
- Use descriptive variable names
- Prefer `snake_case` for functions and variables, `CamelCase` for types

### Testing
- Write unit tests for all public APIs
- Integration tests for distributed scenarios
- Use `proptest` for property-based testing
- Test coverage target: >80%

### Commands
```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all

# Run benchmarks
cargo bench

# Build all workspace members
cargo build --all

# Run with logging
RUST_LOG=debug cargo run
```

### Git Workflow
- Main branch: `main`
- Feature branches: `feature/description`
- Bug fixes: `fix/description`
- Commit messages: Use conventional commits format

### Architecture Decisions
- Use `tokio` for async runtime
- RocksDB for storage engine (may change to custom LSM-tree)
- gRPC (Tonic) for network protocol
- Raft for consensus (evaluate existing libraries first)

### Error Handling
- Use `thiserror` for error types
- Always propagate errors with context
- Log errors at appropriate levels
- Never panic in production code paths

### Performance Guidelines
- Profile before optimizing
- Use `criterion` for benchmarks
- Avoid unnecessary allocations
- Prefer zero-copy operations where possible

### Documentation
- Document all public APIs
- Include examples in doc comments
- Keep DESIGN.md updated with architectural changes
- Write ADRs (Architecture Decision Records) for significant changes

### Security
- Never log sensitive data
- Use TLS for network communication
- Validate all inputs
- Follow principle of least privilege

## Project Structure
```
ferrisdb/
├── ferrisdb-core/       # Common types and traits
├── ferrisdb-storage/    # Storage engine
├── ferrisdb-client/     # Client library
├── ferrisdb-server/     # Server implementation
├── tests/              # Integration tests
├── benches/            # Benchmarks
└── docs/               # Additional documentation
```

## Key Invariants
1. Transactions must be serializable
2. All writes must be durable before acknowledgment
3. Node failures must not cause data loss
4. Reads must see a consistent snapshot

## Dependencies
- Prefer well-maintained, popular crates
- Minimize dependency count
- Pin major versions in Cargo.toml
- Review security advisories regularly

## Debugging Tips
- Use `RUST_LOG=trace` for detailed logging
- Enable debug symbols in release builds for profiling
- Use `tokio-console` for async runtime debugging
- Capture traces for distributed debugging