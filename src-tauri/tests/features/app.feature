Feature: Application flows across contexts
  The app opens vaults, remembers them, clones remotes and synchronizes,
  reporting every result and error in the shape the interface expects.

  Scenario: Opening a vault remembers it
    Given a folder "notes" with the note "a.md"
    When the app on "laptop" opens the folder "notes"
    Then the open vault on "laptop" is named "notes"
    And the most recent vault on "laptop" is the folder "notes"
    And the vault on "laptop" lists "a.md"

  Scenario: Working with notes requires an open vault
    When the app on "laptop" lists the vault entries
    Then the app on "laptop" fails with "vault_not_open"

  Scenario: Cloning a remote opens the clone
    Given a remote vault "journal" with "welcome.md" containing "hello"
    When the app on "laptop" clones the remote vault
    Then the open vault on "laptop" is named "journal"
    And the vault on "laptop" lists "welcome.md"

  Scenario: Cloning into an existing folder is refused
    Given a remote vault "journal" with "welcome.md" containing "hello"
    And the app on "laptop" cloned the remote vault
    When the app on "laptop" clones the remote vault
    Then the app on "laptop" fails with "already_exists"

  Scenario: Cloning an unsupported address fails with its code
    When the app on "laptop" clones "ftp://git.example.com/vault.git"
    Then the app on "laptop" fails with "invalid_url"

  Scenario: Conflicts reach the interface as conflict copies
    Given a remote vault "journal" with "welcome.md" containing "hello"
    And the app on "laptop" cloned the remote vault
    And the app on "phone" cloned the remote vault
    And the app on "laptop" saves "welcome.md" with "from laptop"
    And the app on "laptop" synchronizes
    And the app on "phone" saves "welcome.md" with "from phone"
    When the app on "phone" synchronizes
    Then the last sync on "phone" reported the conflict copy "welcome (conflict 2026-11-21 2245).md" of "welcome.md"
    And the repository on "phone" has nothing pending

  Scenario: Preferences are shown as the interface expects
    When the app on "laptop" sets the language "es", theme "dark" and sync every 10 minutes
    Then the preferences on "laptop" show language "es", theme "dark" and sync every 10 minutes

  Scenario: A token is stored without ever being shown again
    When the app on "laptop" stores the token "token-123"
    Then the app on "laptop" has a token

  Scenario: Creating a vault opens and remembers it
    When the app on "phone" creates the vault "Journal" in the folder "vaults"
    Then the open vault on "phone" is named "Journal"
    And the most recent vault on "phone" is the folder "vaults/Journal"

  Scenario: Deleting a cloned vault frees its folder and forgets it
    Given a remote vault "journal" with "welcome.md" containing "hello"
    And the app on "phone" cloned the remote vault
    When the app on "phone" deletes its vault "journal"
    Then the vault "journal" is gone from "phone"
    And the app on "phone" remembers no vaults
    And listing the entries on "phone" fails with "vault_not_open"

  Scenario: Only vaults kept by the app can be deleted
    Given a folder "notes" with the note "a.md"
    When the app on "laptop" deletes the folder "notes"
    Then the app on "laptop" fails with "invalid_path"
    And the folder "notes" still exists

