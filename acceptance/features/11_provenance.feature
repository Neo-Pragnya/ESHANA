@cap_provenance
Feature: Provenance and the three-layer doctrine

  Scenario: A result records engine, provider, time model, rules and assumptions
    When I run "eshana chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487"
    Then it succeeds
    And the output contains "engine_version"
    And the output contains "provider_accuracy"
    And the output contains "tzdb_version"
    And the output contains "ayanamsha-lahiri@0.1"
    And the output contains "UTC is used as UT1"
    And the output contains "assumptions"

  Scenario: The engine never emits an interpretation of its own
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z --lat 17.385 --lon 78.487 --divisions 9 --dasha"
    Then it succeeds
    And the JSON output at "interpretations" equals []

  Scenario: The provider's measured accuracy is stated, not implied
    When I run "eshana positions --utc 2000-01-01T12:00:00Z"
    Then the output contains "Arc-second level"
    And the output contains "benchmarks/golden/accuracy.md"
