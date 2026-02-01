import torch

from llm_autobatch import Batcher
from llm_autobatch.torch import TorchExecutor


class Tiny(torch.nn.Module):
    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return x * 2


def collate(items: list[torch.Tensor]) -> torch.Tensor:
    return torch.stack(items)


if __name__ == "__main__":
    model = Tiny()
    executor = TorchExecutor(model=model, collate_fn=collate, device="cpu")
    batcher = Batcher(max_batch=8, max_wait_ms=5)

    out = batcher.run(torch.ones(2), executor=executor)
    print(out)
