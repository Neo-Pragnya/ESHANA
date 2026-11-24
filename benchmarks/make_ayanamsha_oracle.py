#!/usr/bin/env python3
"""Regenerate benchmarks/golden/ayanamsha_oracle.json: mean ayanāṃśa of every model at several epochs.

pyswisseph (AGPL) is an offline oracle only: this script runs outside the build and its output is numbers.

    uv venv refenv && uv pip install --python refenv/bin/python pyswisseph
    refenv/bin/python benchmarks/make_ayanamsha_oracle.py

Values are swe.get_ayanamsa_ex(jd_tt, FLG_MOSEPH | FLG_NONUT): the mean ayanāṃśa without nutation, which is what
ESHANA subtracts after removing nutation from the apparent longitude. Epochs are TT Julian days.
"""

from __future__ import annotations

import json
from pathlib import Path

import swisseph as swe

OUT = Path(__file__).resolve().parent / "golden" / "ayanamsha_oracle.json"
MODELS = {"lahiri": "LAHIRI", "raman": "RAMAN", "krishnamurti": "KRISHNAMURTI", "fagan_bradley": "FAGAN_BRADLEY",
          "yukteshwar": "YUKTESHWAR", "jn_bhasin": "JN_BHASIN", "ushashashi": "USHASHASHI", "sassanian": "SASSANIAN",
          "deluce": "DELUCE", "djwhal_khul": "DJWHAL_KHUL",
          "true_citra": "TRUE_CITRA", "true_revati": "TRUE_REVATI", "true_pushya": "TRUE_PUSHYA", "true_mula": "TRUE_MULA"}
TRUE = [k for k in MODELS if k.startswith("true_")]
YEARS = [1800, 1850, 1900, 1950, 2000, 2025, 2050, 2100]


def main() -> None:
    rows = []
    for y in YEARS:
        jd = swe.julday(y, 1, 1, 12.0)
        vals = {}
        for k, name in MODELS.items():
            swe.set_sid_mode(getattr(swe, "SIDM_" + name), 0, 0)
            vals[k] = swe.get_ayanamsa_ex(jd, swe.FLG_MOSEPH | swe.FLG_NONUT)[1]
        rows.append({"year": y, "jd_tt": jd, "t_centuries_tt": (jd - 2451545.0) / 36525.0, "ayanamsha_deg": vals})
    # the star-anchored models carry annual aberration, so sample them through the year as well: 160 instants,
    # 1800-2200, about 2.5 years apart and walking through the seasons
    dense = []
    for i in range(160):
        jd = swe.julday(1800, 1, 1, 0.0) + i * 913.1
        vals = {}
        for k in TRUE:
            swe.set_sid_mode(getattr(swe, "SIDM_" + MODELS[k]), 0, 0)
            vals[k] = swe.get_ayanamsa_ex(jd, swe.FLG_MOSEPH | swe.FLG_NONUT)[1]
        dense.append({"jd_tt": jd, "t_centuries_tt": (jd - 2451545.0) / 36525.0, "ayanamsha_deg": vals})
    OUT.write_text(json.dumps({"source": f"pyswisseph {swe.version} get_ayanamsa_ex(FLG_NONUT)", "epochs": rows,
                               "true_epochs": dense}, indent=1) + "\n")
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
