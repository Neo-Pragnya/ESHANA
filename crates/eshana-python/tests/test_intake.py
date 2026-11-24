"""Guided chart creation (``eshana-report new``): the form waits, refuses invalid input with the engine's code while
keeping what was typed, writes one chart for a valid entry, never overwrites, and the server stops afterwards."""
import http.client
import json
import threading
import urllib.parse

import pytest

from eshana import intake

FAST = {"transit_days": 0, "progression_years": 0, "sections": ["overview", "charts"]}
VALID = {"name": "Asha Rao", "date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata", "latitude": "28.61",
         "longitude": "77.21", "place_label": "Delhi", "ayanamsha": "lahiri", "houses": "whole_sign", "locale": "en"}


class Session:
    """Run :func:`intake.run` in a thread and talk to it like a browser."""

    def __init__(self, out, **kw):
        self.url = None
        self.result = None
        self.error = None
        ready = threading.Event()

        def on_ready(u):
            self.url = u
            ready.set()

        def go():
            try:
                self.result = intake.run(out, open_browser=False, timeout=60, grace=5, report_options=FAST,
                                         on_ready=on_ready, **kw)
            except BaseException as e:  # noqa: BLE001
                self.error = e
                ready.set()

        self.thread = threading.Thread(target=go, daemon=True)
        self.thread.start()
        assert ready.wait(10), "server did not start"
        u = urllib.parse.urlsplit(self.url)
        self.host, self.port, self.base = u.hostname, u.port, u.path

    def req(self, method, path, body=None, headers=None):
        c = http.client.HTTPConnection(self.host, self.port, timeout=60)
        h = {"Host": f"127.0.0.1:{self.port}"}
        if body is not None:
            body = urllib.parse.urlencode(body)
            h["Content-Type"] = "application/x-www-form-urlencoded"
        h.update(headers or {})
        c.request(method, path, body=body, headers=h)
        r = c.getresponse()
        out = (r.status, dict(r.getheaders()), r.read().decode("utf-8", "replace"))
        c.close()
        return out

    def submit(self, form, **h):
        return self.req("POST", self.base + "chart", form, h)


def test_form_then_chart_then_the_server_stops(tmp_path):
    s = Session(tmp_path)
    code, hdr, page = s.req("GET", s.base)
    assert code == 200 and 'action="' + s.base + "chart" in page
    assert "default-src 'none'" in hdr["Content-Security-Policy"]
    assert 'src="http' not in page and 'href="http' not in page  # no external assets
    code, hdr, _ = s.submit(VALID)
    assert code == 303 and hdr["Location"] == s.base + "chart"
    code, _, report = s.req("GET", s.base + "chart")
    assert code == 200 and "Asha Rao" in report
    s.thread.join(15)
    assert not s.thread.is_alive() and s.error is None
    path = s.result["path"]
    assert path.name == "asha-rao-1990-05-15.html" and path.read_text(encoding="utf-8") == report
    with pytest.raises(OSError):  # the server is gone
        s.req("GET", s.base)


def test_invalid_input_returns_to_the_form_and_keeps_waiting(tmp_path):
    s = Session(tmp_path)
    code, _, page = s.submit({**VALID, "date": "", "latitude": "123"})
    assert code == 400 and 'role="alert"' in page and "birth date is required" in page
    assert 'value="Asha Rao"' in page  # what was typed is kept
    # a clock time inside the spring-forward gap: the engine's own code comes back
    code, _, page = s.submit({**VALID, "date": "2021-03-14", "time": "02:30", "timezone": "America/New_York",
                              "latitude": "40.7", "longitude": "-74.0"})
    assert code == 422 and "ESH-" in page and "02:30" in page
    assert not list(tmp_path.iterdir()), "nothing is written for an invalid entry"
    assert s.thread.is_alive()
    assert s.submit(VALID)[0] == 303
    s.req("GET", s.base + "chart")
    s.thread.join(15)
    assert s.result["path"].exists()


def test_refusals_token_host_origin_and_encoding(tmp_path):
    s = Session(tmp_path)
    assert s.req("GET", "/wrong-token/")[0] == 404
    assert s.req("GET", s.base, headers={"Host": "evil.example:80"})[0] == 403
    assert s.submit(VALID, Origin="http://evil.example")[0] == 403
    c = s.req("POST", s.base + "chart", None, {"Content-Type": "application/json", "Content-Length": "2"})
    assert c[0] == 415
    assert not list(tmp_path.iterdir())
    s.submit(VALID)
    s.req("GET", s.base + "chart")
    s.thread.join(15)


def test_never_overwrites_and_saves_a_profile_revision(tmp_path):
    (tmp_path / "asha-rao-1990-05-15.html").write_text("keep me", encoding="utf-8")
    lib = tmp_path / "profiles.json"
    s = Session(tmp_path, library=lib)
    s.submit({**VALID, "save_profile": "on"})
    s.req("GET", s.base + "chart")
    s.thread.join(15)
    assert (tmp_path / "asha-rao-1990-05-15.html").read_text(encoding="utf-8") == "keep me"
    assert s.result["path"].name == "asha-rao-1990-05-15-2.html"
    data = json.loads(lib.read_text(encoding="utf-8"))
    rev = data["profiles"]["Asha Rao"][0]
    assert rev["resolved"]["birth_utc"].startswith("1990-05-15T09:00:00")
    assert s.result["profile"]["revision"] == 1


def test_unknown_time_and_fixed_offset(tmp_path):
    s = Session(tmp_path / "one.html")
    form = {**VALID, "time": "", "time_unknown": "on", "timezone": "", "utc_offset_minutes": "321",
            "date": "1879-12-30"}
    assert s.submit(form)[0] == 303
    s.req("GET", s.base + "chart")
    s.thread.join(15)
    assert s.result["path"] == tmp_path / "one.html"


def test_timeout_writes_nothing(tmp_path):
    with pytest.raises(intake.IntakeTimeout):
        intake.run(tmp_path, open_browser=False, timeout=0.3)
    assert not list(tmp_path.iterdir())


def test_parse_form_rules():
    _, e = intake.parse_form({"date": "1990-01-01", "time": "10:00", "timezone": "Asia/Kolkata",
                              "utc_offset_minutes": "330"})
    assert "utc_offset_minutes" in e
    _, e = intake.parse_form({"date": "1990-01-01", "time": "10:00", "timezone": "Asia/Kolkata", "latitude": "10"})
    assert e == {"longitude": "give both latitude and longitude, or neither"}
    d, e = intake.parse_form({"date": "1990-01-01", "time": "10:00", "timezone": "Asia/Kolkata", "fold": "earliest"})
    assert not e and d["birth"]["time"] == "10:00:00" and d["birth"]["fold"] == "earliest"
