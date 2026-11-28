"""The package's `eshana` command is the full Rust CLI: the console script and `python -m eshana` report the
version, exit 0/2 like the binary, and print exactly what the binary prints."""
import os
import subprocess
import sys
from pathlib import Path

import pytest

import eshana

SCRIPT = Path(sys.executable).parent / ("eshana.exe" if os.name == "nt" else "eshana")


def run(*args, via="module"):
    cmd = [sys.executable, "-m", "eshana", *args] if via == "module" else [str(SCRIPT), *args]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=120)


@pytest.mark.parametrize("via", ["module", "script"])
def test_version_and_exit_codes(via):
    if via == "script" and not SCRIPT.exists():
        pytest.skip("console script not installed in this environment")
    v = run("--version", via=via)
    assert v.returncode == 0 and v.stdout.strip() == f"eshana {eshana.__version__}"
    assert run("--help", via=via).returncode == 0
    bad = run("chart", "vedic", "--date", "1990-05-15", "--time", "14:30", via=via)
    assert bad.returncode == 2 and "ESH-INPUT" in bad.stderr
    assert run("no-such-command", via=via).returncode == 2


def test_in_process_main_returns_the_status(capfd):
    from eshana.cli import main

    assert main(["--version"]) == 0
    assert capfd.readouterr().out.strip() == f"eshana {eshana.__version__}"


def test_output_matches_the_rust_binary():
    binary = os.environ.get("ESHANA_BIN")
    if not binary or not Path(binary).exists():
        pytest.skip("set ESHANA_BIN to the Rust binary to compare")
    args = ["positions", "--utc", "2026-01-01T00:00:00Z", "--zodiac", "sidereal", "--ayanamsha", "lahiri"]
    ours = run(*args)
    theirs = subprocess.run([binary, *args], capture_output=True, text=True, timeout=120)
    assert ours.returncode == theirs.returncode == 0 and ours.stdout == theirs.stdout
