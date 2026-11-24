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
        calendar: Optional[str] = None,
    ) -> "Context":
        """Local civil date and time in an IANA zone. Omit ``time`` for an unknown birth time. ``calendar`` is
        ``gregorian`` (proleptic, the default), ``julian`` or ``auto`` (Julian before 1582-10-15); a signed year such
        as ``-0043-03-15`` is 44 BCE."""
        when = {
            "date": date,
            "time": time,
            "timezone": timezone,
            "utc_offset_minutes": utc_offset_minutes,
            "fold": fold,
            "uncertainty_minutes": uncertainty_minutes,
            "calendar": calendar,
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
        return self.to_html(sections=["overview", "chakra", "charts", "planets", "dasha", "western"])


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
        features: Iterable[str] = (),
        dasha_levels: Optional[int] = None,
        dasha_systems: Iterable[str] = (),
        reference_utc: Optional[str] = None,
        sunrise: Optional[str] = None,
        special_lagna_sun: Optional[str] = None,
        varga_methods: Iterable[str] = (),
    ) -> Chart:
        """Sidereal chart. ``ayanamsha`` is required: ESHANA never picks one silently.

        ``features`` adds Vedic sections: ``panchanga``, ``strength``, ``relationships``, ``drishti``,
        ``ashtakavarga``, ``jaimini``, ``kp``, ``yogas``, ``sade_sati`` or ``all``. ``dasha_levels`` (1-5) nests
        Vimshottari; ``dasha_systems`` adds ``yogini``; ``reference_utc`` marks the periods current at that instant.
        """
        cfg = self._cfg(zodiac="sidereal", ayanamsha=ayanamsha, house_system=houses, node_model=node_model)
        req = self._base(
            ctx, "vedic", cfg, divisions=list(divisions), dasha=dasha, year_convention=year_convention,
            features=list(features) or None, dasha_levels=dasha_levels, dasha_systems=list(dasha_systems) or None,
            reference_utc=reference_utc, sunrise=sunrise, special_lagna_sun=special_lagna_sun,
            varga_methods=list(varga_methods) or None,
        )
        return Chart(execute({"op": "chart", **req}))

    def panchanga(self, ctx: Context, ayanamsha: str = "lahiri", natal_moon_deg: Optional[float] = None,
                  sunrise: Optional[str] = None) -> Dict[str, Any]:
        """Panchanga and day divisions at a moment and place (tithi, vara, nakshatra, yoga, karana, sunrise,
        hora, choghadiya, rahu kalam...). ``natal_moon_deg`` adds tara bala and chandra bala."""
        chart = self._base(ctx, "vedic", self._cfg(zodiac="sidereal", ayanamsha=ayanamsha), sunrise=sunrise)
        return execute({"op": "panchanga", "chart": chart, "natal_moon_deg": natal_moon_deg})

    def matching(self, a: Context, b: Context, ayanamsha: str = "lahiri") -> Dict[str, Any]:
        """Ashtakoota compatibility. ``a`` takes the role classical tables give the groom, ``b`` the bride."""
        cfg = self._cfg(zodiac="sidereal", ayanamsha=ayanamsha)
        return execute({"op": "matching", "a": self._base(a, None, cfg), "b": self._base(b, None, cfg)})

    def ephemeris(self, start_utc: str, step_days: float, count: int, bodies: Optional[Iterable[str]] = None,
                  zodiac: str = "tropical", ayanamsha: Optional[str] = None) -> Dict[str, Any]:
        """Longitudes at regular steps: transit animation, secondary progressions (one day per year)."""
        req = {"op": "ephemeris", "start_utc": start_utc, "step_days": step_days, "count": count,
               "bodies": list(bodies) if bodies else None, "config": self._cfg(zodiac=zodiac, ayanamsha=ayanamsha)}
        if self.spk_path:
            req["spk_path"] = self.spk_path
        return execute({k: v for k, v in req.items() if v is not None})

    def rise_set(self, latitude: float, longitude: float, date: str, timezone: Optional[str] = None,
                 utc_offset_minutes: Optional[int] = None, days: int = 1, bodies: Optional[Iterable[str]] = None,
                 sunrise: Optional[str] = None, twilight: bool = True, altitude_m: Optional[float] = None) -> Dict[str, Any]:
        """Rise, upper transit and set of bodies (default Sun and Moon), and civil, nautical and astronomical
        twilight, for ``days`` local days from ``date`` at a place."""
        req = {"op": "rise_set", "place": {"latitude": latitude, "longitude": longitude, "altitude_m": altitude_m},
               "date": date, "timezone": timezone, "utc_offset_minutes": utc_offset_minutes, "days": days,
               "bodies": list(bodies) if bodies else None, "sunrise": sunrise, "twilight": twilight,
               "config": self._cfg()}
        req["place"] = {k: v for k, v in req["place"].items() if v is not None}
        if self.spk_path:
            req["spk_path"] = self.spk_path
        return execute({k: v for k, v in req.items() if v is not None})

    def returns(self, natal: Context, start_utc: str, end_utc: str, body: str = "sun", zodiac: str = "tropical",
                ayanamsha: Optional[str] = None, latitude: Optional[float] = None, longitude: Optional[float] = None,
                charts: bool = True) -> Dict[str, Any]:
        """Every return of ``body`` (default the Sun) to its natal longitude in [start, end), with a chart at each;
        ``latitude``/``longitude`` cast relocated return charts."""
        system = "vedic" if zodiac == "sidereal" else "western"
        natal_req = self._base(natal, system, self._cfg(zodiac=zodiac, ayanamsha=ayanamsha))
        req = {"op": "returns", "natal": natal_req, "body": body, "start_utc": start_utc, "end_utc": end_utc,
               "charts": charts, "place": _place(latitude, longitude)}
        return execute({k: v for k, v in req.items() if v is not None})

    def synastry(self, a: Context, b: Context, orb_scale: Optional[float] = None, composite: bool = True,
                 davison: bool = True) -> Dict[str, Any]:
        """Two charts compared: inter-aspects, house overlays, a midpoint composite and a Davison chart."""
        req = {"op": "synastry", "a": self._base(a, "western", self._cfg()), "b": self._base(b, "western", self._cfg()),
               "orb_scale": orb_scale, "composite": composite, "davison": davison}
        return execute({k: v for k, v in req.items() if v is not None})

    def gochara(self, natal: Context, start_utc: str, end_utc: str, ayanamsha: str = "lahiri",
                bodies: Optional[Iterable[str]] = None) -> Dict[str, Any]:
        """Transits counted from the natal Moon and lagna, with retrograde and combustion intervals."""
        natal_req = self._base(natal, "vedic", self._cfg(zodiac="sidereal", ayanamsha=ayanamsha))
        req = {"op": "gochara", "natal": natal_req, "start_utc": start_utc, "end_utc": end_utc,
               "bodies": list(bodies) if bodies else None}
        return execute({k: v for k, v in req.items() if v is not None})

    def eclipses(self, start_utc: str, end_utc: str, kinds: Optional[Iterable[str]] = None) -> Dict[str, Any]:
        """Solar and lunar eclipses in [start, end): type, time of greatest eclipse and magnitude."""
        req = {"op": "eclipses", "start_utc": start_utc, "end_utc": end_utc, "kinds": list(kinds) if kinds else None,
               "config": self._cfg()}
        if self.spk_path:
            req["spk_path"] = self.spk_path
        return execute({k: v for k, v in req.items() if v is not None})

    def ingresses(self, body: str, start_utc: str, end_utc: str, zodiac: str = "sidereal",
                  ayanamsha: Optional[str] = "lahiri") -> Dict[str, Any]:
        """Every sign ingress of a body in a range, including retrograde re-entries."""
        req = {"op": "ingresses", "body": body, "start_utc": start_utc, "end_utc": end_utc,
               "config": self._cfg(zodiac=zodiac, ayanamsha=ayanamsha if zodiac == "sidereal" else None)}
        if self.spk_path:
            req["spk_path"] = self.spk_path
        return execute(req)

    def stations(self, body: str, start_utc: str, end_utc: str, zodiac: str = "sidereal",
                 ayanamsha: Optional[str] = "lahiri", tolerance_seconds: Optional[float] = None) -> Dict[str, Any]:
        """Stations of a body in [start, end) and the retrograde intervals they bound, clipped at the range."""
        req = {"op": "stations", "body": body, "start_utc": start_utc, "end_utc": end_utc,
               "tolerance_seconds": tolerance_seconds,
               "config": self._cfg(zodiac=zodiac, ayanamsha=ayanamsha if zodiac == "sidereal" else None)}
        if self.spk_path:
            req["spk_path"] = self.spk_path
        return execute({k: v for k, v in req.items() if v is not None})

    def crossings(self, body: str, point_deg: float, start_utc: str, end_utc: str,
                  aspects: Optional[Iterable[float]] = None, zodiac: str = "sidereal",
                  ayanamsha: Optional[str] = "lahiri", touch_orb_deg: Optional[float] = None,
                  tolerance_seconds: Optional[float] = None, max_results: Optional[int] = None) -> Dict[str, Any]:
        """When a body reaches a longitude (or an aspect to it) in [start, end), with direction; touches apart."""
        req = {"op": "crossings", "body": body, "point_deg": point_deg, "start_utc": start_utc, "end_utc": end_utc,
               "aspects": list(aspects) if aspects is not None else None, "touch_orb_deg": touch_orb_deg,
               "tolerance_seconds": tolerance_seconds, "max_results": max_results,
               "config": self._cfg(zodiac=zodiac, ayanamsha=ayanamsha if zodiac == "sidereal" else None)}
        if self.spk_path:
            req["spk_path"] = self.spk_path
        return execute({k: v for k, v in req.items() if v is not None})

    def ashtakavarga_transits(self, natal: Context, start_utc: str, end_utc: str, ayanamsha: str = "lahiri",
                              bodies: Optional[Iterable[str]] = None) -> Dict[str, Any]:
        """The natal BAV and SAV cells each transiting graha reads, as [start, end) segments between ingresses.
        A node is answered as unavailable (no table exists for it)."""
        chart = self._base(natal, "vedic", self._cfg(zodiac="sidereal", ayanamsha=ayanamsha))
        req = {"op": "ashtakavarga_transits", "natal": chart, "start_utc": start_utc, "end_utc": end_utc,
               "bodies": list(bodies) if bodies is not None else None}
        return execute({k: v for k, v in req.items() if v is not None})

    def muhurta_windows(self, latitude: float, longitude: float, start_utc: str, end_utc: str,
                        timezone: Optional[str] = None, profile: Optional[str] = None,
                        disable: Iterable[str] = (), constraints: Iterable[Any] = (), show: Iterable[str] = (),
                        ayanamsha: str = "lahiri", sunrise: Optional[str] = None) -> Dict[str, Any]:
        """Muhurta windows at an event place: factor timelines in [start, end) cut at every boundary, evaluated
        against a profile (``eshana methods``) and/or constraints, and merged where nothing changes."""
        return execute(self._muhurta(latitude, longitude, start_utc, end_utc, timezone, profile, disable, constraints,
                                     show, ayanamsha, sunrise) | {"op": "muhurta_windows"})

    def personal_calendar(self, natal: Context, latitude: float, longitude: float, start_utc: str, end_utc: str,
                          timezone: Optional[str] = None, profile: Optional[str] = "personal_general",
                          disable: Iterable[str] = (), constraints: Iterable[Any] = (), show: Iterable[str] = (),
                          ayanamsha: str = "lahiri", sunrise: Optional[str] = None,
                          format: str = "json") -> Dict[str, Any]:
        """Muhurta windows with tara and chandra bala from ``natal``, day summaries in the event zone, and an
        export (``format`` = ``csv`` or ``ics`` puts the file in ``content``). The birth place and the event place
        (``latitude``, ``longitude``) stay apart."""
        m = self._muhurta(latitude, longitude, start_utc, end_utc, timezone, profile, disable, constraints, show,
                          ayanamsha, sunrise)
        nat = self._base(natal, "vedic", self._cfg(zodiac="sidereal", ayanamsha=ayanamsha))
        return execute({"op": "personal_calendar", "natal": nat, "muhurta": m, "format": format})

    def pair_calendar(self, a: Context, b: Context, latitude: float, longitude: float, start_utc: str, end_utc: str,
                      timezone: Optional[str] = None, profile: Optional[str] = None, disable: Iterable[str] = (),
                      constraints: Iterable[Any] = (), show: Iterable[str] = (), ayanamsha: str = "lahiri",
                      sunrise: Optional[str] = None, format: str = "json") -> Dict[str, Any]:
        """Windows that suit both people (each one's tara and chandra bala) and the event constraints (default
        profile ``pair_general``); each window is explained by both people's factors and the event's."""
        m = self._muhurta(latitude, longitude, start_utc, end_utc, timezone, profile, disable, constraints, show,
                          ayanamsha, sunrise)
        cfg = self._cfg(zodiac="sidereal", ayanamsha=ayanamsha)
        return execute({"op": "pair_calendar", "a": self._base(a, None, cfg), "b": self._base(b, None, cfg),
                        "muhurta": m, "format": format})

    def _muhurta(self, lat: float, lon: float, start: str, end: str, tz: Optional[str], profile: Optional[str],
                 disable: Iterable[str], constraints: Iterable[Any], show: Iterable[str], ayanamsha: str,
                 sunrise: Optional[str]) -> Dict[str, Any]:
        m: Dict[str, Any] = {"place": {"latitude": lat, "longitude": lon}, "start_utc": start, "end_utc": end,
                             "timezone": tz, "profile": profile, "disable": list(disable),
                             "constraints": list(constraints), "show": list(show), "sunrise": sunrise,
                             "config": self._cfg(zodiac="sidereal", ayanamsha=ayanamsha)}
        if self.spk_path:
            m["spk_path"] = self.spk_path
        return {k: v for k, v in m.items() if v is not None}

    def progressions(self, natal: Context, at_utc: str, angle_method: Optional[str] = None,
                     zodiac: str = "tropical", ayanamsha: Optional[str] = None,
                     year_days: Optional[float] = None) -> Dict[str, Any]:
        """Secondary progressions to ``at_utc``: fractional age, positions at birth + age days, and angles under
        ``angle_method`` (``solar_arc_longitude``, ``naibod_ra``, ``quotidian``); without one they are unavailable."""
        nat = self._base(natal, None, self._cfg(zodiac=zodiac, ayanamsha=ayanamsha))
        req = {"op": "progressions", "natal": nat, "at_utc": at_utc, "angle_method": angle_method, "year_days": year_days}
        return execute({k: v for k, v in req.items() if v is not None})

    @staticmethod
    def synthetic_chart(positions: Iterable[Dict[str, Any]], ascendant_deg: Optional[float] = None) -> Dict[str, Any]:
        """Vedic factors from hand-placed sidereal positions (``[{"body": "saturn", "longitude_deg": 190}]``); no
        ephemeris is used, the result is labelled synthetic, and time-dependent factors are unavailable."""
        req = {"op": "synthetic_chart", "positions": list(positions), "ascendant_deg": ascendant_deg}
        return execute({k: v for k, v in req.items() if v is not None})

    @staticmethod
    def research_window_score(windows: Iterable[Dict[str, Any]]) -> Dict[str, Any]:
        """Experimental model ``research.window_score@0.1``: order passing muhurta windows (from muhurta_windows or a
        calendar). Specified in docs/design/research-models.md; a research hypothesis, not a reading."""
        return execute({"op": "research_window_score", "windows": list(windows)})

    @staticmethod
    def kp_table() -> Dict[str, Any]:
        """The KP 1-249 horary numbers: sign, star lord, sub lord and arc of each."""
        return execute({"op": "kp_table"})

    def kp_horary(self, number: int, ctx: Context, ayanamsha: str = "krishnamurti",
                  occupancy: Optional[str] = None) -> Dict[str, Any]:
        """KP horary chart. ``number`` (1-249) fixes the Ascendant at the start of its arc; ``ctx`` is the moment and
        place of judgement, kept apart from the number. ``occupancy`` is a ``kp_cusp_occupancy`` method id."""
        chart = self._base(ctx, "vedic", self._cfg(zodiac="sidereal", ayanamsha=ayanamsha),
                           kp_cusp_occupancy=occupancy)
        return execute({"op": "kp_horary", "number": number, "chart": chart})

    @staticmethod
    def methods() -> Dict[str, Any]:
        """Every method family and id the engine implements (an unknown id is refused with ESH-METHOD)."""
        return execute({"op": "methods"})

    @staticmethod
    def rulesets() -> Dict[str, Any]:
        """The built-in YAML rule packs, verbatim."""
        return execute({"op": "rulesets"})

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
        extended: bool = False,
        reference_date: Optional[str] = None,
        gender: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Name and date numerology under one named school. ``gender`` (``male`` or ``female``) picks the Kua number. ``extended=True`` adds every school's full set of numbers
        (Pythagorean, Chaldean, Indian/Vedic and the Lo Shu grid); ``reference_date`` (YYYY-MM-DD) sets the personal
        year, month and day cycles."""
        req = {
            "op": "numerology",
            "name": name,
            "date_of_birth": date_of_birth,
            "school": school,
            "keep_master_numbers": keep_master_numbers,
            "y_policy": y_policy,
            "life_path_method": life_path_method,
            "reference_year": reference_year,
            "extended": extended or None,
            "reference_date": reference_date,
            "gender": gender,
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
