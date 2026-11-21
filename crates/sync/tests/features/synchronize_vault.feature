Feature: Synchronize vault
  Notes written on one device reach every other device through the remote,
  and no change is ever lost, even when two devices edit the same note.

  Background:
    Given a remote vault with "welcome.md" containing "hello"
    And devices "laptop" and "phone" cloned the remote vault

  Scenario: A new note reaches the other device
    Given "laptop" writes "ideas.md" with "first idea"
    When "laptop" synchronizes
    And "phone" synchronizes
    Then "phone" has "ideas.md" with "first idea"

  Scenario: Synchronizing without local changes commits nothing
    When "laptop" synchronizes
    Then the last sync committed nothing
    And the last sync pushed to the remote

  Scenario: Edits to different notes are merged
    Given "laptop" writes "a.md" with "from laptop"
    And "laptop" synchronizes
    And "phone" writes "b.md" with "from phone"
    When "phone" synchronizes
    Then "phone" has "a.md" with "from laptop"
    And "phone" has "b.md" with "from phone"
    And the last sync created no conflict copies
    And the remote head is a merge commit "Merge remote changes"

  Scenario: Both devices edit the same note
    Given "laptop" writes "welcome.md" with "from laptop"
    And "laptop" synchronizes
    And "phone" writes "welcome.md" with "from phone"
    When "phone" synchronizes
    Then "phone" has "welcome.md" with "from phone"
    And "phone" has "welcome (conflict 2026-11-21 2245).md" with "from laptop"
    And the last sync reported the conflict copy "welcome (conflict 2026-11-21 2245).md"
    And the remote history was never rewritten

  Scenario: Both devices see the conflict copy after synchronizing
    Given "laptop" writes "welcome.md" with "from laptop"
    And "laptop" synchronizes
    And "phone" writes "welcome.md" with "from phone"
    And "phone" synchronizes
    When "laptop" synchronizes
    Then "laptop" has "welcome.md" with "from phone"
    And "laptop" has "welcome (conflict 2026-11-21 2245).md" with "from laptop"

  Scenario: A note edited locally and deleted remotely is kept
    Given "laptop" deletes "welcome.md"
    And "laptop" synchronizes
    And "phone" writes "welcome.md" with "edited on phone"
    When "phone" synchronizes
    Then "phone" has "welcome.md" with "edited on phone"

  Scenario: A note deleted locally and edited remotely is restored
    Given "laptop" writes "welcome.md" with "edited on laptop"
    And "laptop" synchronizes
    And "phone" deletes "welcome.md"
    When "phone" synchronizes
    Then "phone" has "welcome.md" with "edited on laptop"

  Scenario: A note deleted on both devices stays deleted
    Given "laptop" deletes "welcome.md"
    And "laptop" synchronizes
    And "phone" deletes "welcome.md"
    When "phone" synchronizes
    Then "phone" does not have "welcome.md"
