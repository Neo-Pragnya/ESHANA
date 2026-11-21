@cap_config_discipline
Feature: Configuration is never silently chosen or merged

  Scenario: A sidereal zodiac needs an explicit ayanamsha
    When I run "eshana positions --utc 2000-01-01T12:00:00Z --zodiac sidereal"
    Then the exit code is 2
    And the output contains "ESH-CONFIG"
    And the output contains "never picks"

  Scenario: A tropical zodiac cannot carry an ayanamsha
    When I run "eshana positions --utc 2000-01-01T12:00:00Z --zodiac tropical --ayanamsha lahiri"
    Then the exit code is 2
    And the output contains "ESH-CONFIG"

  Scenario: The Vedic command defaults to Lahiri but records that it did
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z"
    Then it succeeds
    And the JSON output at "calculation.ayanamsha" equals "lahiri"
    And the output contains "defaulted to lahiri"

  Scenario: Tropical and sidereal longitudes are both reported and differ by the ayanamsha
    When I run "eshana positions --utc 2000-01-01T12:00:00Z --zodiac sidereal --ayanamsha lahiri --bodies sun"
    Then it succeeds
    And the output matches ""longitude_deg": 256\.51"
    And the output matches ""tropical_longitude_deg": 280\.3"
    And the output matches ""ayanamsha_deg": 23\.85"

  Scenario: Every named ayanamsha is accepted and an unknown one is refused
    When I run "for a in lahiri raman krishnamurti fagan_bradley; do eshana positions --utc 2000-01-01T12:00:00Z --zodiac sidereal --ayanamsha $a --bodies sun >/dev/null || exit 1; done"
    Then it succeeds
    When I run "eshana positions --utc 2000-01-01T12:00:00Z --zodiac sidereal --ayanamsha galactic"
    Then the exit code is 2
    And the output contains "ESH-INPUT"

  Scenario: Mean and true node are distinct and neither is substituted
    When I run "eshana positions --utc 2024-02-29T18:45:30Z --node true --bodies true_node"
    Then it succeeds
    And the output contains "true_node"
    And the output does not contain "mean_node"
