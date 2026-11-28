"""The `eshana` command of the Python package: the same command line as the Rust `eshana` binary.

`pip install eshana` puts `eshana` on the PATH (and `python -m eshana` works without it). Both call the CLI compiled
into the extension module, so every subcommand, option, output and exit status is the binary's own:

    eshana --version
    eshana chart vedic --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090
"""
from __future__ import annotations

import sys
from typing import List, Optional


def main(argv: Optional[List[str]] = None) -> int:
    """Run `eshana` with ``argv`` (default: the process arguments) and return its exit status."""
    from . import _native

    args = list(sys.argv[1:] if argv is None else argv)
    # the CLI writes to the process's stdout and stderr directly; flush Python's buffers first so output keeps its order
    sys.stdout.flush()
    sys.stderr.flush()
    return int(_native.cli(["eshana", *args]))


if __name__ == "__main__":
    raise SystemExit(main())
