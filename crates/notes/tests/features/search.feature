Feature: Search notes
  Background:
    Given an empty vault
    And the note "jade.md" contains "Notes about JADE\nnothing else"
    And the note "other.md" contains "unrelated"

  Scenario: Search finds titles and lines ignoring case
    When the vault is searched for "jade"
    Then the search hits are:
      | path    | line | text             |
      | jade.md | 0    | jade             |
      | jade.md | 1    | Notes about JADE |

  Scenario: Blank searches are rejected
    When the vault is searched for "   "
    Then the operation fails because the query is empty
