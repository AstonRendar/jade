Feature: Note history
  Background:
    Given a remote vault with "welcome.md" containing "hello"
    And device "laptop" cloned the remote vault

  Scenario: Every synchronized version of a note is listed, newest first
    Given "laptop" writes "welcome.md" with "second"
    And "laptop" synchronizes
    And "laptop" writes "welcome.md" with "third"
    And "laptop" synchronizes
    When the history of "welcome.md" on "laptop" is requested
    Then the history has 3 revisions
    And the oldest revision of "welcome.md" on "laptop" contains "hello"
    And the newest revision of "welcome.md" on "laptop" contains "third"

  Scenario: Notes untouched by a commit are not in its history
    Given "laptop" writes "other.md" with "unrelated"
    And "laptop" synchronizes
    When the history of "welcome.md" on "laptop" is requested
    Then the history has 1 revision
