@cap_numerology
Feature: Numerology

  Scenario: A worked Pythagorean example
    When I run "eshana numerology --name John"
    Then it succeeds
    And the JSON output at "kind" equals "numerology_analysis"
    And the JSON output at "derived_attributes[0].value.expression.compound" equals 20
    And the JSON output at "derived_attributes[0].value.expression.reduced" equals 2
    And the output contains "numerology.pythagorean@0.1"

  Scenario: Chaldean is a separate schema and is never merged
    When I run "eshana numerology --name Elizabeth --school chaldean"
    Then it succeeds
    And the output contains "numerology.chaldean@0.1"
    And the output does not contain "numerology.pythagorean@0.1"

  Scenario: Diacritics are normalised and non-Latin characters are reported
    When I run "eshana numerology --name 'José Ram राम'"
    Then it succeeds
    And the output contains "JOSERAM"
    And the output contains "ignored_characters"

  Scenario: Life path with the master-number policy visible
    When I run "eshana numerology --dob 2000-04-05"
    Then it succeeds
    And the output contains ""life_path": 11"
    When I run "eshana numerology --dob 2000-04-05 --no-master"
    Then it succeeds
    And the output contains ""life_path": 2"

  Scenario: Impossible dates and empty requests are refused
    When I run "eshana numerology --dob 2023-02-29"
    Then the exit code is 2
    And the output contains "ESH-INPUT"
    When I run "eshana numerology"
    Then the exit code is 2

  Scenario: A name with no Latin letters is refused
    When I run "eshana numerology --name राम"
    Then the exit code is 2
    And the output contains "ESH-RULES"
