"""ESHANA: a thin Python facade over the Rust engine.

Every call builds a JSON request and sends it through ``_native.execute``. No rule is implemented here, which is
why the Python results are byte-identical to the CLI's.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Any, Dict, Iterable, List, Optional

from . import _native

__all__ = [
    "ChartEngine",
    "Chart",
    "Context",
    "EshanaError",
    "execute",
    "engine_version",
    "__version__",
]

__version__ = _native.engine_version()
EXPANSION = "Extensible Spatiotemporal Hypersystem for Astroinformatics, Natural Cycles & Associations"


class EshanaError(Exception):
    """An error envelope from the engine. ``code`` is a stable ``ESH-*`` identifier."""

    def __init__(self, code: str, message: str):
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message


def engine_version() -> str:
    """Version of the Rust engine inside this wheel."""
    return _native.engine_version()


def execute(request: Dict[str, Any]) -> Dict[str, Any]:
    """Send a raw request dict and return the envelope, raising :class:`EshanaError` on an error envelope."""
    out = json.loads(_native.execute(json.dumps(request)))
    if out.get("kind") == "error":
        raise EshanaError(out["error"]["code"], out["error"]["message"])
    return out


@dataclass(frozen=True)
class Context:
    """A moment and (optionally) a place. Local times need a zone or an explicit offset: ESHANA assumes neither."""

    when: Dict[str, Any]
    place: Optional[Dict[str, float]] = None

    @classmethod
    def from_local_birth(
        cls,
        date: str,
        time: Optional[str] = None,
        timezone: Optional[str] = None,
        latitude: Optional[float] = None,
        longitude: Optional[float] = None,
        utc_offset_minutes: Optional[int] = None,
        fold: Optional[str] = None,
        uncertainty_minutes: Optional[float] = None,
    ) -> "Context":
        """Local civil date and time in an IANA zone. Omit ``time`` for an unknown birth time."""
        when = {
            "date": date,
            "time": time,
            "timezone": timezone,
            "utc_offset_minutes": utc_offset_minutes,
            "fold": fold,
            "uncertainty_minutes": uncertainty_minutes,
        }
        return cls({k: v for k, v in when.items() if v is not None}, _place(latitude, longitude))

    @classmethod
    def from_utc(cls, instant_utc: str, latitude: Optional[float] = None, longitude: Optional[float] = None) -> "Context":
        """A UTC instant such as ``2000-01-01T12:00:00Z``."""
        return cls({"utc": instant_utc}, _place(latitude, longitude))


def _place(lat: Optional[float], lon: Optional[float]) -> Optional[Dict[str, float]]:
    if lat is None and lon is None:
        return None
    if lat is None or lon is None:
        raise ValueError("give both latitude and longitude, or neither")
    return {"latitude": lat, "longitude": lon}


@dataclass
class Chart:
    """A result envelope with a few lookups. ``raw`` is the complete envelope."""

    raw: Dict[str, Any]

    @property
    def provenance(self) -> Dict[str, Any]:
        return self.raw["provenance"]

    @property
    def positions(self) -> List[Dict[str, Any]]:
        return self.raw["positions"]

    @property
    def derived(self) -> List[Dict[str, Any]]:
        return self.raw["derived_attributes"]

    @property
    def houses(self) -> Optional[Dict[str, Any]]:
        return self.raw.get("houses")

    @property
    def warnings(self) -> List[Dict[str, Any]]:
        return self.raw.get("warnings", [])

    def position(self, body: str) -> Dict[str, Any]:
        """The position of one body, e.g. ``"moon"``."""
        for p in self.positions:
            if p["id"] == body:
                return p
        raise KeyError(body)

    def attribute(self, attribute_id: str) -> Dict[str, Any]:
        """The *value* of one derived attribute, e.g. ``"vedic.moon.nakshatra"``."""
        return self.attribute_full(attribute_id)["value"]

    def attribute_full(self, attribute_id: str) -> Dict[str, Any]:
        """The derived attribute with its rule version and epistemic label."""
        for a in self.derived:
            if a["id"] == attribute_id:
                return a
        raise KeyError(attribute_id)

    def to_json(self, **kw: Any) -> str:
        return json.dumps(self.raw, **kw)

    def to_html(self, path: Optional[str] = None, **kw: Any) -> str:
        """Render this envelope as a self-contained HTML report; also write it to ``path`` when given.

        Keyword arguments go to :func:`eshana.report.render_html` (``title``, ``subject``, ``sections``, ``theme``).
        """
        from .report import render_html

        label = (self.raw.get("calculation") or {}).get("system") or self.raw.get("kind", "chart")
        page = render_html({label: self.raw}, **kw)
        if path:
            with open(path, "w", encoding="utf-8") as fh:
                fh.write(page)
        return page

    def _repr_html_(self) -> str:  # Jupyter renders the report inline
        return self.to_html(sections=["summary", "chakra", "vedic", "divisional", "dasha", "western"])


class ChartEngine:
    """Facade over the engine.

    ``provider`` is ``"analytic"`` (default, no data files) or ``"spk"`` with ``spk_path`` pointing to a JPL kernel.
    """

    def __init__(self, provider: str = "analytic", spk_path: Optional[str] = None):
        self.provider = provider
        self.spk_path = spk_path

    def _cfg(self, **cfg: Any) -> Dict[str, Any]:
        out = {"provider": self.provider}
        out.update({k: v for k, v in cfg.items() if v is not None})
        return out

    def _base(self, ctx: Context, system: Optional[str], cfg: Dict[str, Any], **extra: Any) -> Dict[str, Any]:
        req: Dict[str, Any] = {"when": ctx.when, "config": cfg}
        if ctx.place:
            req["place"] = ctx.place
        if system:
            req["system"] = system
        if self.spk_path:
            req["spk_path"] = self.spk_path
        req.update({k: v for k, v in extra.items() if v is not None})
        return req

    def positions(
        self,
        ctx: Context,
        zodiac: str = "tropical",
        ayanamsha: Optional[str] = None,
        node_model: str = "mean",
        bodies: Optional[Iterable[str]] = None,
    ) -> Chart:
        """Apparent geocentric positions only."""
        cfg = self._cfg(zodiac=zodiac, ayanamsha=ayanamsha, node_model=node_model)
        req = self._base(ctx, None, cfg, bodies=list(bodies) if bodies else None)
        return Chart(execute({"op": "positions", **req}))

    def vedic_chart(
        self,
        ctx: Context,
        ayanamsha: str,
        houses: str = "whole_sign",
        node_model: str = "mean",
        divisions: Iterable[int] = (1,),
        dasha: bool = False,
        year_convention: Optional[str] = None,
    ) -> Chart:
        """Sidereal chart. ``ayanamsha`` is required: ESHANA never picks one silently."""
        cfg = self._cfg(zodiac="sidereal", ayanamsha=ayanamsha, house_system=houses, node_model=node_model)
        req = self._base(ctx, "vedic", cfg, divisions=list(divisions), dasha=dasha, year_convention=year_convention)
        return Chart(execute({"op": "chart", **req}))

    def western_chart(
        self,
        ctx: Context,
        zodiac: str = "tropical",
        houses: str = "whole_sign",
        node_model: str = "mean",
        aspects: bool = True,
        orb_scale: Optional[float] = None,
        ayanamsha: Optional[str] = None,
    ) -> Chart:
        """Western chart; tropical by default."""
        cfg = self._cfg(zodiac=zodiac, ayanamsha=ayanamsha, house_system=houses, node_model=node_model)
        req = self._base(ctx, "western", cfg, aspects=aspects, orb_scale=orb_scale)
        return Chart(execute({"op": "chart", **req}))

    def human_design(self, ctx: Context, node_model: Optional[str] = None) -> Dict[str, Any]:
        """Design instant, gate/line activations and the derived BodyGraph attributes."""
        cfg = self._cfg(node_model=node_model)
        return execute({"op": "human_design", **self._base(ctx, None, cfg)})

    def numerology(
        self,
        name: Optional[str] = None,
        date_of_birth: Optional[str] = None,
        school: str = "pythagorean",
        keep_master_numbers: bool = True,
        y_policy: str = "consonant",
        life_path_method: str = "reduce_parts",
        reference_year: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Name and date numerology under one named school."""
        req = {
            "op": "numerology",
            "name": name,
            "date_of_birth": date_of_birth,
            "school": school,
            "keep_master_numbers": keep_master_numbers,
            "y_policy": y_policy,
            "life_path_method": life_path_method,
            "reference_year": reference_year,
        }
        return execute({k: v for k, v in req.items() if v is not None})

    def compare(self, ctx: Context, system: str, a: Dict[str, Any], b: Dict[str, Any], **base_cfg: Any) -> Dict[str, Any]:
        """Same input under two configurations; returns what changed."""
        base = self._base(ctx, system, self._cfg(**base_cfg))
        return execute({"op": "compare", "base": base, "a": a, "b": b})

    def graph(self, ctx: Context, system: str, fmt: str = "json", **cfg: Any) -> Any:
        """Symbolic graph of a chart. ``json`` returns a dict; ``mermaid`` and ``graphml`` return text."""
        chart = self._base(ctx, system, self._cfg(**cfg), assume_defaults=True)
        out = execute({"op": "graph", "chart": chart, "format": fmt})
        return out if fmt == "json" else out["content"]

    @staticmethod
    def info() -> Dict[str, Any]:
        """Engine version, tz database, providers, rule packs and the error catalogue."""
        return execute({"op": "info"})
