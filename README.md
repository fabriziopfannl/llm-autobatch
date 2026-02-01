# llm-autobatch

Production-minded micro-batching for LLM calls and local PyTorch inference, backed by a single Rust core.

- **Viral simple:** `@autobatch` turns single calls into efficient batches.
- **Adapter-based:** swap HTTP or Torch executors without changing the core.
- **Rust-fast:** thread-safe queues, micro-windows, and backpressure.

## 60-second Quickstart

```bash
pip install llm-autobatch
```

```python
from llm_autobatch import autobatch

@autobatch(max_batch=32, max_wait_ms=10)
def call_llm(prompts: list[str]) -> list[str]:
    # Replace with a real batch call
    return [p.upper() for p in prompts]

print(call_llm("hello"))
```

**Object-based API**

```python
from llm_autobatch import Batcher

batcher = Batcher(max_batch=32, max_wait_ms=10)

def batch_executor(items: list[str]) -> list[str]:
    return [s + "!" for s in items]

print(batcher.run("hi", executor=batch_executor))
```

**HTTP adapter (OpenAI-style)**

```python
from llm_autobatch.http import OpenAIResponsesExecutor
from llm_autobatch import Batcher

executor = OpenAIResponsesExecutor(api_key="...", model="gpt-4o-mini")
batcher = Batcher(max_batch=32, max_wait_ms=10)

out = batcher.run("Explain Rust ownership", executor=executor)
print(out)
```

**Torch adapter**

```python
from llm_autobatch.torch import TorchExecutor
from llm_autobatch import Batcher

executor = TorchExecutor(model=model, collate_fn=collate, device="cuda")
batcher = Batcher(max_batch=64, max_wait_ms=5)

print(batcher.run(x, executor=executor))
```

## Benchmark

Run a local throughput test:

```bash
python benches/bench_throughput.py
```

Sample output (illustrative):

```
items=10000 max_batch=64 max_wait_ms=5  avg_batch=42.7  p99_ms=11.2
```

## Why Rust?

- **Deterministic batching windows** without Python GIL bottlenecks
- **Low-latency coordination** under high concurrency
- **Single core** reused across HTTP and Torch adapters
- **Memory safety** while handling multithreaded queues

## FAQ

**Does this change my model API?**
No. You keep your executor; the core only handles batching and routing.

**Do I need Rust installed?**
No. We publish prebuilt wheels for macOS, Linux, and Windows. `pip install llm-autobatch` should work without Rust.

**How do I enable HTTP or Torch adapters?**
Install extras:

```bash
pip install llm-autobatch[http]
pip install llm-autobatch[torch]
```

**What does backpressure do?**
- `block`: wait for space
- `drop`: reject when full
- `passthrough`: execute immediately

**Can I use async?**
Not in v1. Async support is planned for v1.1.

**Is ordering preserved?**
Yes. Outputs must match the input order for each batch.

## License

MIT
