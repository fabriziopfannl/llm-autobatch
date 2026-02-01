import json

import pytest

httpx = pytest.importorskip("httpx")

from llm_autobatch.http import OpenAIResponsesExecutor


def test_http_executor_parses_output(monkeypatch: pytest.MonkeyPatch):
    def handler(request: httpx.Request) -> httpx.Response:
        payload = json.loads(request.content.decode())
        assert payload["model"] == "test"
        out = {
            "output": [
                {
                    "content": [
                        {"type": "output_text", "text": "ok-1"},
                        {"type": "output_text", "text": "ok-2"},
                    ]
                }
            ]
        }
        return httpx.Response(200, json=out)

    transport = httpx.MockTransport(handler)

    class PatchedClient(httpx.Client):
        def __init__(self, *args, **kwargs):
            super().__init__(transport=transport, *args, **kwargs)

    monkeypatch.setattr(httpx, "Client", PatchedClient)

    execu = OpenAIResponsesExecutor(api_key="x", model="test")
    result = execu(["a", "b"])
    assert result == ["ok-1", "ok-2"]
