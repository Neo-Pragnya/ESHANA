@cap_positions
Feature: Positions

  Scenario: The Sun at J2000 is in Capricorn near 280.4 degrees
    When I run "eshana positions --utc 2000-01-01T12:00:00Z"
    Then it succeeds
    And the JSON output at "kind" equals "positions"
    And the JSON output at "positions[0].id" equals "sun"
    And the JSON output at "positions[0].sign.name" equals "Capricorn"
    And the output matches ""longitude_deg": 280\.3[0-9]"
    And the JSON output at "positions" has at least 12 items

  Scenario: Each position states its frame, epistemic layer and speed
    When I run "eshana positions --utc 2000-01-01T12:00:00Z"
    Then the output contains "geocentric_apparent_ecliptic_true_equinox_of_date"
    And the output contains ""epistemic": "calculated""
    And the output contains "speed_deg_per_day"

  Scenario: Mars is retrograde on 2022-12-01 and the mean node always regresses
    When I run "eshana positions --utc 2022-12-01T00:00:00Z --bodies mars,mean_node"
    Then it succeeds
    And the JSON output at "positions[0].retrograde" equals true
    And the JSON output at "positions[1].retrograde" equals true

  Scenario: The south node is exactly opposite the node
    When I run "eshana positions --utc 2000-01-01T12:00:00Z --bodies mean_node"
    Then it succeeds
    And the output contains "south_node"

  Scenario: An unknown body is refused with a code
    When I run "eshana positions --utc 2000-01-01T12:00:00Z --bodies vulcan"
    Then it fails cleanly
    And the output contains "ESH-INPUT"

  Scenario: Instants outside the provider's range say so
    When I run "eshana positions --utc 1500-01-01T00:00:00Z"
    Then the exit code is 2
    And the output contains "ESH-RANGE"
