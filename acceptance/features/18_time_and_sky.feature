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

  Scenario: A solar return each year, with a chart at each
    When I run "eshana returns --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.209 --start 2025-01-01T00:00:00Z --end 2028-01-01T00:00:00Z"
    Then it succeeds
    And the JSON output at "returns" has at least 3 items
    And the output contains "cusps_deg"

  Scenario: Eclipses of a year, with their types
    When I run "eshana eclipses --start 2026-01-01T00:00:00Z --end 2027-01-01T00:00:00Z"
    Then it succeeds
    And the output contains each of:
      """
      annular
      total
      partial
      """

  Scenario: Gochara counts houses from the natal Moon, with retrograde and combustion intervals
    When I run "eshana gochara --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.209 --ayanamsha lahiri --start 2026-01-01T00:00:00Z --end 2027-01-01T00:00:00Z --transit-bodies mercury,saturn"
    Then it succeeds
    And the output contains each of:
      """
      house_from_moon
      retrograde_intervals
      combustion_intervals
      """

  Scenario: An asteroid without a kernel that carries it is refused
    When I run "eshana positions --utc 2026-11-24T12:00:00Z --bodies ceres"
    Then the exit code is 2

  Scenario: Synastry with a composite and a Davison chart
    When I run "eshana synastry --a 1990-05-15,14:30,Asia/Kolkata,28.6139,77.209 --b 1992-08-20,09:15,Asia/Kolkata,19.076,72.8777"
    Then it succeeds
    And the output contains each of:
      """
      inter_aspects
      house_overlays
      composite
      davison
      """

  Scenario: Western essential dignities
    When I run "eshana chart western --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.209 --features dignities"
    Then it succeeds
    And the output contains each of:
      """
      western.dignities
      peregrine
      exaltation
      """

  Scenario: Gematria reads a name in its own script and refuses a Latin spelling
    When I run "eshana numerology --name שלום --school hebrew_gematria"
    Then it succeeds
    And the JSON output at "gematria.total" equals 376
    When I run "eshana numerology --name Shalom --school hebrew_gematria"
    Then the exit code is 2

  Scenario: A user rule pack cannot claim the built-in namespace
    Given the file "pack.yaml" with content:
      """
      id: vedic.mine
      version: "1"
      yogas: []
      doshas: []
      """
    When I run "eshana chart vedic --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6 --lon 77.2 --rule-pack pack.yaml"
    Then the exit code is 2
    And the output contains "namespace"
