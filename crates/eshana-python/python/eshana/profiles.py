"""A local, revisioned library of birth profiles.

A profile keeps what the person entered (local date and time, zone or fixed offset, fold policy, place) *and* the
birth instant the engine resolved from it, so the chart can be replayed later and checked: if a newer time-zone
database or engine resolves the same civil input to a different UTC (a corrected historic offset, for example), replay
says so instead of silently moving the chart.

Every change appends a revision; nothing is overwritten. The library is one JSON file (schema
``eshana.profiles@1``) that exports and imports losslessly. Nothing leaves the machine.

    lib = ProfileLibrary("~/eshana-profiles.json")
    lib.save("asha", date="1990-05-15", time="14:30", timezone="Asia/Kolkata", latitude=28.61, longitude=77.21)
    lib.replay("asha")   # {"same_utc": True, ...}
"""
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
from typing import Any, Dict, List, Optional

from . import execute

SCHEMA = "eshana.profiles@1"
_FIELDS = ("date", "time", "timezone", "utc_offset_minutes", "fold", "latitude", "longitude", "place_label", "notes")


class ProfileError(ValueError):
    """A profile operation that cannot be done (unknown name, bad file)."""


def civil_to_utc(fields: Dict[str, Any]) -> Dict[str, Any]:
    """The engine's resolution of a profile's civil input: UTC instant, zone, offset (seconds east)."""
    when = {k: fields[k] for k in ("date", "time", "timezone", "utc_offset_minutes", "fold") if fields.get(k) is not None}
    req: Dict[str, Any] = {"op": "positions", "when": when, "bodies": ["sun"], "config": {"zodiac": "tropical"}}
    if fields.get("latitude") is not None and fields.get("longitude") is not None:
        req["place"] = {"latitude": fields["latitude"], "longitude": fields["longitude"]}
    env = execute(req)
    i = env["input"]
    return {"birth_utc": i["instant_utc"], "civil_zone": i.get("civil_zone"), "civil_offset_s": i.get("civil_offset_s"),
            "tzdb_version": env.get("provenance", {}).get("tzdb_version")}


class ProfileLibrary:
    """Profiles in one JSON file; each profile is a list of revisions, newest last."""

    def __init__(self, path: os.PathLike | str):
        self.path = Path(path).expanduser()
        self.data: Dict[str, Any] = {"schema": SCHEMA, "profiles": {}}
        if self.path.exists():
            self.data = self._checked(json.loads(self.path.read_text(encoding="utf-8")))

    @staticmethod
    def _checked(d: Dict[str, Any]) -> Dict[str, Any]:
        if d.get("schema") != SCHEMA or not isinstance(d.get("profiles"), dict):
            raise ProfileError(f"not an {SCHEMA} file")
        return d

    def _write(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        tmp = self.path.with_suffix(self.path.suffix + ".tmp")
        tmp.write_text(json.dumps(self.data, ensure_ascii=False, indent=1, sort_keys=True) + "\n", encoding="utf-8")
        tmp.replace(self.path)

    def names(self) -> List[str]:
        return sorted(self.data["profiles"])

    def save(self, name: str, *, saved_at: Optional[str] = None, **fields: Any) -> Dict[str, Any]:
        """Add a profile or a new revision of it. Unchanged fields carry over from the last revision."""
        unknown = set(fields) - set(_FIELDS)
        if unknown:
            raise ProfileError(f"unknown fields: {', '.join(sorted(unknown))}")
        revs = self.data["profiles"].setdefault(name, [])
        base = copy.deepcopy(revs[-1]["fields"]) if revs else {}
        base.update({k: v for k, v in fields.items()})
        rev = {"revision": len(revs) + 1, "saved_at": saved_at, "fields": base, "resolved": civil_to_utc(base)}
        revs.append(rev)
        self._write()
        return copy.deepcopy(rev)

    def get(self, name: str, revision: Optional[int] = None) -> Dict[str, Any]:
        revs = self.data["profiles"].get(name)
        if not revs:
            raise ProfileError(f"no profile '{name}'")
        rev = revs[-1] if revision is None else next((r for r in revs if r["revision"] == revision), None)
        if rev is None:
            raise ProfileError(f"'{name}' has no revision {revision}")
        return copy.deepcopy(rev)

    def history(self, name: str) -> List[Dict[str, Any]]:
        return copy.deepcopy(self.data["profiles"].get(name, []))

    def replay(self, name: str, revision: Optional[int] = None) -> Dict[str, Any]:
        """Resolve the stored civil input again and compare with the stored instant, zone and offset."""
        rev = self.get(name, revision)
        now = civil_to_utc(rev["fields"])
        stored = rev["resolved"]
        return {"name": name, "revision": rev["revision"], "stored": stored, "now": now,
                "same_utc": stored["birth_utc"] == now["birth_utc"],
                "same_offset": stored["civil_offset_s"] == now["civil_offset_s"]}

    def context(self, name: str, revision: Optional[int] = None) -> Any:
        """An :class:`eshana.Context` for the stored UTC instant and place (what the charts use)."""
        from . import Context
        rev = self.get(name, revision)
        f = rev["fields"]
        return Context.from_utc(rev["resolved"]["birth_utc"], f.get("latitude"), f.get("longitude"))

    def export(self, path: os.PathLike | str) -> Path:
        out = Path(path).expanduser()
        out.write_text(json.dumps(self.data, ensure_ascii=False, indent=1, sort_keys=True) + "\n", encoding="utf-8")
        return out

    def import_file(self, path: os.PathLike | str, *, on_conflict: str = "keep_both") -> List[str]:
        """Merge another library file. A name already here is kept and the incoming one is stored as ``name (2)``
        (``keep_both``), or the incoming revisions are appended after the existing ones (``append``)."""
        other = self._checked(json.loads(Path(path).expanduser().read_text(encoding="utf-8")))
        added = []
        for name, revs in other["profiles"].items():
            if name not in self.data["profiles"]:
                self.data["profiles"][name] = copy.deepcopy(revs)
                added.append(name)
            elif on_conflict == "append":
                cur = self.data["profiles"][name]
                for r in revs:
                    r2 = copy.deepcopy(r)
                    r2["revision"] = len(cur) + 1
                    cur.append(r2)
                added.append(name)
            else:
                k = 2
                while f"{name} ({k})" in self.data["profiles"]:
                    k += 1
                self.data["profiles"][f"{name} ({k})"] = copy.deepcopy(revs)
                added.append(f"{name} ({k})")
        self._write()
        return added
