# Contributing

Thanks for your interest in contributing to llm-autobatch.

## Development setup

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install maturin pytest ruff
maturin develop
```

## Running tests

```bash
pytest
```

## Linting

```bash
ruff check .
```

## Rust checks

```bash
cd rust
cargo fmt --all
cargo clippy --all-targets --all-features -D warnings
cargo test
```

## Pull request checklist

- Add tests for new behavior
- Update docs if user-facing behavior changes
- Keep public API backward compatible unless explicitly planned
- Ensure CI passes
