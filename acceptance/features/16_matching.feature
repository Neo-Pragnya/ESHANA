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
