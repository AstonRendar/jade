Feature: Repository status
  Background:
    Given a remote vault with "welcome.md" containing "hello"
    And device "laptop" cloned the remote vault

  Scenario: Unsynchronized changes are pending
    Given "laptop" writes "ideas.md" with "idea"
    When the status of "laptop" is requested
    Then "laptop" has 1 pending change
    And "laptop" is on branch "main" with a remote

  Scenario: A synchronized vault has nothing pending
    Given "laptop" writes "ideas.md" with "idea"
    And "laptop" synchronizes
    When the status of "laptop" is requested
    Then "laptop" has 0 pending changes
    And "laptop" is 0 commits ahead and 0 behind

  Scenario: A folder that is not a repository
    Given a plain folder on device "desk"
    When the status of "desk" is requested
    Then "desk" is not a repository
