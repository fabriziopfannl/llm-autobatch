PYTHON ?= python3
VENV ?= .venv
ACTIVATE = . $(VENV)/bin/activate

.PHONY: venv fmt clippy build test check clean

venv:
	$(PYTHON) -m venv $(VENV)
	$(ACTIVATE); python -m pip install --upgrade pip
	$(ACTIVATE); python -m pip install maturin pytest ruff

fmt:
	cd rust && cargo fmt --all -- --check

clippy:
	cd rust && cargo clippy --all-targets --all-features -- -D warnings

build:
	$(ACTIVATE); python -m maturin develop

test:
	$(ACTIVATE); python -m pytest

check: fmt clippy build test

clean:
	rm -rf $(VENV) .pytest_cache rust/target
