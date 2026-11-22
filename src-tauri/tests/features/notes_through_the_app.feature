Feature: Notes and history through the app
  Background:
    Given a folder "notes" with the note "a.md"
    And the app on "laptop" opened the folder "notes"

  Scenario: Organising notes
    When the app on "laptop" creates the note "Ideas" in the root
    And the app on "laptop" creates the folder "projects"
    And the app on "laptop" renames "Ideas.md" to "projects/Ideas.md"
    And the app on "laptop" saves "projects/Ideas.md" with "see [[a]]"
    Then the vault on "laptop" lists "projects/Ideas.md"
    And "projects/Ideas.md" on "laptop" reads "see [[a]]"
    And searching "see" on "laptop" finds "projects/Ideas.md"
    And the backlinks of "a.md" on "laptop" come from "projects/Ideas.md"
    And the link "a" on "laptop" resolves to "a.md"

  Scenario: Following a missing link creates the note
    When the app on "laptop" follows the link "Someday"
    Then the vault on "laptop" lists "Someday.md"

  Scenario: Deleting a note
    When the app on "laptop" deletes "a.md"
    Then the vault on "laptop" does not list "a.md"

  Scenario: Versioning a plain folder
    When the app on "laptop" initialises a repository
    And the app on "laptop" synchronizes
    Then the repository on "laptop" has nothing pending
    And the history of "a.md" on "laptop" has 1 revision
    And the first revision of "a.md" on "laptop" reads "text"

  Scenario: Forgetting a recent vault
    When the app on "laptop" forgets the folder "notes"
    Then "laptop" remembers no vaults
