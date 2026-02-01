from __future__ import annotations

from typing import Any

import httpx


class OpenAIResponsesExecutor:
    """
    Minimal OpenAI-style batch executor for HTTP APIs that accept a list of prompts.

    This adapter is intentionally generic and can be replaced or customized.
    """

    def __init__(
        self,
        api_key: str,
        model: str,
        base_url: str = "https://api.openai.com/v1/responses",
        timeout_s: float = 30.0,
    ) -> None:
        self.api_key = api_key
        self.model = model
        self.base_url = base_url
        self.timeout_s = timeout_s

    def __call__(self, prompts: list[str]) -> list[str]:
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": self.model,
            "input": [{"role": "user", "content": p} for p in prompts],
        }

        with httpx.Client(timeout=self.timeout_s) as client:
            resp = client.post(self.base_url, headers=headers, json=payload)
            resp.raise_for_status()
            data = resp.json()

        outputs: list[str] = []
        for item in data.get("output", []):
            for content in item.get("content", []):
                if content.get("type") == "output_text":
                    outputs.append(content.get("text", ""))

        if len(outputs) != len(prompts):
            raise RuntimeError(
                f"Executor returned {len(outputs)} items for {len(prompts)} inputs"
            )

        return outputs
