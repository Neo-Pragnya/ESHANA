"""The cookbook notebooks (DOC-04) run top to bottom: every code cell, in order, in one namespace."""
import json
from pathlib import Path

import pytest

BOOKS = sorted((Path(__file__).resolve().parents[3] / "examples" / "notebooks").glob("*.ipynb"))


def test_there_are_notebooks():
    assert len(BOOKS) >= 4


@pytest.mark.parametrize("book", BOOKS, ids=lambda p: p.name)
def test_notebook_runs(book, capsys):
    nb = json.loads(book.read_text(encoding="utf-8"))
    ns: dict = {"__name__": "__notebook__"}
    for i, cell in enumerate(c for c in nb["cells"] if c["cell_type"] == "code"):
        exec(compile("".join(cell["source"]), f"{book.name}[{i}]", "exec"), ns)  # noqa: S102 - our own notebooks
    assert capsys.readouterr().out.strip(), "a notebook should print something"
