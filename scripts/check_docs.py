#!/usr/bin/env python3
"""Docs checks: every page has a title and description, every relative .md link resolves, headings are unique per page,
mermaid fences are balanced, and no placeholder text is left. Exits non-zero with a list of problems."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
LINK = re.compile(r"(?<!\!)\[[^\]]*\]\(([^)\s]+)\)")
PLACEHOLDER = re.compile(r"\b(TODO|FIXME|lorem ipsum|TBD)\b", re.I)


def front_matter(text: str) -> dict[str, str]:
    m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
    if not m:
        return {}
    out = {}
    for line in m.group(1).splitlines():
        if ":" in line and not line.startswith(" "):
            k, v = line.split(":", 1)
            out[k.strip()] = v.strip().strip('"')
    return out


def main() -> int:
    problems: list[str] = []
    pages = sorted(DOCS.rglob("*.md"))
    for p in pages:
        rel = p.relative_to(ROOT)
        text = p.read_text(encoding="utf-8")
        fm = front_matter(text)
        if not fm.get("title"):
            problems.append(f"{rel}: missing front-matter title")
        if not fm.get("description"):
            problems.append(f"{rel}: missing front-matter description")
        raw_fm = re.match(r"^---\n(.*?)\n---\n", text, re.S)
        for line in (raw_fm.group(1).splitlines() if raw_fm else []):
            m2 = re.match(r"^(title|description):\s+([^\"'].*)$", line)
            if m2 and (": " in m2.group(2) or " #" in m2.group(2)):
                problems.append(f"{rel}: unquoted front-matter {m2.group(1)} contains ': ' (invalid YAML); quote it")
        body = re.sub(r"```.*?```", "", text, flags=re.S)
        if PLACEHOLDER.search(body) and "design/master-plan" not in str(rel):
            problems.append(f"{rel}: placeholder text ({PLACEHOLDER.search(body).group(0)})")
        fences = re.findall(r"^```", text, re.M)
        if len(fences) % 2:
            problems.append(f"{rel}: unbalanced code fences")
        for target in LINK.findall(body):
            if re.match(r"^(https?:|mailto:|#)", target):
                continue
            path = target.split("#")[0]
            if not path:
                continue
            dest = (p.parent / path).resolve()
            if path.endswith(".md"):
                if not dest.exists():
                    problems.append(f"{rel}: broken link {target}")
            elif not dest.exists() and not (ROOT / path.lstrip("/")).exists():
                problems.append(f"{rel}: broken link {target}")
        heads = re.findall(r"^#{2,3} (.+)$", body, re.M)
        dupes = {h for h in heads if heads.count(h) > 1}
        if dupes and "design/master-plan" not in str(rel):
            problems.append(f"{rel}: duplicate headings {sorted(dupes)}")
    # every ADR is in the index; every error code is documented
    index = (DOCS / "adr" / "index.md").read_text(encoding="utf-8")
    for adr in sorted((DOCS / "adr").glob("0*.md")):
        if adr.name not in index:
            problems.append(f"docs/adr/index.md does not list {adr.name}")
    errors_doc = (DOCS / "reference" / "errors.md").read_text(encoding="utf-8")
    core = (ROOT / "crates" / "eshana-core" / "src" / "lib.rs").read_text(encoding="utf-8")
    for code in re.findall(r'"(ESH-[A-Z-]+)"', core.split("pub fn catalog")[1].split("]")[0]):
        if f"`{code}`" not in errors_doc:
            problems.append(f"docs/reference/errors.md does not document {code}")
    for p in ROOT.glob("*.md"):
        if "TODO" in p.read_text(encoding="utf-8") and p.name != "ESHANA_Project_Vision_and_Master_Plan.md":
            problems.append(f"{p.name}: contains TODO")
    if problems:
        print("docs problems:\n  " + "\n  ".join(problems), file=sys.stderr)
        return 1
    print(f"docs ok: {len(pages)} pages, links, front matter, ADR index and error codes checked")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
