@cap_determinism
Feature: Determinism

  Scenario: The same request gives the same bytes twice
    When I run "eshana --compact chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --divisions 1,9,10 --dasha > a.json && eshana --compact chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --divisions 1,9,10 --dasha > b.json && cmp a.json b.json && echo same-bytes"
    Then it succeeds
    And the output contains "same-bytes"

  Scenario: A raw request file produces the same result as the flags
    Given the file "req.json" with content:
      """
      {"op": "chart", "system": "vedic", "assume_defaults": true,
       "when": {"date": "1992-05-28", "time": "20:05", "timezone": "Asia/Kolkata"},
       "place": {"latitude": 17.385, "longitude": 78.487}}
      """
    When I run "eshana --compact request req.json > viafile.json && eshana --compact chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 > viaflags.json && cmp viafile.json viaflags.json && echo equal"
    Then it succeeds
    And the output contains "equal"
