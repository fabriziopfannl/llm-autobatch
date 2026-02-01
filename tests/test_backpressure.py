import threading
import time

import pytest

pytest.importorskip("llm_autobatch._native")

from llm_autobatch import Batcher


def test_backpressure_drop():
    batcher = Batcher(max_batch=2, max_wait_ms=50, backpressure="drop")

    def slow_executor(items: list[int]) -> list[int]:
        time.sleep(0.05)
        return items

    errors: list[Exception] = []

    def worker(x: int) -> None:
        try:
            batcher.run(x, executor=slow_executor)
        except Exception as exc:  # noqa: BLE001
            errors.append(exc)

    threads = [threading.Thread(target=worker, args=(i,)) for i in range(10)]
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    assert errors, "Expected some drops when queue is full"
