@cap_muhurta
Feature: Muhurta windows and the personal calendar

  Scenario: A profile's windows are exact [start, end) intervals at the event place
    When I run "eshana muhurta --lat 19.07 --lon 72.88 --tz Asia/Kolkata --start 2026-11-01T00:00:00Z --end 2026-11-03T00:00:00Z --profile daily_avoidances"
    Then it succeeds
    And the output contains ""kind": "muhurta_windows""
    And the JSON output at "windows" has at least 2 items
    And the output contains "[start, end)"
    And the output contains "+05:30"

  Scenario: An impossible combination is empty and says which constraints removed it
    When I run "eshana muhurta --lat 19.07 --lon 72.88 --tz Asia/Kolkata --start 2026-11-01T00:00:00Z --end 2026-11-03T00:00:00Z --constraint abhijit_only,brahma_muhurta_only"
    Then it succeeds
    And the output contains "empty_because"

  Scenario: An experimental preset says what it lacks
    When I run "eshana muhurta --lat 19.07 --lon 72.88 --tz Asia/Kolkata --start 2026-11-01T00:00:00Z --end 2026-11-02T00:00:00Z --profile travel"
    Then it succeeds
    And the output contains ""status": "experimental""
    And the output contains "missing"

  Scenario: Polar night is unavailable, never a window
    When I run "eshana muhurta --lat 78.22 --lon 15.65 --tz Arctic/Longyearbyen --start 2026-12-20T00:00:00Z --end 2026-12-21T00:00:00Z --profile daily_avoidances"
    Then it succeeds
    And the output contains ""status": "unavailable""
    And the JSON output at "windows" equals []

  Scenario: The personal calendar keeps the birth place and the event place apart and exports ICS
    When I run "eshana calendar --birth 1990-05-15,14:30,Asia/Kolkata,28.61,77.21 --lat 19.07 --lon 72.88 --tz Asia/Kolkata --start 2026-11-01T00:00:00Z --end 2026-11-08T00:00:00Z --format ics"
    Then it succeeds
    And the output contains "BEGIN:VCALENDAR"
    And the output contains "BEGIN:VEVENT"

  Scenario: An unknown profile is refused with ESH-METHOD
    When I run "eshana muhurta --lat 19.07 --lon 72.88 --start 2026-11-01T00:00:00Z --end 2026-11-02T00:00:00Z --profile nope"
    Then the exit code is 2
    And the output contains "ESH-METHOD"
