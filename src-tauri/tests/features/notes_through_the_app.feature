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

  Scenario: Querying a base
    Given the folder "notes" has the file "Index.base" with "views:\n  - order: [file.name, status, file.tags, file.links]"
    And the folder "notes" has the file "Ideas.md" with "---\nstatus: open\ntags: [idea]\n---\nsee [[a]]"
    Then the base "Index.base" on "laptop" shows "Ideas.md" with "Ideas.md | open | idea | a"

  Scenario: A malformed base is reported with its code
    Given the folder "notes" has the file "Broken.base" with "views: ["
    When the app on "laptop" queries the base "Broken.base"
    Then the app on "laptop" fails with "invalid_base"

  Scenario: Editing properties as a form
    Given the folder "notes" has the file "Ideas.md" with "---\nstatus: 🌱 open\n---\nbody"
    When the app on "laptop" sets the property "done" of "Ideas.md" to true
    Then "Ideas.md" on "laptop" has the properties "status, done" and the body "body"
    And the vault on "laptop" knows the property "done" as a checkbox

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
