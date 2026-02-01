from __future__ import annotations

from typing import Any, Callable, TypeVar

from .batcher import Batcher

T = TypeVar("T")
R = TypeVar("R")


def autobatch(max_batch: int = 32, max_wait_ms: int = 10, backpressure: str = "block"):
    """
    Decorator for batching single-item calls via a list-based batch executor.

    The wrapped function must accept list[T] and return list[R].
    """

    def decorator(fn: Callable[[list[T]], list[R]]) -> Callable[[T], R]:
        batcher = Batcher(max_batch=max_batch, max_wait_ms=max_wait_ms, backpressure=backpressure)

        def executor(items: list[Any]) -> list[Any]:
            return fn(items)

        def wrapped(item: T) -> R:
            return batcher.run(item, executor=executor)

        return wrapped

    return decorator
