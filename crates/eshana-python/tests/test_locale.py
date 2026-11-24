"""Interface languages change labels only: the embedded results are identical in every locale."""
import json
import re

import pytest

from eshana import Context
from eshana import report as R

CTX = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.6139, 77.2090)


def data_of(page):
    raw = re.search(r'<script type="application/json" id="eshana-data">(.*?)</script>', page, re.S).group(1)
    return json.loads(raw.replace("<\\/", "</"))


def test_numerical_output_is_identical_across_locales():
    kw = dict(reference_utc="2026-11-21T00:00:00Z", transit_days=30, progression_years=5)
    en, hi = data_of(R.birth_report(CTX, **kw)), data_of(R.birth_report(CTX, locale="hi", **kw))
    assert en["locale"]["lang"] == "en" and hi["locale"]["lang"] == "hi"
    for d in (en, hi):
        d.pop("locale")
    assert en == hi


def test_catalogues_cover_every_page_and_mark_their_review_status():
    pages = {p for p in R.SECTIONS if p not in ("planet", "house", "point", "yoga")}
    for code in R.LOCALES:
        cat = R._locale(code)
        assert set(cat["pages"]) >= pages, (code, sorted(pages - set(cat["pages"])))
        assert cat["status"] in ("reference", "reviewed", "draft")
        assert set(cat["ui"]) >= {"search", "settings", "theme", "print", "json", "copy", "not_computed"}
    assert R._locale("hi")["status"] == "reviewed" and R._locale("hi")["reviewed_by"]
    with pytest.raises(ValueError):
        R._locale("xx")


def test_every_file_carries_every_catalogue_and_defaults_to_english():
    """The language is a reader setting: each file embeds all catalogues; the file's default is English unless chosen."""
    kw = dict(reference_utc="2026-11-21T00:00:00Z", transit_days=0, progression_years=0, sections=["overview"])
    d = data_of(R.birth_report(CTX, **kw))
    assert set(d["locales"]) == set(R.LOCALES)
    assert d["locale"]["lang"] == "en"
    assert data_of(R.birth_report(CTX, locale="hi", **kw))["locale"]["lang"] == "hi"
