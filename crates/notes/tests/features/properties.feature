Feature: Note properties
  Background:
    Given an empty vault

  Scenario: Properties written by Jade are read back unchanged
    When the properties of "body" are set to:
      | key     | value             |
      | tipo    | personaje         |
      | mentor  | [[Bruno]]         |
      | anyo    | 1024              |
      | aliases | Ana, La Torre     |
    Then reading them back gives the same properties and the body "body"

  Scenario: The vault lists each property with its type and values
    Given the note "Ana.md" contains "---\ntipo: personaje\nanyo: '1024'\ntags: [saga]\n---\n"
    And the note "Torre.md" contains "---\ntipo: lugar\n---\n"
    And the file ".obsidian/types.json" contains:
      """
      {"types": {"anyo": "number"}}
      """
    When the properties of the vault are listed
    Then the vault properties are:
      | name | type   | count | values           |
      | anyo | number | 1     | 1024             |
      | tags | tags   | 1     | saga             |
      | tipo | text   | 2     | lugar, personaje |

  Scenario: Changing a property type is stored where Obsidian reads it
    Given the note "Ana.md" contains "---\nanyo: '1024'\n---\n"
    When the type of the property "anyo" is set to "number"
    And the properties of the vault are listed
    Then the vault properties are:
      | name | type   | count | values |
      | anyo | number | 1     | 1024   |
    And the file ".obsidian/types.json" reads:
      """
      {
        "types": {
          "anyo": "number"
        }
      }
      """
