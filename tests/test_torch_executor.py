import pytest

torch = pytest.importorskip("torch")

from llm_autobatch.torch import TorchExecutor


def test_torch_executor():
    model = torch.nn.Linear(2, 2, bias=False)

    def collate(items: list[torch.Tensor]) -> torch.Tensor:
        return torch.stack(items)

    execu = TorchExecutor(model=model, collate_fn=collate, device="cpu")
    items = [torch.ones(2), torch.zeros(2)]
    out = execu(items)
    assert len(out) == 2
