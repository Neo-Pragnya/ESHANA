@cap_matching
Feature: Ashtakoota compatibility

  Scenario: Eight kootas, scored out of 36, with the rule pack named
    When I run "eshana matching --a 1990-05-15,14:30,Asia/Kolkata,28.61,77.21 --b 1992-08-20,09:15,Asia/Kolkata,19.08,72.88"
    Then it succeeds
    And the JSON output at "ashtakoota.max" equals 36
    And the JSON output at "ashtakoota.kootas" has at least 8 items
    And the output contains "vedic.matching.ashtakoota@0.2"
    And the output contains "manglik"
    And the JSON output at "interpretations" equals []

  Scenario: A partner without a usable date is refused
    When I run "eshana matching --a 1990-05-15,14:30,Asia/Kolkata --b nonsense"
    Then the exit code is 2

  Scenario: Supplementary factors sit beside the 36 points, each with its rule
    When I run "eshana matching --a 1990-05-15,14:30,Asia/Kolkata,28.61,77.21 --b 1992-08-20,09:15,Asia/Kolkata,19.076,72.8777"
    Then it succeeds
    And the output contains "matching.supplementary@0.1#rajju"
    And the output contains "mahendra"
    And the output contains "manglik_trace"
    And the output contains "pair_ashtakavarga"

  Scenario: A pair calendar explains each window by both people and the event
    When I run "eshana pair-calendar --a 1990-05-15,14:30,Asia/Kolkata,28.61,77.21 --b 1991-01-10,09:15,Asia/Kolkata,19.076,72.8777 --lat 19.07 --lon 72.88 --tz Asia/Kolkata --start 2026-11-01T00:00:00Z --end 2027-01-01T00:00:00Z"
    Then it succeeds
    And the output contains ""kind": "pair_calendar""
    And the output contains "explained"
