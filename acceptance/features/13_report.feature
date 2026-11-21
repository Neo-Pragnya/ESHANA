@cap_report
Feature: Self-contained HTML chart reports

  Background:
    Given a fresh project

  Scenario: A complete report is one offline file with every chart and the raw envelopes
    When I run "eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 --name Tester -o chart.html"
    Then it succeeds
    And the file "chart.html" exists
    And the file "chart.html" contains "Panchang chakra"
    And the file "chart.html" contains "South and North Indian"
    And the file "chart.html" contains "vedic.moon.d9"
    And the file "chart.html" contains "vedic.moon.d60"
    And the file "chart.html" contains "Vimśottarī daśā"
    And the file "chart.html" contains "vedic.dasha.yogini"
    And the file "chart.html" contains "vedic.ashtakavarga"
    And the file "chart.html" contains "vedic.kp"
    And the file "chart.html" contains "vedic.yogas"
    And the file "chart.html" contains "Western chart (tropical)"
    And the file "chart.html" contains "Human Design"
    And the file "chart.html" contains "Numerology"
    And the file "chart.html" contains "Provenance"
    And the file "chart.html" contains "eshana-data"
    And the file "chart.html" does not contain "<link"
    And the file "chart.html" does not contain "fetch("
    And the file "chart.html" does not contain "XMLHttpRequest"

  Scenario: An unknown birth time omits what it cannot know and says why
    When I run "eshana-report --date 1990-05-15 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 -o chart.html"
    Then it succeeds
    And the file "chart.html" contains "Not computed"
    And the file "chart.html" contains "ESH-TIME-UNCERTAIN"
    And the file "chart.html" contains "lagna-based values are omitted"

  Scenario: Saved CLI envelopes render without recomputing
    Given I have run "eshana chart vedic --date 1996-12-07 --time 10:34 --tz Asia/Kolkata --lat 13.0878 --lon 80.2785 --divisions 9 --dasha --compact > vedic.json"
    When I run "eshana-report --from-json vedic.json --sections summary,chakra,dasha -o out.html"
    Then it succeeds
    And the file "out.html" contains "Vimśottarī daśā"
    And the file "out.html" does not contain "Western chart (tropical)"

  Scenario: A partner adds the compatibility section
    When I run "eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.61 --lon 77.21 --partner 1992-08-20,09:15,Asia/Kolkata,19.08,72.88 --reference 2026-11-21T00:00:00Z --transit-days 0 -o pair.html"
    Then it succeeds
    And the file "pair.html" contains "ashtakoota"
    And the file "pair.html" contains "Compatibility (Ashtakoota)"

  Scenario: A bad option is refused cleanly
    When I run "eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --sections horoscope -o x.html"
    Then it fails cleanly
    And the output contains "unknown sections"
    And the file "x.html" does not exist

  Scenario: A skipped local time is refused for the whole report
    When I run "eshana-report --date 2024-03-10 --time 02:30 --tz America/New_York --systems vedic -o x.html"
    Then it succeeds
    And the file "x.html" contains "ESH-TIME-NONEXISTENT"
