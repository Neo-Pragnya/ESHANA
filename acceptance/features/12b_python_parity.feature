@cap_python_parity
Feature: The Python facade and the CLI agree

  # Same stated settings on both sides. (Relying on CLI defaults adds a recorded assumption, so the envelopes would
  # legitimately differ in provenance.assumptions only.)
  Scenario: A Vedic chart is identical through both
    Given the file "py_side.py" with content:
      """
      import json
      import eshana

      c = eshana.Context.from_local_birth(
          date="1992-05-28", time="20:05:00", timezone="Asia/Kolkata", latitude=17.385, longitude=78.487
      )
      chart = eshana.ChartEngine().vedic_chart(c, ayanamsha="lahiri", divisions=[1, 9], dasha=True)
      print(json.dumps(chart.raw))
      """
    And the file "compare_json.py" with content:
      """
      import json, sys
      a, b = (json.load(open(p)) for p in sys.argv[1:3])
      print("IDENTICAL" if a == b else "DIFFERENT")
      sys.exit(0 if a == b else 1)
      """
    When I run "eshana --compact chart vedic --date 1992-05-28 --time 20:05:00 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --ayanamsha lahiri --houses whole_sign --node mean --divisions 1,9 --dasha > cli.json"
    And I run "python py_side.py > py.json"
    And I run "python compare_json.py cli.json py.json"
    Then it succeeds
    And the output contains "IDENTICAL"

  Scenario: Python raises a typed error with the same code
    Given the file "typed_error.py" with content:
      """
      import eshana
      try:
          eshana.execute({"op": "positions", "when": {"utc": "1500-01-01T00:00:00Z"}})
      except eshana.EshanaError as e:
          print(e.code)
      """
    When I run "python typed_error.py"
    Then it succeeds
    And the output contains "ESH-RANGE"

  Scenario: The MCP server lists its tools and computes a chart over stdio
    When I run "eshana-mcp" with input:
      """
      {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "acceptance", "version": "1"}}}
      {"jsonrpc": "2.0", "method": "notifications/initialized"}
      {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
      {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "create_vedic_chart", "arguments": {"date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata", "latitude": 28.6139, "longitude": 77.209}}}
      {"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "create_vedic_chart", "arguments": {"date": "1990-05-15", "time": "14:30", "timezone": "Mars/Olympus"}}}
      """
    Then it succeeds
    And the output contains ""serverInfo": {"name": "eshana""
    And the output contains ""name": "find_eclipses""
    And the output contains "vedic.moon.nakshatra"
    And the output contains ""isError": true"
    And the output contains "ESH-"
