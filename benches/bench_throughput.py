import os
import time
from concurrent.futures import ThreadPoolExecutor, wait

from llm_autobatch import Batcher


def main() -> None:
    batcher = Batcher(max_batch=64, max_wait_ms=5)

    def executor(items: list[int]) -> list[int]:
        return items

    total = 10000
    # Warmup is 10% of total, clamped to a sensible range.
    warmup = min(2000, max(200, total // 10))
    # Cap workers to avoid oversubscription; enough to saturate most systems.
    max_workers = min(32, (os.cpu_count() or 1) * 4)

    def worker(i: int) -> None:
        batcher.run(i, executor=executor)

    with ThreadPoolExecutor(max_workers=max_workers) as pool:
        # Warmup to avoid including lazy init or thread startup overhead.
        warm_futures = [pool.submit(worker, i) for i in range(warmup)]
        wait(warm_futures)

        baseline = batcher.metrics()
        start = time.perf_counter()
        futures = [pool.submit(worker, i) for i in range(total)]
        wait(futures)
        dur = time.perf_counter() - start

    metrics = batcher.metrics()
    total_items = metrics.get("total_items", 0) - baseline.get("total_items", 0)
    total_batches = metrics.get("total_batches", 0) - baseline.get("total_batches", 0)
    avg_batch = total_items / max(total_batches, 1)

    print(
        f"items={total} max_batch=64 max_wait_ms=5  avg_batch={avg_batch:.1f}  seconds={dur:.2f}  items_per_sec={total_items/dur:.1f}  max_workers={max_workers} warmup={warmup}"
    )


if __name__ == "__main__":
    main()
