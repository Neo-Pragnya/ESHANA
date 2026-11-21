"""Birth-chart accuracy suite.

Layers (see docs/guides/accuracy.md):

1. ESHANA vs Swiss Ephemeris on 44 charts (16 cities, 1852-2140): planets, nodes, ayanamsha, Ascendant, MC and
   cusps for whole-sign, equal and Placidus houses, in both zodiacs.
2. Categorical agreement: rashi, nakshatra, pada, nakshatra lord, bhava and every varga, re-derived from the Swiss
   longitude. Values within tolerance of a boundary are skipped and counted, never silently passed.
3. Re-derivation: an independent Python implementation of the tradition rules applied to ESHANA's own longitudes must
   reproduce ESHANA's derived attributes exactly (sign, nakshatra, pada, lord, bhava, D3-D12, Vimshottari).
4. Published values: the Lahiri definition (Calendar Reform Committee, 1956) and a recorded ProKerala chart.
5. Civil time: ESHANA's civil-to-UTC conversion against Python's zoneinfo, an independent tz implementation.

Run: ``make accuracy-suite`` (add ``ESHANA_SPK=/path/de440s.bsp`` to include the spk provider).
"""
from __future__ import annotations

import datetime as dt

import pytest

import lib

REF = lib.load("swiss_charts.json")["charts"]
PUB = lib.load("published.json")
TOL = lib.tolerances()
PROVIDERS = lib.providers()


def charts_for(provider: str):
    return [r for r in REF if r["range"] == "analytic" or provider == "spk"]


CASES = [pytest.param(p, spk, row, id=f"{p}:{row['id']}") for p, spk in PROVIDERS for row in charts_for(p)]
BODIES = list(lib.PLANETS) + ["mean_node", "true_node"]


def _env(row, provider, spk, **kw):
    env = lib.eshana_chart(row, provider, spk, **kw)
    assert env["kind"] != "error", env
    return env


def _dt_corr(env, row, speed):
    """Degrees a body moves over the delta-T difference between ESHANA and the Swiss reference."""
    return speed * (env["input"]["delta_t_s"] - row["delta_t_s"]) / 86400.0


# ── 1. positions, ayanamsha, angles, cusps ─────────────────────────────────────────────────────────────────────────
@pytest.mark.parametrize("provider,spk,row", CASES)
def test_positions_match_swiss(provider, spk, row):
    fails = []
    for node in ("mean", "true"):
        env = _env(row, provider, spk, node=node)
        pos = lib.positions(env)
        for b, p in pos.items():
            if b == "south_node":
                continue
            corr = _dt_corr(env, row, p["speed_deg_per_day"])
            tol = TOL["body_arcsec"][provider][b]
            for frame, key in (("sidereal", "longitude_deg"), ("tropical", "tropical_longitude_deg")):
                d = abs(lib.adiff(p[key] - corr, row[frame][b]["lon"])) * 3600
                if d > tol:
                    fails.append(f"{b} {frame}: {d:.2f}\" > {tol}\"")
        south = pos["south_node"]
        node_id = "mean_node" if node == "mean" else "true_node"
        assert abs(lib.adiff(south["longitude_deg"], pos[node_id]["longitude_deg"] + 180)) < 1e-9
    assert not fails, f"{row['id']} ({provider}): " + "; ".join(fails)


@pytest.mark.parametrize("provider,spk,row", CASES)
def test_ayanamsha_matches_swiss(provider, spk, row):
    env = _env(row, provider, spk)
    sun = lib.positions(env)["sun"]
    effective = lib.adiff(sun["tropical_longitude_deg"], sun["longitude_deg"]) % 360
    d = abs(effective - row["ayanamsha"]["lahiri"]) * 3600
    assert d <= TOL["ayanamsha_arcsec"], f"effective Lahiri {effective:.6f} vs {row['ayanamsha']['lahiri']:.6f}"


@pytest.mark.parametrize("provider,spk,row", CASES)
@pytest.mark.parametrize("system,zodiac", [("vedic", "sidereal"), ("western", "tropical")])
@pytest.mark.parametrize("hs,code", [("whole_sign", "W"), ("equal", "E"), ("placidus", "P")])
def test_houses_match_swiss(provider, spk, row, system, zodiac, hs, code):
    env = lib.eshana_chart(row, provider, spk, system=system, houses=hs)
    if code not in row["houses"]:
        assert env["kind"] == "error" and env["error"]["code"] == "ESH-LATITUDE", \
            "Placidus inside the polar circles must be refused, not approximated"
        return
    assert env["kind"] != "error", env
    ref = row["houses"][code][zodiac]
    h = env["houses"]
    tol = TOL["angles_arcsec"]
    assert abs(lib.adiff(h["ascendant"]["longitude_deg"], ref["asc"])) * 3600 <= tol
    assert abs(lib.adiff(h["midheaven"]["longitude_deg"], ref["mc"])) * 3600 <= tol
    worst = max(abs(lib.adiff(c, ref["cusps"][i])) * 3600 for i, c in enumerate(h["cusps_deg"]))
    limit = 1e-6 if code == "W" else TOL["cusps_arcsec"]
    assert worst <= limit, f"{hs} {zodiac}: worst cusp {worst:.3f}\""


# ── 2. categorical agreement with values re-derived from the Swiss longitudes ─────────────────────────────────────
WIDTH = {1: 30.0, 2: 15.0, 3: 10.0, 4: 7.5, 5: 6.0, 6: 5.0, 7: 30.0 / 7.0, 8: 3.75, 9: 10.0 / 3.0, 10: 3.0, 12: 2.5, 16: 30 / 16, 20: 1.5,
         24: 1.25, 27: 30 / 27, 30: 1.0, 40: 0.75, 45: 30 / 45, 60: 0.5}  # D30's unequal parts end on whole degrees


@pytest.mark.parametrize("provider,spk,row", CASES)
def test_categorical_values_match_swiss(provider, spk, row):
    env = _env(row, provider, spk)
    at = lib.attrs(env)
    mismatches, skipped = [], 0
    pos = lib.positions(env)
    for b in [x for x in BODIES if x != "true_node"]:
        # the reference longitude differs from ESHANA's by the tolerance plus the stated delta-T difference
        tol_deg = TOL["body_arcsec"][provider][b] / 3600.0 + abs(_dt_corr(env, row, pos[b]["speed_deg_per_day"]))
        swiss = row["sidereal"][b]["lon"]
        if lib.near_boundary(swiss, lib.PADA, tol_deg):
            skipped += 1
            continue  # sign, nakshatra and pada boundaries are all multiples of a pada
        ra = at[f"vedic.{b}.rashi"]["index"]
        nk = at[f"vedic.{b}.nakshatra"]
        if ra != lib.sign(swiss):
            mismatches.append(f"{b} rashi {ra} vs {lib.sign(swiss)}")
        if nk["index"] != lib.nakshatra(swiss) or nk["pada"] != lib.pada(swiss):
            mismatches.append(f"{b} nakshatra {nk['index']}/{nk['pada']} vs {lib.nakshatra(swiss)}/{lib.pada(swiss)}")
        for d in lib.DIVISIONS[1:]:
            if lib.near_boundary(swiss % 30.0, WIDTH[d], tol_deg):
                skipped += 1
                continue
            got = at[f"vedic.{b}.d{d}"]["index"]
            if got != lib.varga(swiss, d):
                mismatches.append(f"{b} D{d} {got} vs {lib.varga(swiss, d)}")
    asc = row["houses"]["W"]["sidereal"]["asc"]
    if not lib.near_boundary(asc, lib.PADA, TOL["angles_arcsec"] / 3600):
        if at["vedic.lagna.rashi"]["index"] != lib.sign(asc):
            mismatches.append("lagna rashi")
        if at["vedic.lagna.nakshatra"]["index"] != lib.nakshatra(asc):
            mismatches.append("lagna nakshatra")
    assert not mismatches, f"{row['id']}: {mismatches} (skipped {skipped} boundary-ambiguous values)"


# ── 3. independent re-derivation from ESHANA's own longitudes (exact) ────────────────────────────────────────────
@pytest.mark.parametrize("provider,spk,row", CASES)
def test_rederivation_reproduces_every_derived_attribute(provider, spk, row):
    env = _env(row, provider, spk)
    at = lib.attrs(env)
    pos = lib.positions(env)
    asc = env["houses"]["ascendant"]["longitude_deg"]
    lagna_sign = lib.sign(asc)
    for b, p in pos.items():
        lon = p["longitude_deg"]
        assert at[f"vedic.{b}.rashi"]["index"] == lib.sign(lon), b
        nk = at[f"vedic.{b}.nakshatra"]
        assert (nk["index"], nk["pada"]) == (lib.nakshatra(lon), lib.pada(lon)), b
        assert nk["lord"].lower() == lib.nak_lord(nk["index"]), b
        assert at[f"vedic.{b}.bhava"]["house"] == (lib.sign(lon) - lagna_sign) % 12 + 1, b
        for d in lib.DIVISIONS[1:]:
            assert at[f"vedic.{b}.d{d}"]["index"] == lib.varga(lon, d), f"{b} D{d}"
    for d in lib.DIVISIONS[1:]:
        assert at[f"vedic.lagna.d{d}"]["index"] == lib.varga(asc, d), f"lagna D{d}"


@pytest.mark.parametrize("provider,spk,row", CASES)
def test_vimshottari_rederived_and_against_swiss_moon(provider, spk, row):
    env = _env(row, provider, spk)
    v = lib.attrs(env)["vedic.dasha.vimshottari"]
    moon = lib.positions(env)["moon"]["longitude_deg"]
    mine = lib.vimshottari(moon, env["input"]["instant_utc"])
    assert abs(v["balance_at_birth_years"] - mine["balance_years"]) < 1e-9
    assert [m["lord"].lower() for m in v["mahadashas"]] == [m["lord"] for m in mine["mahadashas"]]
    for got, exp in zip(v["mahadashas"], mine["mahadashas"]):
        for k in ("start", "end"):
            delta = abs((lib.parse_utc(got[f"{k}_utc"]) - exp[k]).total_seconds())
            assert delta <= TOL["dasha_seconds_vs_rederived"], f"{got['lord']} {k}: {delta:.3f}s"
        ants = got["antardashas"]
        assert ants[-1]["end_utc"] == got["end_utc"]
        order = lib.DASHA_ORDER[lib.DASHA_ORDER.index(got["lord"].lower()):] + \
            lib.DASHA_ORDER[:lib.DASHA_ORDER.index(got["lord"].lower())]
        if got is v["mahadashas"][0]:
            # sub-periods that ended before birth are left out; the first one shown is clipped at birth
            assert [a["lord"].lower() for a in ants] == order[9 - len(ants):]
            assert ants[0]["clipped_at_birth"] and ants[0]["start_utc"][:19] == env["input"]["instant_utc"][:19]
        else:
            assert [a["lord"].lower() for a in ants] == order
        maha_years = lib.DASHA_YEARS[got["lord"].lower()]
        for a in ants:
            if a.get("clipped_at_birth"):
                continue
            days = (lib.parse_utc(a["end_utc"]) - lib.parse_utc(a["start_utc"])).total_seconds() / 86400
            expect = maha_years * lib.DASHA_YEARS[a["lord"].lower()] / 120 * 365.25
            assert abs(days - expect) < 1e-3, f"{got['lord']}/{a['lord']}: {days} vs {expect} days"
    # against the Swiss Moon: the first lord must agree, the balance by no more than the Moon tolerance allows
    swiss_moon = row["sidereal"]["moon"]["lon"]
    tol_deg = TOL["body_arcsec"][provider]["moon"] / 3600 + abs(
        moon - (moon - _dt_corr(env, row, lib.positions(env)["moon"]["speed_deg_per_day"])))
    if not lib.near_boundary(swiss_moon, lib.NAKS, tol_deg):
        ref = lib.vimshottari(swiss_moon, row["utc"])
        assert ref["lord"] == mine["lord"]
        allowed = tol_deg / lib.NAKS * lib.DASHA_YEARS[ref["lord"]]
        assert abs(ref["balance_years"] - mine["balance_years"]) <= allowed + 1e-12


# ── 4. published values ────────────────────────────────────────────────────────────────────────────────────────────
@pytest.mark.parametrize("provider,spk", PROVIDERS, ids=[p for p, _ in PROVIDERS])
def test_lahiri_definition_1956(provider, spk):
    d = PUB["lahiri_definition"]
    req = {"op": "positions", "when": {"utc": d["utc"]}, "bodies": ["sun"],
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri"}}
    if spk:
        req["spk_path"] = spk
    sun = lib.run(req)["positions"][0]
    effective = (sun["tropical_longitude_deg"] - sun["longitude_deg"]) % 360
    assert abs(effective - d["true_ayanamsha_deg"]) * 3600 <= d["tolerance_arcsec"], effective


def _norm(name: str) -> str:
    return name.lower().replace("h", "").replace(" ", "")


SNAPS = [pytest.param(p, spk, s, id=f"{p}:{s['id']}") for p, spk in PROVIDERS for s in PUB["prokerala_snapshots"]]


@pytest.mark.parametrize("provider,spk,snap", SNAPS)
def test_prokerala_snapshot(provider, spk, snap):
    req = {"op": "chart", "system": "vedic",
           "when": {"date": snap["date"], "time": snap["time"], "utc_offset_minutes": snap["utc_offset_minutes"]},
           "place": {"latitude": snap["lat"], "longitude": snap["lon"]},
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri", "node_model": snap["node"],
                      "house_system": "whole_sign"}}
    if spk:
        req["spk_path"] = spk
    env = lib.run(req)
    assert env["kind"] != "error", env
    at, pos = lib.attrs(env), lib.positions(env)
    pt = TOL["published"]
    problems = []
    for b, printed in snap["printed"].items():
        if b == "lagna":
            lon = env["houses"]["ascendant"]["longitude_deg"]
            tol = pt["unstated_location_deg"]
        else:
            lon = pos[b]["longitude_deg"]
            tol = pt["rounding_deg"] + (TOL["body_arcsec"][provider].get(b, TOL["body_arcsec"][provider]["mean_node"])
                                        + pt["ayanamsha_variant_arcsec"]) / 3600
        d = abs(lib.adiff(lon, printed["full_deg"]))
        if d > tol:
            problems.append(f"{b}: {lon:.4f} vs printed {printed['full_deg']} ({d * 3600:.0f}\" > {tol * 3600:.0f}\")")
        if _norm(at[f"vedic.{b}.rashi"]["name"]) != _norm(printed["rashi"]):
            problems.append(f"{b} rashi {at[f'vedic.{b}.rashi']['name']} vs {printed['rashi']}")
        nk = at[f"vedic.{b}.nakshatra"]
        if _norm(nk["name"]) != _norm(printed["nakshatra"]) or nk.get("pada") != printed["pada"]:
            problems.append(f"{b} nakshatra {nk['name']} {nk.get('pada')} vs {printed['nakshatra']} {printed['pada']}")
        if printed.get("retrograde") is not None and pos[b]["retrograde"] != printed["retrograde"]:
            problems.append(f"{b} retrograde")
    for b, house in snap["printed_houses_whole_sign"].items():
        if b != "lagna" and at[f"vedic.{b}.bhava"]["house"] != house:
            problems.append(f"{b} bhava {at[f'vedic.{b}.bhava']['house']} vs printed {house}")
    assert not problems, problems


# ── 5. civil time ──────────────────────────────────────────────────────────────────────────────────────────────────
@pytest.mark.parametrize("row", REF, ids=[r["id"] for r in REF])
def test_civil_time_matches_zoneinfo(row):
    env = lib.eshana_chart(row, "analytic", None) if row["range"] == "analytic" else lib.run(
        {"op": "positions", "bodies": ["sun"], "config": {"zodiac": "tropical"},
         "when": {"date": row["date"], "time": row["time"], "timezone": row["timezone"]}})
    if env["kind"] == "error":  # positions outside the analytic range still convert the time first
        assert env["error"]["code"] == "ESH-RANGE", env
        return
    assert env["input"]["instant_utc"] == row["utc"]
    local = dt.datetime.fromisoformat(f"{row['date']}T{row['time']}")
    assert env["input"]["civil_offset_s"] == int(
        (local.replace(tzinfo=dt.timezone.utc) - lib.parse_utc(row["utc"])).total_seconds())
