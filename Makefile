.PHONY: gallery help install fmt fmt-check lint test py-test verify build build-release wheel acceptance acceptance-wheel \
        scan deny audit golden accuracy demo docs-check docs-site tracking tracking-check clean \
        accuracy-suite accuracy-report reference diagrams reports dist dist-linux publish publish-test

VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' crates/eshana-python/pyproject.toml | head -1)

help:
	@echo "make install | verify | accuracy-suite | acceptance | acceptance-wheel | reports | diagrams | dist | publish | docs-site | tracking | scan"

# ── toolchain ────────────────────────────────────────────────────────────────────────────────────────────────
install:
	command -v cargo >/dev/null || { echo "Install Rust: https://rustup.rs"; exit 1; }
	uv venv -q .venv
	uv pip install -q --python .venv/bin/python maturin ziglang pytest pytest-bdd pytest-xdist pyyaml jsonschema

fmt:
	cargo fmt --all
fmt-check:
	cargo fmt --all -- --check
lint:
	cargo clippy --workspace --exclude eshana-python --all-targets -- -D warnings
test:
	cargo test --workspace --exclude eshana-python
# the engine for the browser (ADR-013), shipped in the wheel for eshana-report --live
wasm:
	cargo build -q -p eshana-wasm --release --target wasm32-unknown-unknown
	mkdir -p crates/eshana-python/python/eshana/data
	cp target/wasm32-unknown-unknown/release/eshana_wasm.wasm crates/eshana-python/python/eshana/data/eshana_engine.wasm

py-test: build-release wasm
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin develop -q --uv
	ESHANA_BIN=$(CURDIR)/target/release/eshana .venv/bin/python -m pytest crates/eshana-python/tests -q

# fmt + clippy (deny warnings) + every Rust test + Python tests + tracker freshness
verify: fmt-check lint test py-test tracking-check accuracy-suite

build:
	cargo build --workspace --exclude eshana-python
build-release:
	cargo build --release -p eshana-cli

# ── wheel (Python) ───────────────────────────────────────────────────────────────────────────────────────────
wheel: wasm
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin build --release --out ../../dist

# ── accuracy and golden data ─────────────────────────────────────────────────────────────────────────────────
# Measured error of each provider against JPL DE440s. The spk run needs a kernel: ESHANA_SPK=/path/de440s.bsp
accuracy:
	cargo run -q -p eshana-ephemeris --example accuracy -- analytic
golden:
	ESHANA_SPK=$(ESHANA_SPK) cargo test -p eshana-ephemeris --test golden

# ── birth-chart accuracy suite (docs/guides/accuracy-suite.md) ───────────────────────────────────────────────
# Add ESHANA_SPK=/path/de440s.bsp to include the spk provider.
accuracy-suite:
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin develop -q --uv --release
	.venv/bin/python -m pytest accuracy -q -n auto -p no:cacheprovider
accuracy-report:
	.venv/bin/python scripts/accuracy_report.py --check
# Rebuild the Swiss Ephemeris reference set (offline oracle; numbers only, nothing ships)
reference:
	uv run --no-project --with pyswisseph python scripts/build_reference.py

# ── diagrams + example reports ────────────────────────────────────────────────────────────────────────────────
diagrams:
	ARCHLIB_ICONS=diagrams/icons python3 diagrams/build_arch.py diagrams/arch_specs.py --out docs/architecture --no-stages \
	  --title "ESHANA architecture" --intro "Concept diagrams generated from diagrams/arch_specs.py. Press play to walk each numbered flow."
reports: build-release
	mkdir -p dist
	.venv/bin/eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 \
	  --name "Example Person" --place "New Delhi, India" --reference 2026-11-21T00:00:00Z \
	  -o examples/reports/example-birth-chart.html
	.venv/bin/eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 \
	  --name "Example Person" --place "New Delhi, India" --reference 2026-11-21T00:00:00Z \
	  --partner 1992-08-20,09:15,Asia/Kolkata,19.076,72.8777 --title "Compatibility example" --icons glyph \
	  -o examples/reports/example-compatibility.html
	.venv/bin/eshana-report --date 1990-05-15 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 --name "Example Person" \
	  --place "New Delhi, India (time unknown)" --title "Unknown birth time" --reference 2026-11-21T00:00:00Z \
	  -o examples/reports/example-unknown-time.html
	.venv/bin/eshana-report --date 1985-11-03 --time 06:45 --tz Europe/London --lat 51.5074 --lon=-0.1278 \
	  --systems western,numerology --name "Sample Western" --place "London, UK" --theme light \
	  -o examples/reports/example-western.html
	.venv/bin/eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 \
	  --name "Example Person" --place "New Delhi, India" --reference 2026-11-21T00:00:00Z \
	  --partner 1992-08-20,09:15,Asia/Kolkata,19.076,72.8777 --calendar-days 31 --kp-number 100 --live \
	  --title "The studio as an app" -o examples/reports/example-studio-app.html
	./target/release/eshana chart vedic --date 1996-12-07 --time 10:34 --tz Asia/Kolkata --lat 13.0878 --lon 80.2785 \
	  --divisions 9 --dasha > dist/vedic-chennai.json
	.venv/bin/eshana-report --from-json dist/vedic-chennai.json --title "From the CLI" --place "Chennai" \
	  -o examples/reports/example-from-cli.html
	.venv/bin/eshana-report --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 \
	  --node true --sunrise centre_unrefracted --name "Sri Ramana Maharishi" --gender male --place "Madurai, India" \
	  --title "Reference chart" --reference 2026-11-21T00:00:00Z -o examples/reports/reference-ramana.html

# Website chart gallery (CHK-22): screenshots of real reports, examples/gallery/*.webp + website/src/data/gallery.json
gallery: reports
	node scripts/gallery.mjs

# ── release (docs/guides/publishing.md) ───────────────────────────────────────────────────────────────────────
# Files are versioned, so publish uploads only this version's files from dist/.
dist: build-release wasm
	python3 scripts/readme_sections.py
	python3 scripts/pypi_readme.py
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin sdist --out ../../dist
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin build --release --out ../../dist
dist-linux: wasm
	rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin build --release --zig \
	  --target x86_64-unknown-linux-gnu --compatibility manylinux2014 --out ../../dist
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin build --release --zig \
	  --target aarch64-unknown-linux-gnu --compatibility manylinux2014 --out ../../dist
# Only the UV_PUBLISH_TOKEN* lines of .env are read; the token is never printed.
publish-test:
	@set -a; eval "$$(grep -E '^UV_PUBLISH_TOKEN_TESTPYPI=' .env)"; set +a; \
	  UV_PUBLISH_TOKEN="$$UV_PUBLISH_TOKEN_TESTPYPI" uv publish --publish-url https://test.pypi.org/legacy/ \
	  dist/eshana-$(VERSION)*
publish:
	@set -a; eval "$$(grep -E '^UV_PUBLISH_TOKEN=' .env)"; set +a; uv publish dist/eshana-$(VERSION)*

# ── acceptance (capability scenarios + release gate) ─────────────────────────────────────────────────────────
# See acceptance/README.md. The gate writes acceptance/report/{acceptance.json,summary.md}; BLOCK fails.
ACCEPTANCE_ARGS ?=
acceptance: build-release wasm
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin develop -q --uv
	ln -sf $(CURDIR)/target/release/eshana $(CURDIR)/.venv/bin/eshana
	ACCEPTANCE_BIN_DIR=$(CURDIR)/.venv/bin .venv/bin/python -m pytest acceptance -n auto -p no:cacheprovider $(ACCEPTANCE_ARGS)

# The same against the built wheel in a fresh venv (what ships)
ACCEPTANCE_VENV := .acceptance-venv
acceptance-wheel: build-release wheel
	uv venv -q --allow-existing --python 3.12 $(ACCEPTANCE_VENV)
	uv pip install -q --python $(ACCEPTANCE_VENV)/bin/python --force-reinstall "$$(ls -t dist/*.whl | head -1)" pytest pytest-bdd pytest-xdist pyyaml
	ln -sf $(CURDIR)/target/release/eshana $(ACCEPTANCE_VENV)/bin/eshana
	ACCEPTANCE_BIN_DIR=$(CURDIR)/$(ACCEPTANCE_VENV)/bin $(ACCEPTANCE_VENV)/bin/python -m pytest acceptance -n auto -p no:cacheprovider $(ACCEPTANCE_ARGS)

# Real-Chrome journeys (Node 22+, Chrome; CHROME=/path overrides). Studio: the generated chart app. Site: website/dist.
JOURNEY_SHOTS ?=
studio-journeys: build-release wasm
	node acceptance/studio/wasm_parity.mjs target/wasm32-unknown-unknown/release/eshana_wasm.wasm target/release/eshana
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin develop -q --uv --release
	mkdir -p target/journeys
	.venv/bin/eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 \
	  --name "Example Person" --place "New Delhi, India" --reference 2026-11-21T00:00:00Z \
	  --partner "1992-08-20,09:15,Asia/Kolkata,19.076,72.8777" --calendar-days 31 --kp-number 100 --live \
	  -o target/journeys/studio.html
	node acceptance/studio/journeys.mjs target/journeys/studio.html $(if $(JOURNEY_SHOTS),--shots $(JOURNEY_SHOTS)/studio)
	# the reference chart (born 1879, read today): daśās that have ended, a parivartana, a crowded chart
	.venv/bin/eshana-report --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 \
	  --node true --sunrise centre_unrefracted --name "Sri Ramana Maharishi" --gender male --place "Madurai, India" \
	  --reference 2026-11-21T00:00:00Z -o target/journeys/ramana.html
	node acceptance/studio/journeys.mjs target/journeys/ramana.html $(if $(JOURNEY_SHOTS),--shots $(JOURNEY_SHOTS)/ramana)
	# guided creation: the real `eshana-report new` command, filled in by keyboard in Chrome
	node acceptance/studio/new_chart.mjs .venv/bin/eshana-report $(if $(JOURNEY_SHOTS),--shots $(JOURNEY_SHOTS)/new)
site-journeys:
	node acceptance/site/journeys.mjs website/dist --base $${SITE_BASE:-/} $(if $(JOURNEY_SHOTS),--shots $(JOURNEY_SHOTS)/site)

# Local review preview (git-ignored preview/): studio files for a few charts, the website, an index page
preview: build-release
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin develop -q --uv --release
	python3 scripts/build_preview.py
preview-serve: preview
	cd preview && python3 -m http.server 8765 --bind 127.0.0.1

# ── supply chain ─────────────────────────────────────────────────────────────────────────────────────────────
deny:
	cargo deny check
audit:
	cargo audit
scan: wheel
	uv run --no-project python scripts/scan_dist.py --dist dist --out scan-report $(SCAN_ARGS)

# ── demo + docs site ─────────────────────────────────────────────────────────────────────────────────────────
demo: build-release
	python3 scripts/build_demo.py website/src/data
docs-check:
	python3 scripts/check_docs.py
	python3 scripts/readme_sections.py --check
	python3 scripts/pypi_readme.py --check
# reference docs vs the installed wheel (operations, error codes, CLI commands, methods); --write regenerates methods.md
capabilities-check: build-release
	cd crates/eshana-python && VIRTUAL_ENV=$(CURDIR)/.venv $(CURDIR)/.venv/bin/maturin develop -q --uv
	ESHANA_BIN=$(CURDIR)/target/release/eshana .venv/bin/python scripts/check_capabilities.py
docs-site: demo docs-check
	cd website && npm ci && npm run build

# ── tracking ─────────────────────────────────────────────────────────────────────────────────────────────────
tracking:
	python3 scripts/tracking_report.py
	python3 scripts/progress_report.py
tracking-check:
	python3 scripts/tracking_report.py --check
	python3 scripts/progress_report.py --check

clean:
	cargo clean
	@echo "also remove dist, scan-report, .acceptance-venv, acceptance/report, website/dist, website/.astro by hand if wanted"
