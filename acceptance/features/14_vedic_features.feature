@cap_vedic_features
Feature: Vedic features from data-driven rule packs

  Scenario: Every feature is computed and names its YAML rule pack
    When I run "eshana chart vedic --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 --features all --dasha --dasha-levels 3 --dasha-systems vimshottari,yogini --reference-utc 2026-11-21T00:00:00Z"
    Then it succeeds
    And the output contains "vedic.ashtakavarga@0.2"
    And the output contains "vedic.yogas@0.2"
    And the output contains "vedic.panchanga@0.2"
    And the output contains "vedic.core@0.2"
    And the output contains "pratyantardashas"
    And the output contains "pranadasha"
    And the output contains "starting_yogini"
    And the output contains "sub_sub_lord"
    And the output contains "karakamsa_sign"
    And the output contains "A12_upapada"
    And the output contains "aspects_grahas"
    And the output contains "windows"
    And the JSON output at "interpretations" equals []

  Scenario: The Sarvashtakavarga always totals 337
    When I run "eshana chart vedic --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 --features ashtakavarga"
    Then it succeeds
    And the output contains "sarvashtakavarga"
    And the output contains ": 337"

  Scenario: Features that need the lagna are omitted, with a reason, when the birth time is unknown
    When I run "eshana chart vedic --date 1990-05-15 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 --features ashtakavarga,jaimini,kp"
    Then it succeeds
    And the output contains "ashtakavarga omitted"
    And the output contains "arudha padas omitted"
    And the output contains "kp omitted"

  Scenario: An unknown feature is refused
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z --features horoscope"
    Then the exit code is 2
    And the output contains "unknown feature"

  Scenario: Features are Vedic only
    When I run "eshana chart western --utc 2000-01-01T12:00:00Z --features kp"
    Then the exit code is 2
    And the output contains "ESH-CONFIG"

  Scenario: Every Parashari varga to D60 is served
    When I run "eshana chart vedic --utc 2000-01-01T12:00:00Z --divisions 2,16,20,24,27,30,40,45,60"
    Then it succeeds
    And the output contains "vedic.moon.d60"
    And the output contains "vedic.sun.d30"

  Scenario: The rule packs are YAML and can be read back
    When I run "eshana rulesets"
    Then it succeeds
    And the output contains "vedic-core.yaml"
    And the output contains "ashtakavarga.yaml"
    And the output contains "matching.yaml"
