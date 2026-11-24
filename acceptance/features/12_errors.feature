@cap_errors
Feature: Errors are clean and carry stable codes

  Scenario: A refusal prints an error envelope, a one-line message on stderr and exits 2
    When I run "eshana positions --utc 1500-01-01T00:00:00Z"
    Then the exit code is 2
    And the output contains ""kind": "error""
    And the output contains "ESH-RANGE"
    And the output does not contain "panicked"

  Scenario: explain knows every code in the catalogue
    Given the file "explain_all.py" with content:
      """
      import json, subprocess, sys
      codes = [e["code"] for e in json.loads(subprocess.check_output(["eshana", "doctor"]))["error_codes"]]
      assert len(codes) >= 11, codes
      for c in codes:
          r = subprocess.run(["eshana", "explain", c], capture_output=True, text=True)
          assert r.returncode == 0 and c in r.stdout, (c, r.stderr)
      print("all-explained", len(codes))
      """
    When I run "python explain_all.py"
    Then it succeeds
    And the output contains "all-explained"

  Scenario: An unknown code is refused
    When I run "eshana explain ESH-NOPE"
    Then the exit code is 2

  Scenario: Malformed request JSON is an input error
    Given the file "bad.json" with content:
      """
      {not json
      """
    When I run "eshana request bad.json"
    Then the exit code is 2

  Scenario: An unknown request field is refused, not ignored
    Given the file "req.json" with content:
      """
      {"op": "positions", "when": {"utc": "2000-01-01T12:00:00Z"}, "surprise": true}
      """
    When I run "eshana request req.json"
    Then the exit code is 2
    And the output contains "ESH-INPUT"

  Scenario: A missing SPK kernel is a clear configuration error
    When I run "env -u ESHANA_SPK eshana positions --utc 2000-01-01T12:00:00Z --provider spk"
    Then the exit code is 2
    And the output contains "ESH-CONFIG"
    And the output contains "naif.jpl.nasa.gov"

  Scenario: A search past its documented span is a work-limit error, not an input error
    When I run "eshana crossings --body mars --point 10 --start 1800-01-01T00:00:00Z --end 2090-01-01T00:00:00Z --zodiac tropical"
    Then the exit code is 2
    And the output contains "ESH-LIMIT"

  Scenario: An empty search range has its own code
    When I run "eshana crossings --body mars --point 10 --start 2024-01-01T00:00:00Z --end 2024-01-01T00:00:00Z --zodiac tropical"
    Then the exit code is 2
    And the output contains "ESH-RANGE-INVALID"

  Scenario: The method registry lists every family with stable ids
    When I run "eshana methods"
    Then it succeeds
    And the output contains ""kind": "methods""
    And the output contains "house_system"
    And the output contains "placidus"
