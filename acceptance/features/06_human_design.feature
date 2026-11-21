@cap_human_design
Feature: Human Design calculation

  Scenario: The design instant is the 88-degree solar arc, not 88 days
    When I run "eshana human-design --date 1992-05-28 --time 20:05:00 --tz Asia/Kolkata --lat 17.385 --lon 78.487"
    Then it succeeds
    And the JSON output at "kind" equals "human_design"
    And the output matches ""design_arc_deg_achieved": (88\.0|87\.9999)"
    And the output matches ""design_offset_days": (8[5-9]|9[0-2])\."
    And the JSON output at "positions.personality.activations" has at least 13 items
    And the JSON output at "positions.design.activations" has at least 13 items

  Scenario: The BodyGraph is derived and names its rule pack
    When I run "eshana human-design --date 1992-05-28 --time 20:05:00 --tz Asia/Kolkata"
    Then it succeeds
    And the output contains "human_design.basic@0.1"
    And the output contains ""profile""
    And the output contains ""authority""
    And the output contains ""defined_centers""

  Scenario: The IP position is recorded as an assumption
    When I run "eshana human-design --date 1992-05-28 --time 20:05:00 --tz Asia/Kolkata"
    Then the output contains "ADR-009"

  Scenario: An unknown birth time is refused instead of guessed
    When I run "eshana human-design --date 1992-05-28 --tz Asia/Kolkata"
    Then the exit code is 2
    And the output contains "ESH-TIME-UNCERTAIN"

  Scenario: Human Design uses the tropical zodiac only
    When I run "eshana human-design --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --zodiac sidereal --ayanamsha lahiri"
    Then the exit code is 2
    And the output contains "ESH-CONFIG"
