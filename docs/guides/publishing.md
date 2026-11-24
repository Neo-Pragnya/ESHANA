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

### Attestations, SBOM and the GitHub release

Once the files are on PyPI, run the **attest** workflow (`gh workflow run attest.yml -f version=<version>`, or
Actions → attest → *Run workflow*; the workflow must already be on `main`). It downloads exactly the files PyPI
serves, checks them against PyPI's SHA-256, writes a CycloneDX SBOM ([scripts/sbom.py](../../scripts/sbom.py): every
Rust crate compiled into the wheel, with licences) and `SHA256SUMS`, signs build-provenance and SBOM attestations with
GitHub's Sigstore-backed service, and creates the GitHub release `v<version>` with the changelog section and all of
those files attached. Anyone can then check a download:

```bash
gh attestation verify eshana-<version>-<tags>.whl --repo Neo-Pragnya/ESHANA
```

The wheels are built on the maintainer's machine, so the provenance says which workflow and commit observed those
digests, not that GitHub built them; tag-built wheels (section 6) would make it build provenance in full.
`make sbom` writes the same SBOM for a local `dist/` (into `target/sbom/`, so `make publish` does not upload it).

## 5. Update the website

The docs site (GitHub Pages) is built by `.github/workflows/docs-site.yml` from `main`. A release is not finished until
it is on `main` and the site has redeployed:

1. Merge the release branch into `main` (a pull request). The push to `main` starts the docs-site workflow.
2. Check the run: Actions → docs-site, or the *Docs site* badge in the README. It fails on demo drift
   (`build_demo.py --check`), docs checks and `astro check`, so run `make demo docs-check` locally first.
3. Open <https://neo-pragnya.github.io/ESHANA/> and the README images under `/readme/`.

To redeploy without a code change: Actions → docs-site → *Run workflow*.

## 6. In CI (next)

`.github/workflows/release.yml` builds wheels for Linux, macOS and Windows with `maturin-action` on a version tag and
publishes with PyPI **trusted publishing**, so no token lives in GitHub. Enable it once on PyPI (project → Publishing
→ add GitHub publisher: repository `Neo-Pragnya/ESHANA`, workflow `release.yml`, environment `pypi`).
