@cap_ephemeris_events
Feature: Ephemeris tables, ingresses, stations and crossings

  Scenario: An ephemeris table at regular steps
    When I run "eshana ephemeris --start 2026-01-01T00:00:00Z --step-days 1 --count 10 --bodies sun,moon"
    Then it succeeds
    And the JSON output at "rows" has at least 10 items

  Scenario: The Sun enters every sidereal sign once a year
    When I run "eshana ingresses --body sun --start 2026-01-01T00:00:00Z --end 2027-01-01T00:00:00Z --zodiac sidereal --ayanamsha lahiri"
    Then it succeeds
    And the JSON output at "ingresses" has at least 12 items

  Scenario: Retrograde re-entries are found and labelled
    When I run "eshana ingresses --body saturn --start 1990-01-01T00:00:00Z --end 1997-01-01T00:00:00Z --zodiac sidereal --ayanamsha lahiri"
    Then it succeeds
    And the output contains "retrograde"

  Scenario: An oversized table is refused
    When I run "eshana ephemeris --start 2026-01-01T00:00:00Z --count 50000"
    Then the exit code is 2

  Scenario: Mercury's stations bound its retrograde intervals
    When I run "eshana stations --body mercury --start 2024-01-01T00:00:00Z --end 2025-01-01T00:00:00Z --zodiac tropical"
    Then it succeeds
    And the JSON output at "stations" has at least 7 items
    And the JSON output at "retrograde_intervals" has at least 3 items
    And the output contains "clipped_start"

  Scenario: A planet crossing a point three times in a retrograde loop
    When I run "eshana crossings --body saturn --point 335 --start 2024-01-01T00:00:00Z --end 2027-01-01T00:00:00Z --zodiac sidereal --ayanamsha lahiri"
    Then it succeeds
    And the JSON output at "events" has at least 3 items
    And the output contains "retrograde"

  Scenario: A reversed search range is refused
    When I run "eshana stations --body mars --start 2025-01-01T00:00:00Z --end 2024-01-01T00:00:00Z --zodiac tropical"
    Then the exit code is 2
    And the output contains "ESH-RANGE-INVALID"

  Scenario: Transit Ashtakavarga is a step function that changes at ingresses
    When I run "eshana ashtakavarga-transits --start 2025-01-01T00:00:00Z --end 2026-01-01T00:00:00Z --transit-bodies saturn,mean_node --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 --ayanamsha lahiri"
    Then it succeeds
    And the JSON output at "grahas.saturn.segments" has at least 2 items
    And the output contains ""status": "unavailable""
    And the output contains "cells_read"

  Scenario: Secondary progressions state the age and the angle method
    When I run "eshana progress --at 2020-05-15T00:00:00Z --angles naibod_ra --utc 1990-05-15T09:00:00Z --lat 28.61 --lon 77.21 --zodiac tropical"
    Then it succeeds
    And the output contains ""kind": "progressions""
    And the output contains "age_years"
    And the output contains ""id": "naibod_ra""

  Scenario: Without an angle method progressed angles are unavailable, not invented
    When I run "eshana progress --at 2020-05-15T00:00:00Z --utc 1990-05-15T09:00:00Z --lat 28.61 --lon 77.21 --zodiac tropical"
    Then it succeeds
    And the output contains ""status": "unavailable""
