from __future__ import annotations

from typing import Any, Callable

from . import _native

Executor = Callable[[list[Any]], list[Any]]


class Batcher:
    """
    Thin Python wrapper around the Rust batching core.

    The executor must be a callable that accepts a list of inputs and returns
    a list of outputs of the same length and order.
    """

    def __init__(
        self,
        max_batch: int = 32,
        max_wait_ms: int = 10,
        backpressure: str = "block",
    ) -> None:
        self._core = _native.BatcherCore(max_batch, max_wait_ms, backpressure)

    def run(self, item: Any, executor: Executor) -> Any:
        return self._core.submit(item, executor)

    def flush(self) -> None:
        self._core.flush()

    def metrics(self) -> dict[str, int]:
        return self._core.metrics().as_dict()

    def close(self) -> None:
        self._core.close()
