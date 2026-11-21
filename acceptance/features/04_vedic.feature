@cap_vedic
Feature: Vedic chart

  Background:
    Given a fresh project

  Scenario: A Hyderabad evening birth reads as expected
    When I run "eshana chart vedic --date 1992-05-28 --time 20:05:00 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --divisions 1,9 --dasha"
    Then it succeeds
    And the JSON output at "kind" equals "computed_chart"
    And the JSON output at "houses.ascendant.sign.name" equals "Dhanu"
    And the JSON output at "positions[1].sign.name" equals "Meena"
    And the output contains "Revati"
    And the output contains "vedic.moon.d9"
    And the output contains "vedic.dasha.vimshottari"

  Scenario: Derived values name their rule pack and tradition
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z --lat 17.385 --lon 78.487"
    Then it succeeds
    And the output contains "vedic.parashari@0.1"
    And the output contains ""tradition_id": "vedic.parashari""
    And the output contains ""epistemic": "tradition_rule""

  Scenario: Dasha structure and year convention
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z --dasha --year-convention 360"
    Then it succeeds
    And the output contains ""year_convention": "360""
    And the output contains "balance_at_birth_years"
    And the output contains "antardashas"

  Scenario: An unknown year convention is refused, not guessed
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z --dasha --year-convention lunar"
    Then the exit code is 2
    And the output contains "ESH-RULES"

  Scenario: A divisional chart that is not implemented says so
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z --divisions 60"
    Then the exit code is 2
    And the output contains "not implemented"

  Scenario: A Vedic chart is always sidereal
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z --zodiac tropical"
    Then the exit code is 2
    And the output contains "ESH-CONFIG"
