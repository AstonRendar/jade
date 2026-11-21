Feature: Preferences
  Preferences survive restarts of the app.

  Scenario: A first start uses the defaults
    Given the app has never been started
    When the preferences are read
    Then the language follows the system
    And the theme is "system"
    And automatic sync runs every 5 minutes

  Scenario: Changed preferences are kept after a restart
    Given the app has never been started
    When the language is set to "zh-CN" with theme "dark" and sync every 30 minutes
    And the app restarts
    And the preferences are read
    Then the language is "zh-CN"
    And the theme is "dark"
    And automatic sync runs every 30 minutes

  Scenario: Recently opened vaults are remembered across restarts
    Given the app has never been started
    When the vault "/vaults/work" is opened
    And the vault "/vaults/home" is opened
    And the app restarts
    And the preferences are read
    Then the last vault is "/vaults/home"
    And the recent vaults are "/vaults/home, /vaults/work"

  Scenario: An unreadable preferences file is reported
    Given the preferences file is corrupt
    When the preferences are read
    Then reading the preferences fails
