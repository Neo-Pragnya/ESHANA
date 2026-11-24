@cap_kp_horary
Feature: KP horary from a number

  Scenario: The 1-249 table tiles the zodiac
    When I run "eshana kp-table"
    Then it succeeds
    And the JSON output at "table.entries" has at least 249 items
    And the output contains ""count": 249"

  Scenario: The number fixes the Ascendant; the moment and place are those of judgement
    When I run "eshana kp-horary --number 23 --utc 2026-11-23T10:00:00Z --lat 13.08 --lon 80.27"
    Then it succeeds
    And the output contains ""kind": "kp_horary""
    And the output contains ""judgement_utc": "2026-11-23T10:00:00Z""
    And the output contains ""longitude_deg": 30.0"

  Scenario: A number outside 1-249 is refused
    When I run "eshana kp-horary --number 250 --utc 2026-11-23T10:00:00Z --lat 13.08 --lon 80.27"
    Then the exit code is 2
    And the output contains "ESH-INPUT"

  Scenario: An unknown cusp-occupancy method is refused, not replaced by the default
    When I run "eshana chart vedic --utc 1990-05-15T09:00:00Z --lat 28.61 --lon 77.21 --zodiac sidereal --ayanamsha krishnamurti --features kp --kp-occupancy nearest"
    Then the exit code is 2
    And the output contains "ESH-METHOD"

  Scenario: The alternative occupancy is opt-in and named in the result
    When I run "eshana chart vedic --utc 1990-05-15T09:00:00Z --lat 28.61 --lon 77.21 --zodiac sidereal --ayanamsha krishnamurti --features kp --kp-occupancy cusp_midpoint"
    Then it succeeds
    And the output contains ""occupancy_method": "cusp_midpoint""
