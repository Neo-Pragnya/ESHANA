@cap_compare
Feature: Method comparison

  Scenario: Two ayanamshas differ by a constant shift that is explained
    When I run "eshana compare vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --a ayanamsha=lahiri --b ayanamsha=fagan_bradley"
    Then it succeeds
    And the JSON output at "kind" equals "method_comparison"
    And the JSON output at "configuration_differences" contains "ayanamsha"
    And the output contains "not software defects"

  Scenario: Mean versus true node shows up as a configuration difference
    When I run "eshana compare western --utc 2024-02-29T18:45:30Z --zodiac tropical --a node_model=mean --b node_model=true"
    Then it succeeds
    And the JSON output at "configuration_differences" contains "node_model"

  Scenario: Tropical versus sidereal changes every longitude
    When I run "eshana compare western --utc 2000-01-01T12:00:00Z --a zodiac=tropical --b zodiac=sidereal --b ayanamsha=lahiri"
    Then it succeeds
    And the JSON output at "configuration_differences" contains "zodiac"
    And the output matches ""sign_changes": [1-9]"
