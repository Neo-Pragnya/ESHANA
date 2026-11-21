@cap_packaging
Feature: The package installs cleanly

  Scenario: eshana starts and lists its commands
    When I run "eshana --help"
    Then it succeeds
    And it finishes within 5 seconds
    And the output contains "positions"
    And the output contains "human-design"

  Scenario: The CLI reports the version being released
    When I run "eshana --version"
    Then it succeeds
    And the output names the package version

  Scenario: The Python wheel imports and reports the same engine version
    When I run "python -c 'import eshana; print(eshana.__version__)'"
    Then it succeeds
    And the output names the package version

  Scenario: doctor shows the environment, providers, rule packs and error codes
    When I run "eshana doctor"
    Then it succeeds
    And the JSON output at "kind" equals "info"
    And the JSON output at "providers" contains "analytic"
    And the JSON output at "providers" contains "spk"
    And the JSON output at "error_codes" has at least 11 items
    And the output contains "Extensible Spatiotemporal Hypersystem for Astroinformatics, Natural Cycles & Associations"
