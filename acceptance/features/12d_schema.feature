@cap_schema
Feature: Results validate against the published schema

  Scenario: The schema is available from the CLI and is valid JSON
    When I run "eshana schema"
    Then it succeeds
    And the JSON output at "title" equals "ESHANA result envelope"
