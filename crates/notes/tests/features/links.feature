Feature: Links between notes
  Notes link to each other with Obsidian wikilinks.

  Background:
    Given an empty vault
    And the note "plan.md" contains "# Plan"
    And the note "daily/today.md" contains "Review [[Plan]] today"

  Scenario: Following a link opens the linked note
    When the link "plan" is followed
    Then the followed note is "plan.md"
    And no note was created

  Scenario: Following a link to a missing note creates it
    When the link "Someday" is followed
    Then the followed note is "Someday.md"
    And a note was created
    And "Someday.md" is empty

  Scenario: Following a link with a folder creates the note there
    When the link "projects/Roadmap" is followed
    Then the followed note is "projects/Roadmap.md"

  Scenario: Backlinks list every line linking to a note
    When the backlinks of "plan.md" are requested
    Then the backlinks are:
      | source         | line | text                 |
      | daily/today.md | 1    | Review [[Plan]] today |

  Scenario: Links are resolved in batch
    When the links "Plan, daily/today, Missing" are resolved
    Then "Plan" resolves to "plan.md"
    And "daily/today" resolves to "daily/today.md"
    And "Missing" does not resolve
