Feature: Local vault without remote
  A vault can be versioned before it is connected to any remote.

  Scenario: Synchronizing a vault without remote only commits
    Given a new local vault on device "desk"
    And "desk" writes "draft.md" with "draft"
    When "desk" synchronizes
    Then the last sync committed the changes
    And the last sync did not push

  Scenario: Connecting a remote publishes the vault
    Given an empty remote vault
    And a new local vault on device "desk"
    And "desk" writes "draft.md" with "draft"
    And "desk" is connected to the remote vault
    When "desk" synchronizes
    And device "laptop" cloned the remote vault
    Then "laptop" has "draft.md" with "draft"

  Scenario: Initialising a folder that is already a repository fails
    Given a new local vault on device "desk"
    When "desk" is initialised again
    Then initialising fails because it is already a repository
