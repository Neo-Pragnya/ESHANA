@cap_uncertain_time
Feature: Birth-time uncertainty is first class

  Scenario: An unknown birth time gives intervals and no houses
    When I run "eshana chart western --date 1992-05-28 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --zodiac tropical"
    Then it succeeds
    And the JSON output at "input.birth_time_quality" equals "date_only"
    And the JSON output at "houses" equals null
    And the JSON output at "warnings[0].code" equals "ESH-TIME-UNCERTAIN"
    And the output contains "longitude_interval_deg"

  Scenario: A sign that could change inside the window is flagged, not guessed
    When I run "eshana chart vedic --date 1992-05-28 --tz Asia/Kolkata"
    Then it succeeds
    And the output contains ""stable_within_uncertainty": false"
    And the output contains "candidates"

  Scenario: A small uncertainty keeps the houses and records the quality
    When I run "eshana chart western --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --zodiac tropical --uncertainty-minutes 5"
    Then it succeeds
    And the JSON output at "input.birth_time_quality" equals "approximate"
    And the output contains "ascendant"

  Scenario: The dasha is withheld when the Moon's longitude is uncertain
    When I run "eshana chart vedic --date 1992-05-28 --tz Asia/Kolkata --dasha"
    Then it succeeds
    And the output does not contain "vedic.dasha.vimshottari"
