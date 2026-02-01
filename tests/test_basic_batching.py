import threading

import pytest

pytest.importorskip("llm_autobatch._native")

from llm_autobatch import Batcher


def test_basic_batching_order():
    batcher = Batcher(max_batch=8, max_wait_ms=10)

    def executor(items: list[int]) -> list[int]:
        return [i * 2 for i in items]

    results: list[int] = []

    def worker(x: int) -> None:
        results.append(batcher.run(x, executor=executor))

    threads = [threading.Thread(target=worker, args=(i,)) for i in range(20)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert sorted(results) == sorted([i * 2 for i in range(20)])
