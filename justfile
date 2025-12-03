# List available commands
default:
    @just --list

# Build debug binary
build:
    cargo build

# Build release binary
release:
    cargo build --release

# Install locally via cargo
install:
    cargo install --path .

# Uninstall
uninstall:
    cargo uninstall claude-sessions

# Run tests
test:
    cargo test

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# Check everything before commit
check: fmt lint test

# Clean build artifacts
clean:
    cargo clean

# Run the CLI (debug build)
run *args:
    cargo run -- {{args}}

# Show binary size (release)
size: release
    @ls -lh target/release/claude-sessions | awk '{print $5, $9}'
