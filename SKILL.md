# llm-autobatch Skill

This skill describes how to use the llm-autobatch repo effectively in local development.

## What this repo does

- Provides a Rust-backed batching core via PyO3
- Exposes a simple Python API (`Batcher`, `@autobatch`)
- Includes HTTP and Torch executors for common LLM workflows

## Local development

1) Build the native extension

```bash
maturin develop
```

2) Run tests

```bash
pytest
```

3) Run the throughput benchmark

```bash
python benches/bench_throughput.py
```

## Key directories

- `rust/`: Rust core implementation
- `src/llm_autobatch/`: Python API + adapters
- `tests/`: Unit tests
- `examples/`: Usage demos
- `docs/`: Roadmap and docs

## Conventions

- Executors must return a list with the same length and order as inputs.
- All documentation and examples remain in English.
