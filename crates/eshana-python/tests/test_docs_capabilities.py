"""The reference docs describe what this wheel does: operations, error codes, CLI commands and methods."""
import importlib.util
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[3] / "scripts" / "check_capabilities.py"


def test_docs_match_the_installed_wheel():
    spec = importlib.util.spec_from_file_location("check_capabilities", SCRIPT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    assert mod.check() == []
