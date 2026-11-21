@cap_ephemeris_events
Feature: Ephemeris tables and ingress search

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
