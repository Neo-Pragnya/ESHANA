"""The MCP server (BIND-05): the handshake, the tool list and real tool calls over stdio, as a client sees them."""
import json
import subprocess
import sys


def session(*messages):
    proc = subprocess.run([sys.executable, "-m", "eshana.mcp"], input="\n".join(json.dumps(m) for m in messages) + "\n",
                          capture_output=True, text=True, timeout=120)
    return [json.loads(line) for line in proc.stdout.splitlines() if line.strip()]


def test_handshake_tools_and_calls():
    out = session(
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
        {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "create_vedic_chart", "arguments": {
            "date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata", "latitude": 28.6139, "longitude": 77.209}}},
        {"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "create_vedic_chart", "arguments": {"date": "1990-05-15", "time": "14:30"}}},
        {"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "panchanga", "arguments": {
            "date": "2026-11-24", "time": "06:00", "timezone": "Asia/Kolkata", "latitude": 28.6, "longitude": 77.2}}},
        {"jsonrpc": "2.0", "id": 6, "method": "nope"},
    )
    by = {m["id"]: m for m in out}
    assert set(by) == {1, 2, 3, 4, 5, 6}  # nothing for the notification
    assert by[1]["result"]["protocolVersion"] == "2025-06-18" and by[1]["result"]["serverInfo"]["name"] == "eshana"
    names = {t["name"] for t in by[2]["result"]["tools"]}
    assert {"compute_positions", "create_vedic_chart", "compare_charts", "compute_human_design", "compute_numerology", "eshana_request"} <= names
    assert all(t["inputSchema"]["type"] == "object" for t in by[2]["result"]["tools"])
    chart = by[3]["result"]
    assert chart["isError"] is False and chart["structuredContent"]["kind"] == "computed_chart"
    refused = by[4]["result"]  # no time zone: the engine refuses, and the tool says why
    assert refused["isError"] is True and "ESH-" in refused["content"][0]["text"]
    assert by[5]["result"]["isError"] is False
    assert by[6]["error"]["code"] == -32601


def test_explain_derivation_traces_an_attribute_to_its_rule_and_inputs():
    args = {"date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata", "latitude": 28.6139, "longitude": 77.209}
    out = session(
        {"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "explain_derivation", "arguments": {**args, "attribute_id": "vedic.moon.nakshatra"}}},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "explain_derivation", "arguments": {**args, "attribute_id": "vedic.moon.horoscope"}}},
    )
    by = {m["id"]: m for m in out}
    d = by[1]["result"]["structuredContent"]
    assert d["kind"] == "derivation" and d["attribute"]["id"] == "vedic.moon.nakshatra"
    assert d["attribute"]["rule_version"] and [p["id"] for p in d["derived_from"]["positions"]] == ["moon"]
    assert d["provenance"]["engine_version"]
    assert by[2]["result"]["isError"] is True and "no attribute" in by[2]["result"]["content"][0]["text"]
