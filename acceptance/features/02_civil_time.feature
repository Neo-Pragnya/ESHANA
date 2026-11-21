@cap_civil_time
Feature: Civil time is resolved honestly

  Scenario: A Kolkata evening resolves to UTC with the historical offset
    When I run "eshana positions --date 1992-05-28 --time 20:05:00 --tz Asia/Kolkata"
    Then it succeeds
    And the JSON output at "input.instant_utc" equals "1992-05-28T14:35:00Z"
    And the JSON output at "input.civil_offset_s" equals 19800

  Scenario: A repeated local time is an error unless a fold policy is given
    When I run "eshana positions --date 2024-11-03 --time 01:30 --tz America/New_York"
    Then the exit code is 2
    And the output contains "ESH-TIME-AMBIGUOUS"

  Scenario: The fold policy picks the occurrence explicitly
    When I run "eshana positions --date 2024-11-03 --time 01:30 --tz America/New_York --fold latest"
    Then it succeeds
    And the JSON output at "input.instant_utc" equals "2024-11-03T06:30:00Z"

  Scenario: A skipped local time is a different error
    When I run "eshana positions --date 2024-03-10 --time 02:30 --tz America/New_York --fold latest"
    Then the exit code is 2
    And the output contains "ESH-TIME-NONEXISTENT"

  Scenario: An unknown zone is refused
    When I run "eshana positions --date 2000-01-01 --time 10:00 --tz Mars/Olympus"
    Then the exit code is 2
    And the output contains "ESH-TZ-UNKNOWN"

  Scenario: A local time without any zone is refused rather than assumed
    When I run "eshana positions --date 2000-01-01 --time 10:00"
    Then the exit code is 2
    And the output contains "will not assume one"

  Scenario: A fixed offset works when no zone is known
    When I run "eshana positions --date 2000-01-01 --time 17:30 --offset-minutes 330"
    Then it succeeds
    And the JSON output at "input.instant_utc" equals "2000-01-01T12:00:00Z"

  Scenario: The tz database version is recorded
    When I run "eshana positions --date 1992-05-28 --time 20:05 --tz Asia/Kolkata"
    Then the output matches ""tzdb_version": "20[0-9]{2}[a-z]""
