@cap_panchanga
Feature: Panchanga and the divisions of the day

  Scenario: The five limbs, sunrise and the day divisions
    When I run "eshana panchanga --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090"
    Then it succeeds
    And the JSON output at "panchanga.vara.english" equals "Tuesday"
    And the JSON output at "panchanga.tithi.paksha" equals "krishna"
    And the output contains "rahu_kalam"
    And the output contains "choghadiya"
    And the output contains "horas"
    And the output contains "moonrise"
    And the JSON output at "panchanga.sunrise_convention" equals "upper_limb_refracted"

  Scenario: The sunrise convention is a stated choice, never silent
    When I run "eshana panchanga --date 1996-12-07 --time 10:34 --offset-minutes 330 --lat 13.0878 --lon 80.2785 --sunrise centre_unrefracted"
    Then it succeeds
    And the JSON output at "panchanga.sunrise_convention" equals "centre_unrefracted"
    And the JSON output at "panchanga.special_lagnas.sun_taken_at" equals "birth"

  Scenario: Tara bala against a natal Moon
    When I run "eshana panchanga --utc 2026-11-21T06:00:00Z --lat 28.61 --lon 77.21 --natal-moon 271.89"
    Then it succeeds
    And the output contains "tara_bala"
    And the output contains "chandra_bala"

  Scenario: A place is required
    When I run "eshana panchanga --utc 2026-11-21T06:00:00Z"
    Then the exit code is 2
    And the output contains "needs a place"

  Scenario: Inside the polar night there is no sunrise, and the result says so
    When I run "eshana panchanga --utc 2026-12-21T12:00:00Z --lat 78.22 --lon 15.65"
    Then it succeeds
    And the output contains "does not rise or set"
