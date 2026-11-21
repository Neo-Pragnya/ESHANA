---
title: "Publishing a release"
description: "How ESHANA is released to PyPI: the gates that must pass, the wheels that are built, where the token comes from and how to check the result."
---

# Publishing a release

ESHANA is published as open-source libraries ([ADR-011](../adr/011-open-source-distribution.md)). The Python
distribution is [`eshana` on PyPI](https://pypi.org/project/eshana/). Rust crates follow on crates.io (`BIND-07`).

## 1. Gates (all must pass)

```bash
make verify            # fmt, clippy -D warnings, Rust tests, Python tests, tracker freshness
make accuracy-suite    # birth-chart accuracy, ~970 checks (add ESHANA_SPK for the spk provider)
make acceptance-wheel  # Gherkin capability suite against the built wheel in a fresh venv: must say PROMOTE
make deny audit        # licences and RustSec advisories
make scan              # dist scan of the wheel and sdist: secrets, licences, malware heuristics
make docs-check        # docs front matter, links, ADR index
```

A failing security check is a bug to fix, not a warning to dismiss.

## 2. Build

```bash
make dist              # sdist + abi3 wheel for this machine, into dist/
make dist-linux        # manylinux wheels for x86_64 and aarch64 (cross-built with zig)
```

The wheels are `abi3`: one wheel per platform serves CPython 3.9 and later. Any other platform installs from the
sdist, which needs a Rust toolchain (`rustup`).

## 3. Publish

The token is read from the environment. Keep it in a git-ignored `.env` at the repository root:

```bash
# .env  (git-ignored; never commit it)
UV_PUBLISH_TOKEN=pypi-...
UV_PUBLISH_TOKEN_TESTPYPI=pypi-...
```

```bash
make publish-test      # TestPyPI first
make publish           # PyPI
```

Both targets load only the `UV_PUBLISH_TOKEN*` lines from `.env` and pass them to `uv publish`. The token is never
printed. A version can be uploaded only once: bump `version` in `crates/eshana-python/pyproject.toml` and the
workspace `Cargo.toml` for every release, and move the `CHANGELOG.md` entry from *Unreleased*.

## 4. Check what shipped

```bash
uv run --no-project --with eshana==<version> -- python -c "import eshana; print(eshana.__version__)"
uv run --no-project --with eshana==<version> -- eshana-report --date 2000-01-01 --time 12:00 --tz UTC -o /tmp/x.html
```

Then record the release in `planning/backlog.json` (`BIND-06` → `released`) and regenerate `TRACKING.md`.

## 5. In CI (next)

`.github/workflows/release.yml` builds wheels for Linux, macOS and Windows with `maturin-action` on a version tag and
publishes with PyPI **trusted publishing**, so no token lives in GitHub. Enable it once on PyPI (project → Publishing
→ add GitHub publisher: repository `Neo-Pragnya/ESHANA`, workflow `release.yml`, environment `pypi`).
