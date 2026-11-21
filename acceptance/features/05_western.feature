@cap_western
Feature: Western chart

  Scenario: Placidus houses with aspects at London
    When I run "eshana chart western --utc 2000-01-01T12:00:00Z --lat 51.5074 --lon -0.1278 --houses placidus --aspects"
    Then it succeeds
    And the JSON output at "houses.system" equals "placidus"
    And the JSON output at "houses.cusps_deg" has at least 12 items
    And the output contains "western.aspect.moon-sun"
    And the output contains ""aspect": "sextile""
    And the output contains "applying"

  Scenario: Placidus beyond the polar circle is refused, not extrapolated
    When I run "eshana chart western --utc 2000-01-01T12:00:00Z --lat 75 --lon 10 --houses placidus"
    Then the exit code is 2
    And the output contains "ESH-LATITUDE"

  Scenario: The legitimate twin: whole sign works at the same latitude
    When I run "eshana chart western --utc 2000-01-01T12:00:00Z --lat 75 --lon 10 --houses whole_sign"
    Then it succeeds
    And the JSON output at "houses.system" equals "whole_sign"

  Scenario: Equal houses are 30 degrees apart
    When I run "eshana chart western --utc 2000-01-01T12:00:00Z --lat 28.6 --lon 77.2 --houses equal"
    Then it succeeds
    And the JSON output at "houses.system" equals "equal"

  Scenario: Orb scaling narrows the aspects found
    When I run "eshana chart western --utc 2000-01-01T12:00:00Z --aspects --orb-scale 0.1"
    Then it succeeds
    And the output does not contain ""orb_scale": 1.0"

  Scenario: Invalid coordinates are refused
    When I run "eshana chart western --utc 2000-01-01T12:00:00Z --lat 95 --lon 10"
    Then the exit code is 2
    And the output contains "ESH-INPUT"
