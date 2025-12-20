# Remove build artifacts
@clean:
    echo "{{BLUE}}Removing linux-rs build artifacts...{{NORMAL}}"
    cargo clean

# Format and lint
@check:
    echo "{{BLUE}}Formatting and linting linux-rs...{{NORMAL}}"
    cargo +nightly fmt --all
    cargo +nightly clippy --workspace --all-features -- -D warnings

# Sort dependencies
@sort:
    echo "{{BLUE}}Sorting linux-rs dependencies...{{NORMAL}}"
    cargo sort -g

# Update dependencies
@update:
    echo "{{BLUE}}Updating linux-rs dependencies...{{NORMAL}}"
    cargo update

# Check for outdated dependencies
@outdated:
    echo "{{BLUE}}Checking for outdated linux-rs dependencies...{{NORMAL}}"
    cargo outdated -R

# Debug build
@debug:
    echo "{{BLUE}}Building linux-rs in debug mode...{{NORMAL}}"
    cargo build

# Release build
@release:
    echo "{{BLUE}}Building linux-rs in release mode...{{NORMAL}}"
    cargo build --release