from __future__ import annotations

from typing import Any, Callable

import torch


class TorchExecutor:
    """
    Basic PyTorch batch executor.

    collate_fn: function that transforms list[Any] -> batched tensors/inputs.
    """

    def __init__(
        self,
        model: torch.nn.Module,
        collate_fn: Callable[[list[Any]], Any],
        device: str = "cpu",
    ) -> None:
        self.model = model.to(device)
        self.collate_fn = collate_fn
        self.device = device

    def __call__(self, items: list[Any]) -> list[Any]:
        batch = self.collate_fn(items)
        if hasattr(batch, "to"):
            batch = batch.to(self.device)

        self.model.eval()
        with torch.no_grad():
            outputs = self.model(batch)

        if isinstance(outputs, (list, tuple)):
            return list(outputs)

        return [outputs] * len(items)
