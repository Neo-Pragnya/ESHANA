@cap_time_and_sky
Feature: Calendars and BCE dates, JPL kernels, rise and set, topocentric positions and house systems

  Scenario: A Julian-calendar date is converted and the conversion is recorded
    When I run "eshana positions --date 1850-03-01 --calendar julian --time 12:00 --offset-minutes 0 --bodies sun"
    Then it succeeds
    And the output contains "julian 1850-03-01 = gregorian 1850-03-13"
    And the JSON output at "input.calendar.name" equals "julian"

  Scenario: The ten days skipped in 1582 are refused
    When I run "eshana positions --date 1582-10-10 --calendar auto --time 12:00 --offset-minutes 0"
    Then the exit code is 2
    And the output contains "1582-10-15"

  Scenario: The kernel catalogue lists the JPL kernels with NAIF's checksums
    When I run "eshana kernel list"
    Then it succeeds
    And the output contains each of:
      """
      de440s
      de441-part1
      3917ee56769db332790c751e2168843d
      """

  Scenario: A file that is not a known kernel does not verify
    Given the file "notes.txt" with content:
      """
      not a kernel
      """
    When I run "eshana kernel verify notes.txt"
    Then the exit code is 2
    And the output contains "unknown"

  Scenario: Rise, transit and set for several local days, with twilight
    When I run "eshana rise-set --date 2026-11-24 --tz Asia/Kolkata --lat 28.6139 --lon 77.209 --days 3 --bodies sun,moon,venus"
    Then it succeeds
    And the JSON output at "days" has at least 3 items
    And the output contains each of:
      """
      rises_and_sets
      astronomical
      """

  Scenario: The midnight sun never sets
    When I run "eshana rise-set --date 2026-06-21 --tz Europe/Oslo --lat 69.6492 --lon 18.9553"
    Then it succeeds
    And the output contains "above_horizon_all_day"

  Scenario: Topocentric positions are labelled and need a place
    When I run "eshana positions --utc 2026-11-24T12:00:00Z --lat -0.18 --lon -78.47 --altitude-m 2850 --topocentric --bodies moon"
    Then it succeeds
    And the JSON output at "calculation.positions_reference" equals "topocentric"

  Scenario: Topocentric without a place is refused
    When I run "eshana positions --utc 2026-11-24T12:00:00Z --topocentric --bodies moon"
    Then the exit code is 2

  Scenario: Every house system draws twelve cusps
    When I run "eshana chart western --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.209 --houses regiomontanus"
    Then it succeeds
    And the JSON output at "calculation.house_system" equals "regiomontanus"
    And the JSON output at "houses.cusps_deg" has at least 12 items
