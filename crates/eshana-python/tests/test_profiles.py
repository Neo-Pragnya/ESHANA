"""The profile library: revisions, lossless export/import, and replay that reproduces the birth UTC, including
historic offsets (local mean time, war time, double summer time)."""
import json

import pytest

from eshana.profiles import ProfileError, ProfileLibrary

CASES = [  # name, fields, expected UTC
    ("delhi", dict(date="1990-05-15", time="14:30", timezone="Asia/Kolkata", latitude=28.61, longitude=77.21), "1990-05-15T09:00:00Z"),
    # India kept war time (UTC+6:30) in 1942–45
    ("war-time", dict(date="1943-06-01", time="12:00", timezone="Asia/Kolkata", latitude=19.07, longitude=72.88), "1943-06-01T05:30:00Z"),
    # British double summer time (UTC+2) in June 1941
    ("double-summer", dict(date="1941-06-21", time="12:00", timezone="Europe/London", latitude=51.5, longitude=-0.12), "1941-06-21T10:00:00Z"),
    # before standard time: a fixed offset as printed on the record
    ("madurai-1879", dict(date="1879-12-30", time="01:00", utc_offset_minutes=321, latitude=9.9333, longitude=78.1167), "1879-12-29T19:39:00Z"),
]


@pytest.mark.parametrize("name,fields,want", CASES, ids=[c[0] for c in CASES])
def test_replay_reproduces_the_birth_utc(tmp_path, name, fields, want):
    lib = ProfileLibrary(tmp_path / "p.json")
    rev = lib.save(name, saved_at="2026-11-23", **fields)
    assert rev["resolved"]["birth_utc"].startswith(want[:19]), rev["resolved"]
    r = lib.replay(name)
    assert r["same_utc"] and r["same_offset"], r


def test_revisions_append_and_carry_fields(tmp_path):
    lib = ProfileLibrary(tmp_path / "p.json")
    lib.save("a", **CASES[0][1])
    lib.save("a", time="14:45")
    h = lib.history("a")
    assert [r["revision"] for r in h] == [1, 2]
    assert h[1]["fields"]["timezone"] == "Asia/Kolkata" and h[1]["fields"]["time"] == "14:45"
    assert h[0]["resolved"]["birth_utc"] != h[1]["resolved"]["birth_utc"]
    assert lib.get("a", 1)["fields"]["time"] == "14:30"
    with pytest.raises(ProfileError):
        lib.save("a", colour="blue")


def test_export_import_round_trip(tmp_path):
    lib = ProfileLibrary(tmp_path / "p.json")
    for name, fields, _ in CASES:
        lib.save(name, **fields)
    out = lib.export(tmp_path / "export.json")
    fresh = ProfileLibrary(tmp_path / "q.json")
    assert sorted(fresh.import_file(out)) == sorted(c[0] for c in CASES)
    assert fresh.data == lib.data
    # importing again keeps both, never overwrites
    again = fresh.import_file(out)
    assert "delhi (2)" in again and fresh.get("delhi") == lib.get("delhi")
    # reopening the file gives the same library
    assert ProfileLibrary(tmp_path / "q.json").data == fresh.data
    with pytest.raises(ProfileError):
        (tmp_path / "bad.json").write_text(json.dumps({"schema": "other"}))
        fresh.import_file(tmp_path / "bad.json")


def test_context_uses_the_stored_instant(tmp_path):
    lib = ProfileLibrary(tmp_path / "p.json")
    lib.save("delhi", **CASES[0][1])
    ctx = lib.context("delhi")
    assert ctx.when == {"utc": lib.get("delhi")["resolved"]["birth_utc"]}
