Feature: Branches
  A device works on one branch of the vault and can switch to another one.

  Background:
    Given a remote vault with "welcome.md" containing "hello"
    And the remote also has the branch "develop" with "draft.md" containing "work"
    And devices "laptop" and "phone" cloned the remote vault

  Scenario: The branches of the remote are listed
    When "laptop" lists its branches
    Then "laptop" is on "main" and knows the branches "develop, main"

  Scenario: Switching branch brings its notes and keeps pending edits on the old branch
    Given "laptop" writes "welcome.md" with "edited before switching"
    When "laptop" switches to the branch "develop"
    Then "laptop" has "draft.md" with "work"
    And "laptop" has "welcome.md" with "hello"
    When "laptop" switches to the branch "main"
    Then "laptop" has "welcome.md" with "edited before switching"
    And "laptop" does not have "draft.md"

  Scenario: Two devices on the same branch synchronize through it
    Given "laptop" switches to the branch "develop"
    And "phone" switches to the branch "develop"
    And "laptop" writes "draft.md" with "work from the laptop"
    When "laptop" synchronizes
    And "phone" synchronizes
    Then "phone" has "draft.md" with "work from the laptop"

  Scenario: Switching to a branch nobody has fails
    When "laptop" switches to the branch "nowhere"
    Then switching fails because the branch does not exist
