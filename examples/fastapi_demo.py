from fastapi import FastAPI

from llm_autobatch import autobatch

app = FastAPI()


@autobatch(max_batch=32, max_wait_ms=10)
def call_llm(prompts: list[str]) -> list[str]:
    return [p.upper() for p in prompts]


@app.get("/answer")
def answer(prompt: str) -> dict[str, str]:
    return {"answer": call_llm(prompt)}
