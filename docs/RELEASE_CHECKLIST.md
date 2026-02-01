# Release Checklist

Use this checklist to cut the first release (v0.1.0) and future versions.

## 1) Prep

- [ ] Update version in `pyproject.toml` and `rust/Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Ensure README and docs are up to date
- [ ] Run tests: `pytest`
- [ ] Run Rust checks: `cargo fmt`, `cargo clippy`, `cargo test`

## 2) Build

- [ ] Build wheels locally: `maturin build --release --sdist`
- [ ] Inspect `dist/` artifacts
- [ ] Validate clean install in a fresh venv: `pip install dist/*.whl`

## 3) Tag

- [ ] Commit changes
- [ ] Create tag: `git tag vX.Y.Z`
- [ ] Push: `git push --tags`

## 4) Publish (CI)

- [ ] GitHub Actions `release.yml` builds wheels and publishes to TestPyPI
- [ ] Verify TestPyPI install works
- [ ] GitHub Actions `release.yml` publishes to PyPI
- [ ] Verify release on PyPI
- [ ] Announce release notes
