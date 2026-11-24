"""Guided chart creation: ``eshana-report new``.

The command starts a one-shot web form on this machine, opens it in the browser and waits. When the person submits
birth details, the engine checks them (a time inside a daylight-saving gap or overlap, a missing zone and the like
come back to the form with the engine's own error code). Once the details are valid, the chart studio is written to
a new HTML file, shown in the browser, and the command stops, printing the file's path.

The flow, start to finish:

1. Bind an HTTP server to ``127.0.0.1`` on a free port, in a background thread. Every URL carries a random token, and
   requests with any other ``Host`` or ``Origin`` are refused, so other sites and other users on the machine cannot
   drive it.
2. Open the form (``--no-browser`` only prints its URL). The main thread waits on an event, with a timeout
   (``--timeout``, 30 minutes by default) and Ctrl-C.
3. On submit, resolve the civil input with the engine. An invalid entry goes back to the form, keeping what was typed,
   and the command keeps waiting.
4. On a valid entry, build the report, write it to a new file (an existing file is never overwritten), optionally
   save the birth as a new profile revision (``--library``), and redirect the browser to the report.
5. Once the report has been served, or after a short grace period, shut the server down and return the path.

Nothing leaves the machine: the form has no external assets and the server answers only on the loopback interface.
"""
from __future__ import annotations

import html
import re
import secrets
import sys
import threading
import webbrowser
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Tuple
from urllib.parse import parse_qs, urlsplit

AYANAMSHAS = ("lahiri", "raman", "krishnamurti", "fagan_bradley", "yukteshwar", "jn_bhasin", "ushashashi", "sassanian",
              "deluce", "djwhal_khul")
HOUSES = ("whole_sign", "equal", "placidus")
LOCALES = ("en", "hi")
FOLDS = ("reject", "earliest", "latest")
MAX_BODY = 32 * 1024

EXIT_TIMEOUT = 3
EXIT_INTERRUPTED = 130


class IntakeTimeout(TimeoutError):
    """No valid chart was submitted before the timeout."""


@dataclass
class _State:
    token: str
    out: Path
    out_is_dir: bool
    defaults: Dict[str, Any]
    report_options: Dict[str, Any]
    library: Optional[Path]
    lock: threading.Lock = field(default_factory=threading.Lock)
    written: threading.Event = field(default_factory=threading.Event)
    served: threading.Event = field(default_factory=threading.Event)
    path: Optional[Path] = None
    profile: Optional[Dict[str, Any]] = None
    host_ok: Tuple[str, ...] = ()


# ---------------------------------------------------------------------------------------------------------------------
# Form parsing and validation


def _zones() -> List[str]:
    try:
        import zoneinfo
        return sorted(z for z in zoneinfo.available_timezones() if "/" in z and not z.startswith(("Etc/", "SystemV/")))
    except Exception:  # pragma: no cover - zoneinfo always exists on 3.9+, the data may not
        return []


def _num(v: str, what: str, lo: float, hi: float, errs: Dict[str, str]) -> Optional[float]:
    if v == "":
        return None
    try:
        x = float(v)
    except ValueError:
        errs[what] = "not a number"
        return None
    if not lo <= x <= hi:
        errs[what] = f"must be between {lo:g} and {hi:g}"
        return None
    return x


def parse_form(form: Dict[str, str]) -> Tuple[Dict[str, Any], Dict[str, str]]:
    """Turn the submitted fields into birth details and report options, or field errors (field id -> message)."""
    errs: Dict[str, str] = {}
    g = lambda k: (form.get(k) or "").strip()  # noqa: E731
    b: Dict[str, Any] = {"name": g("name") or None, "gender": g("gender") or None, "place_label": g("place_label") or None}
    if b["gender"] not in (None, "male", "female"):
        errs["gender"] = "male, female or blank"
    date = g("date")
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", date):
        errs["date"] = "the birth date is required (YYYY-MM-DD)"
    b["date"] = date
    unknown = g("time_unknown") == "on"
    t = g("time")
    if unknown:
        b["time"] = None
    elif re.fullmatch(r"\d{2}:\d{2}(:\d{2})?", t):
        b["time"] = t if len(t) == 8 else t + ":00"
    else:
        errs["time"] = "give the birth time (HH:MM), or tick 'time unknown'"
    tz, off = g("timezone"), g("utc_offset_minutes")
    b["timezone"] = tz or None
    b["utc_offset_minutes"] = None
    if off:
        try:
            b["utc_offset_minutes"] = int(off)
        except ValueError:
            errs["utc_offset_minutes"] = "whole minutes east of UTC, e.g. 330"
    if tz and off:
        errs["utc_offset_minutes"] = "give a time zone or a fixed offset, not both"
    if not tz and not off:
        errs["timezone"] = "a time zone (e.g. Asia/Kolkata) or a fixed UTC offset is required"
    fold = g("fold") or "reject"
    if fold not in FOLDS:
        errs["fold"] = "unknown choice"
    b["fold"] = None if fold == "reject" else fold
    b["uncertainty_minutes"] = _num(g("uncertainty_minutes"), "uncertainty_minutes", 0, 1440, errs)
    lat = _num(g("latitude"), "latitude", -90, 90, errs)
    lon = _num(g("longitude"), "longitude", -180, 180, errs)
    if (lat is None) != (lon is None) and "latitude" not in errs and "longitude" not in errs:
        errs["latitude" if lat is None else "longitude"] = "give both latitude and longitude, or neither"
    b["latitude"], b["longitude"] = lat, lon
    opts = {"ayanamsha": g("ayanamsha") or "lahiri", "houses": g("houses") or "whole_sign", "locale": g("locale") or "en",
            "live": g("live") == "on", "save_profile": g("save_profile") == "on"}
    for k, allowed in (("ayanamsha", AYANAMSHAS), ("houses", HOUSES), ("locale", LOCALES)):
        if opts[k] not in allowed:
            errs[k] = "unknown choice"
    return {"birth": b, "options": opts}, errs


def check_birth(b: Dict[str, Any]) -> Dict[str, Any]:
    """The engine's resolution of the civil input (raises :class:`eshana.EshanaError` when it is not a valid moment)."""
    from .profiles import civil_to_utc
    return civil_to_utc({k: b.get(k) for k in ("date", "time", "timezone", "utc_offset_minutes", "fold", "latitude",
                                          "longitude")})


def _slug(s: Optional[str]) -> str:
    s = re.sub(r"[^A-Za-z0-9]+", "-", (s or "").strip()).strip("-").lower()
    return s[:40] or "chart"


def _new_path(state: _State, b: Dict[str, Any]) -> Path:
    if not state.out_is_dir:
        p = state.out
    else:
        p = state.out / f"{_slug(b.get('name'))}-{b['date']}.html"
    p.parent.mkdir(parents=True, exist_ok=True)
    stem, k = p.with_suffix(""), 2
    while p.exists():  # never overwrite: name-date-2.html, -3, ...
        p = Path(f"{stem}-{k}.html")
        k += 1
    return p


def build(state: _State, data: Dict[str, Any]) -> Path:
    """Write the report for valid birth details and return its path."""
    from . import Context
    from .report import birth_report
    b, o = data["birth"], data["options"]
    ctx = Context.from_local_birth(b["date"], b["time"], b["timezone"], b["latitude"], b["longitude"],
                                   b["utc_offset_minutes"], b["fold"], b["uncertainty_minutes"])
    page = birth_report(ctx, name=b["name"], gender=b["gender"], place_label=b["place_label"], ayanamsha=o["ayanamsha"],
                        houses=o["houses"], live=o["live"], locale=o["locale"], **state.report_options)
    path = _new_path(state, b)
    path.write_text(page, encoding="utf-8")
    if state.library and o["save_profile"]:
        from .profiles import ProfileLibrary
        fields = {k: b[k] for k in ("date", "time", "timezone", "utc_offset_minutes", "fold", "latitude", "longitude",
                                    "place_label") if b.get(k) is not None}
        state.profile = ProfileLibrary(state.library).save(b["name"] or path.stem, **fields)
    return path


# ---------------------------------------------------------------------------------------------------------------------
# The page

_CSS = """
:root{color-scheme:light dark;--bg:#f7f5f0;--fg:#1d1b17;--mut:#5f5a50;--line:#d8d2c4;--acc:#8a4b12;--err:#a4161a;--card:#fff}
@media (prefers-color-scheme:dark){:root{--bg:#16140f;--fg:#ece6d8;--mut:#a69f90;--line:#3a352b;--acc:#e0a060;--err:#ff8a80;--card:#1f1c16}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.5 system-ui,sans-serif}
main{max-width:46rem;margin:0 auto;padding:2rem 1rem 4rem}h1{font-size:1.6rem;margin:0 0 .25rem}
p.lead{color:var(--mut);margin:0 0 1.5rem}fieldset{border:1px solid var(--line);border-radius:10px;background:var(--card);
margin:0 0 1rem;padding:1rem 1.1rem}legend{font-weight:600;padding:0 .3rem}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(13rem,1fr));gap:.8rem 1rem}
label{display:block;font-size:.9rem;font-weight:600;margin-bottom:.2rem}.hint{font-size:.8rem;color:var(--mut);margin:.2rem 0 0}
input,select{width:100%;font:inherit;padding:.45rem .55rem;border:1px solid var(--line);border-radius:6px;background:var(--bg);color:var(--fg)}
input:focus,select:focus,button:focus{outline:3px solid var(--acc);outline-offset:1px}
.check{display:flex;gap:.5rem;align-items:center;font-weight:400}.check input{width:auto}
.bad input,.bad select{border-color:var(--err)}.msg{color:var(--err);font-size:.85rem;margin:.2rem 0 0}
.alert{border:2px solid var(--err);border-radius:10px;padding:.8rem 1rem;margin:0 0 1rem;background:var(--card)}
.alert code{font-weight:700}details summary{cursor:pointer;font-weight:600}
button{font:inherit;font-weight:600;padding:.7rem 1.4rem;border:0;border-radius:8px;background:var(--acc);color:var(--bg);cursor:pointer}
button[disabled]{opacity:.6;cursor:progress}footer{color:var(--mut);font-size:.8rem;margin-top:1.5rem}
"""

_JS = """
(function(){
  var f=document.getElementById('f'), tz=document.getElementById('timezone'), off=document.getElementById('utc_offset_minutes');
  if(tz && !tz.value && !off.value){try{tz.value=Intl.DateTimeFormat().resolvedOptions().timeZone||'';}catch(e){}}
  var u=document.getElementById('time_unknown'), t=document.getElementById('time');
  function sync(){t.disabled=u.checked; t.required=!u.checked;}
  u.addEventListener('change',sync); sync();
  f.addEventListener('submit',function(){var b=document.getElementById('go'); b.disabled=true; b.textContent='Computing the chart\\u2026';});
  var a=document.querySelector('.alert'); if(a){a.focus();}
})();
"""


def _esc(v: Any) -> str:
    return html.escape("" if v is None else str(v), quote=True)


def render_form(state: _State, values: Optional[Dict[str, str]] = None, errors: Optional[Dict[str, str]] = None,
                engine_error: Optional[Tuple[str, str]] = None) -> str:
    """The form, pre-filled with ``values`` (what was typed) or the command's defaults."""
    v = dict(state.defaults)
    v.update(values or {})
    errors = errors or {}

    def fld(fid: str, label: str, typ: str = "text", hint: str = "", **attrs: Any) -> str:
        extra = " ".join(f'{k.rstrip("_")}="{_esc(x)}"' for k, x in attrs.items() if x not in (None, False))
        err = errors.get(fid)
        desc = f' aria-describedby="{fid}-m"' if err or hint else ""
        inv = ' aria-invalid="true"' if err else ""
        return (f'<div class="{"bad" if err else ""}"><label for="{fid}">{label}</label>'
                f'<input id="{fid}" name="{fid}" type="{typ}" value="{_esc(v.get(fid))}"{desc}{inv} {extra}>'
                + (f'<p class="msg" id="{fid}-m">{_esc(err)}</p>' if err else
                   f'<p class="hint" id="{fid}-m">{hint}</p>' if hint else "") + "</div>")

    def sel(fid: str, label: str, options: Tuple[str, ...], names: Optional[Dict[str, str]] = None) -> str:
        cur = v.get(fid) or options[0]
        opts = "".join(f'<option value="{o}"{" selected" if o == cur else ""}>{_esc((names or {}).get(o, o))}</option>'
                       for o in options)
        err = errors.get(fid)
        return (f'<div class="{"bad" if err else ""}"><label for="{fid}">{label}</label><select id="{fid}" name="{fid}">'
                f'{opts}</select>' + (f'<p class="msg">{_esc(err)}</p>' if err else "") + "</div>")

    def chk(fid: str, label: str) -> str:
        on = v.get(fid) in ("on", True)
        return (f'<label class="check" for="{fid}"><input type="checkbox" id="{fid}" name="{fid}"'
                f'{" checked" if on else ""}> {label}</label>')

    alert = ""
    if engine_error or errors:
        items = "".join(f"<li><a href=\"#{k}\">{_esc(k.replace('_', ' '))}</a>: {_esc(m)}</li>" for k, m in errors.items())
        eng = ""
        if engine_error and engine_error[0]:
            eng = f"<p>The engine refused these details: <code>{_esc(engine_error[0])}</code> {_esc(engine_error[1])}</p>"
        elif engine_error:
            eng = f"<p>The details could not be used: {_esc(engine_error[1])}</p>"
        alert = (f'<div class="alert" role="alert" tabindex="-1"><strong>The chart was not created.</strong>{eng}'
                 + (f"<ul>{items}</ul>" if items else "") + "</div>")
    zones = "".join(f'<option value="{z}">' for z in _zones())
    library = (f'<fieldset><legend>Profile library</legend>{chk("save_profile", "Save this birth as a profile revision")}'
               f'<p class="hint">Library file: {_esc(state.library)}. A save adds a revision; nothing is overwritten.</p>'
               f'</fieldset>') if state.library else ""
    return f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>New chart · ESHANA</title>
<style>{_CSS}</style></head><body><main>
<h1>New chart</h1><p class="lead">Enter the birth details. The chart is computed on this machine and written to a file;
this page then shows it and the command finishes.</p>
{alert}
<form id="f" method="post" action="/{state.token}/chart" autocomplete="off" novalidate>
<fieldset><legend>Person</legend><div class="grid">
{fld("name", "Name", hint="Shown in the report and used for numerology")}
{sel("gender", "Gender", ("", "female", "male"), {"": "not given"})}
</div></fieldset>
<fieldset><legend>Birth moment</legend><div class="grid">
{fld("date", "Date", "date", required=True)}
{fld("time", "Local time", "time", hint="As on the birth record", step="1")}
<div>{chk("time_unknown", "Time unknown")}<p class="hint">The report then leaves out everything that needs the hour.</p></div>
{fld("timezone", "Time zone", hint="IANA zone of the birth place; the historic offset is applied", list="zones",
     placeholder="Asia/Kolkata")}
</div><datalist id="zones">{zones}</datalist>
<details{" open" if any(k in errors for k in ("utc_offset_minutes", "fold", "uncertainty_minutes")) or engine_error else ""}>
<summary>Offset, ambiguous times and uncertainty</summary><div class="grid">
{fld("utc_offset_minutes", "Fixed UTC offset (minutes)", "number", "Only when no zone applies, e.g. 330 for +05:30")}
{sel("fold", "Clock time that happened twice", FOLDS, {"reject": "ask me (refuse)", "earliest": "the earlier one",
                                                       "latest": "the later one"})}
{fld("uncertainty_minutes", "Uncertainty (± minutes)", "number", "Kept with the chart; the report flags what it can change",
     min="0", step="any")}
</div></details>
</fieldset>
<fieldset><legend>Birth place</legend><div class="grid">
{fld("place_label", "Place name", hint="Only a label; the coordinates are used")}
{fld("latitude", "Latitude (°, north +)", "number", step="any", min="-90", max="90")}
{fld("longitude", "Longitude (°, east +)", "number", step="any", min="-180", max="180")}
</div><p class="hint">Without coordinates there are no houses or lagna.</p></fieldset>
<fieldset><legend>Settings</legend><div class="grid">
{sel("ayanamsha", "Ayanāṃśa", AYANAMSHAS)}
{sel("houses", "Vedic houses", HOUSES)}
{sel("locale", "Interface language", LOCALES, {"en": "English", "hi": "हिन्दी"})}
<div>{chk("live", "Embed the engine (offline recompute, ~4 MB)")}</div>
</div></fieldset>
{library}
<button id="go" type="submit">Create chart</button>
</form>
<footer>Served only to this machine (127.0.0.1). Closing this tab does not stop the command; Ctrl-C in the terminal does.</footer>
</main><script>{_JS}</script></body></html>"""


# ---------------------------------------------------------------------------------------------------------------------
# The server


def _handler(state: _State) -> type:
    class H(BaseHTTPRequestHandler):
        server_version = "eshana-intake"
        sys_version = ""

        def log_message(self, *a: Any) -> None:  # quiet: the terminal shows only the URL and the result
            pass

        def _send(self, code: int, body: bytes, ctype: str = "text/html; charset=utf-8",
                  headers: Optional[Dict[str, str]] = None, strict: bool = True) -> None:
            self.send_response(code)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Referrer-Policy", "same-origin")  # "no-referrer" makes Chrome send Origin: null
            self.send_header("X-Frame-Options", "DENY")
            if strict:
                self.send_header("Content-Security-Policy", "default-src 'none'; style-src 'unsafe-inline'; "
                                 "script-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'")
            for k, x in (headers or {}).items():
                self.send_header(k, x)
            self.end_headers()
            if self.command != "HEAD":
                self.wfile.write(body)

        def _guard(self) -> Optional[str]:
            """The path after the token, or None after answering a refusal."""
            if self.headers.get("Host", "") not in state.host_ok:
                self._send(403, b"forbidden host", "text/plain")
                return None
            origin = self.headers.get("Origin")
            if origin is not None and origin not in {f"http://{h}" for h in state.host_ok}:
                self._send(403, b"forbidden origin", "text/plain")
                return None
            parts = urlsplit(self.path).path.split("/", 2)
            if len(parts) < 2 or not secrets.compare_digest(parts[1], state.token):
                self._send(404, b"not found", "text/plain")
                return None
            return parts[2] if len(parts) > 2 else ""

        def do_GET(self) -> None:  # noqa: N802
            rest = self._guard()
            if rest is None:
                return
            if rest == "":
                if state.written.is_set():
                    self._send(303, b"", headers={"Location": f"/{state.token}/chart"})
                else:
                    self._send(200, render_form(state).encode())
            elif rest == "chart" and state.path is not None:
                self._send(200, state.path.read_bytes(), strict=False)
                state.served.set()
            else:
                self._send(404, b"not found", "text/plain")

        def do_POST(self) -> None:  # noqa: N802
            rest = self._guard()
            if rest is None:
                return
            if rest != "chart":
                self._send(404, b"not found", "text/plain")
                return
            n = int(self.headers.get("Content-Length") or 0)
            if n > MAX_BODY:
                self._send(413, b"too large", "text/plain")
                return
            if not self.headers.get("Content-Type", "").startswith("application/x-www-form-urlencoded"):
                self._send(415, b"form encoding expected", "text/plain")
                return
            raw = parse_qs(self.rfile.read(n).decode("utf-8", "replace"), keep_blank_values=True)
            form = {k: x[-1] for k, x in raw.items()}
            from . import EshanaError
            with state.lock:
                if state.written.is_set():  # a second click after success: show the chart already written
                    self._send(303, b"", headers={"Location": f"/{state.token}/chart"})
                    return
                data, errs = parse_form(form)
                if errs:
                    self._send(400, render_form(state, form, errs).encode())
                    return
                try:
                    check_birth(data["birth"])
                    state.path = build(state, data)
                except EshanaError as e:
                    self._send(422, render_form(state, form, engine_error=(e.code, e.message)).encode())
                    return
                except (ValueError, OSError) as e:
                    self._send(422, render_form(state, form, engine_error=("", str(e))).encode())
                    return
                state.written.set()
            self._send(303, b"", headers={"Location": f"/{state.token}/chart"})

        do_HEAD = do_GET

    return H


def run(out: Path | str = ".", *, port: int = 0, open_browser: bool = True, timeout: float = 1800.0,
        grace: float = 15.0, library: Optional[Path | str] = None, defaults: Optional[Dict[str, Any]] = None,
        report_options: Optional[Dict[str, Any]] = None, on_ready: Optional[Callable[[str], None]] = None) -> Dict[str, Any]:
    """Serve the form until one valid chart is written; return ``{"path", "profile", "url"}``.

    ``out`` is a directory (a file name is made from the name and date) or an ``.html`` file. Raises
    :class:`IntakeTimeout` after ``timeout`` seconds without a valid submission; Ctrl-C raises ``KeyboardInterrupt``.
    Either way the server is shut down before returning.
    """
    out_p = Path(out).expanduser()
    state = _State(token=secrets.token_urlsafe(18), out=out_p, out_is_dir=out_p.suffix.lower() != ".html",
                   defaults={k: ("on" if x is True else x) for k, x in (defaults or {}).items() if x not in (None, False)},
                   report_options=dict(report_options or {}), library=Path(library).expanduser() if library else None)
    srv = ThreadingHTTPServer(("127.0.0.1", port), _handler(state))
    srv.daemon_threads = True
    real_port = srv.server_address[1]
    state.host_ok = (f"127.0.0.1:{real_port}", f"localhost:{real_port}")
    url = f"http://127.0.0.1:{real_port}/{state.token}/"
    th = threading.Thread(target=srv.serve_forever, kwargs={"poll_interval": 0.2}, name="eshana-intake", daemon=True)
    th.start()
    try:
        (on_ready or (lambda u: None))(url)
        if open_browser:
            webbrowser.open(url)
        if not state.written.wait(timeout):
            raise IntakeTimeout(f"no chart was submitted within {timeout:g} s")
        state.served.wait(grace)  # let the browser fetch the report before the server goes away
    finally:
        srv.shutdown()
        srv.server_close()
        th.join(5)
    return {"path": state.path, "profile": state.profile, "url": url}


def main(argv: Optional[List[str]] = None) -> int:
    """``eshana-report new``: the command-line front of :func:`run`."""
    import argparse
    p = argparse.ArgumentParser(prog="eshana-report new",
                                description="Open a form in the browser, wait for birth details, write the chart, stop.")
    p.add_argument("-o", "--output", default=".", help="directory for the new file, or an .html path (default: .)")
    p.add_argument("--library", help="profile library file; offers to save the birth as a profile revision")
    p.add_argument("--port", type=int, default=0, help="local port (default: any free port)")
    p.add_argument("--no-browser", action="store_true", help="print the URL instead of opening a browser")
    p.add_argument("--timeout", type=float, default=1800, help="seconds to wait for a submission (default 1800)")
    p.add_argument("--ayanamsha", default="lahiri", choices=AYANAMSHAS, help="pre-selected ayanāṃśa")
    p.add_argument("--houses", default="whole_sign", choices=HOUSES, help="pre-selected Vedic houses")
    p.add_argument("--locale", default="en", choices=LOCALES, help="pre-selected interface language")
    p.add_argument("--live", action="store_true", help="pre-tick embedding the engine")
    p.add_argument("--transit-days", type=int, help="as in eshana-report (0 = no transits)")
    p.add_argument("--progression-years", type=int, help="as in eshana-report (0 = none)")
    a = p.parse_args(argv)
    ropts = {k: x for k, x in {"transit_days": a.transit_days, "progression_years": a.progression_years}.items()
             if x is not None}
    defaults = {"ayanamsha": a.ayanamsha, "houses": a.houses, "locale": a.locale, "live": a.live,
                "save_profile": bool(a.library)}

    def ready(url: str) -> None:
        print(f"eshana-report new: waiting for birth details at {url}", file=sys.stderr)
        print("  (Ctrl-C to stop)", file=sys.stderr)

    try:
        res = run(a.output, port=a.port, open_browser=not a.no_browser, timeout=a.timeout, library=a.library,
                  defaults=defaults, report_options=ropts, on_ready=ready)
    except IntakeTimeout as e:
        print(f"eshana-report new: {e}; nothing written", file=sys.stderr)
        return EXIT_TIMEOUT
    except KeyboardInterrupt:
        print("eshana-report new: stopped; nothing written", file=sys.stderr)
        return EXIT_INTERRUPTED
    except OSError as e:
        print(f"eshana-report new: {e}", file=sys.stderr)
        return 2
    if res["profile"]:
        print(f"saved profile revision {res['profile']['revision']} to {a.library}", file=sys.stderr)
    print(res["path"])
    return 0
