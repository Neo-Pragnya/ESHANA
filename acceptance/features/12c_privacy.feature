@cap_privacy
Feature: Computation is local and leaves no trace

  Scenario: A chart writes nothing to the project or HOME
    Given a fresh project
    When I run "eshana chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --dasha > /dev/null && test -z "$(git status --porcelain)" && test -z "$(find $HOME -type f 2>/dev/null)" && echo clean"
    Then it succeeds
    And the output contains "clean"

  Scenario: Birth data is not echoed to stderr
    When I run "eshana chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 2>err.txt >/dev/null; test ! -s err.txt && echo quiet"
    Then it succeeds
    And the output contains "quiet"
