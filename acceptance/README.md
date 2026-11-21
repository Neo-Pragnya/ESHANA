# Capability acceptance suite

Behaviour-level checks for everything ESHANA promises, run against the real `eshana` binary and the Python package before every
release. Unit tests prove the parts; this suite proves the product.

```bash
make acceptance            # dev build
make acceptance-wheel      # the built wheel in a fresh venv (what ships)
```

- **Scenarios** (`features/*.feature`, one Feature per file) run in sandboxes: a throw-away git project, its own `HOME`, no
  colour. Commands go through `bash -c` with the target's `bin` first on `PATH`.
- **Capabilities** (`capabilities.yaml`) group scenarios via `@cap_<id>`; `blocker: true` ones veto a release.
- **The gate** writes `report/summary.md` and `report/acceptance.json`: PROMOTE (all pass), CONDITIONAL (known issues, unverified,
  non-blocker gaps, low CLI coverage), BLOCK (a blocker failed or has no scenario).
- `@known_issue` states the *correct* behaviour of something currently broken (strict xfail: fails loudly once fixed).
- `@adversarial` probes sit next to a legitimate twin that must still work (polar Placidus is refused; whole sign works).
- Step quoting: write quotes inside a step literally, no backslashes (`the output contains "kind": "error"`). For anything with
  nested quoting, put the script in a file with `Given the file "x.py" with content:`.

`harness.py` is a byte-identical copy from Ananke Plexus / Dev Guru; repository-specific steps live in `conftest.py`.
