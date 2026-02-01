import time
import threading

from llm_autobatch import Batcher


def main() -> None:
    batcher = Batcher(max_batch=64, max_wait_ms=5)

    def executor(items: list[int]) -> list[int]:
        return items

    total = 10000
    start = time.perf_counter()

    def worker(i: int) -> None:
        batcher.run(i, executor=executor)

    threads = [threading.Thread(target=worker, args=(i,)) for i in range(total)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    dur = time.perf_counter() - start
    avg_batch = batcher.metrics().get("total_items", 0) / max(
        batcher.metrics().get("total_batches", 1), 1
    )

    print(
        f"items={total} max_batch=64 max_wait_ms=5  avg_batch={avg_batch:.1f}  seconds={dur:.2f}"
    )


if __name__ == "__main__":
    main()
