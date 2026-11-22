Feature: Manage notes and folders
  The vault is a plain folder of Markdown files that can be organised freely.

  Background:
    Given an empty vault

  Scenario: Opening a vault names it after its folder
    When the vault is opened
    Then the vault is named after its folder

  Scenario: Opening a folder that does not exist fails
    When a missing folder is opened as a vault
    Then the vault cannot be found

  Scenario: Creating a note at the root
    When a note titled "Ideas" is created
    Then the vault lists the note "Ideas.md"
    And "Ideas.md" is empty

  Scenario: Creating a note inside a folder
    Given the folder "projects" exists
    When a note titled "Plan" is created in "projects"
    Then the vault lists the note "projects/Plan.md"

  Scenario: Creating a note that already exists fails
    Given the note "Ideas.md" contains "original"
    When a note titled "Ideas" is created
    Then the operation fails because "Ideas.md" already exists
    And "Ideas.md" contains "original"

  Scenario: Saving and reading a note
    Given the note "Ideas.md" contains "first"
    When "Ideas.md" is saved with "second"
    Then "Ideas.md" contains "second"

  Scenario: Renaming a note
    Given the note "Ideas.md" contains "text"
    When "Ideas.md" is renamed to "Thoughts.md"
    Then the vault lists the note "Thoughts.md"
    And the vault does not list "Ideas.md"

  Scenario: Moving a folder keeps its notes
    Given the note "projects/Plan.md" contains "plan"
    When "projects" is renamed to "archive/projects"
    Then "archive/projects/Plan.md" contains "plan"

  Scenario: A folder cannot be moved inside itself
    Given the folder "projects" exists
    When "projects" is renamed to "projects/inner"
    Then the operation fails because the move is not possible

  Scenario: Renaming onto an existing entry fails
    Given the note "a.md" contains "a"
    And the note "b.md" contains "b"
    When "a.md" is renamed to "b.md"
    Then the operation fails because "b.md" already exists

  Scenario: Deleting a folder deletes its notes
    Given the note "projects/Plan.md" contains "plan"
    When "projects" is deleted
    Then the vault does not list "projects/Plan.md"
    And the vault does not list "projects"

  Scenario: Hidden folders are not listed
    Given the note ".obsidian/workspace.md" contains "{}"
    And the note "visible.md" contains "text"
    Then the vault lists exactly "visible.md"

  Scenario: Other files are listed but are not notes
    Given the file "diagram.png" exists
    Then the vault lists the file "diagram.png"

  Scenario: Renaming a note updates the links to it
    Given the note "plan.md" contains "# Plan"
    And the note "daily.md" contains "Review [[Plan#Goals|the plan]] and ![[plan]]"
    When "plan.md" is renamed to "Roadmap.md"
    Then "daily.md" contains "Review [[Roadmap#Goals|the plan]] and ![[Roadmap]]"

  Scenario: Moving a folder updates the links to its notes
    Given the note "projects/plan.md" contains "# Plan"
    And the note "daily.md" contains "See [[projects/plan]]"
    And the note "projects/notes.md" contains "Back to [[plan]]"
    When "projects" is renamed to "archive"
    Then "daily.md" contains "See [[archive/plan]]"
    And "archive/notes.md" contains "Back to [[plan]]"

  Scenario: Creating a new vault
    When a vault named "Journal" is created next to the current one
    Then the new vault is named "Journal"
    And creating it again fails because it already exists
