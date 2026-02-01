# Roadmap

This roadmap reflects the planned evolution from a minimal, viral-ready v1 to a production-grade v2.

## v1 (viral + minimal)

- Rust batching core (single-process, multi-thread)
- Python Batcher + @autobatch decorator
- HTTP executor (generic callable + OpenAI-style example)
- Basic metrics:
  - average batch size
  - flush reason counts
- Benchmark script
- Minimal FastAPI demo
- Wheels + PyPI release pipeline

## v1.1

- Async support (run_async)
- Basic Torch executor (no DataLoader integration)

## v2

- Cross-process batching (shared memory or sockets)
- PyTorch DataLoader integration
  - prefetch
  - padding / packing
- Pluggable telemetry backends (OpenTelemetry, Prometheus)
- More backpressure policies (adaptive throttling)
