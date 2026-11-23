Feature: Query a base
  Background:
    Given an empty vault
    And the note "Characters/Ana.md" contains "---\ntipo: personaje\ntags:\n  - saga\n---\nSee [[Tower]]"
    And the note "Places/Tower.md" contains "---\ntipo: lugar\n---\n#saga"
    And the note "_Workshop/Draft.md" contains "---\ntipo: personaje\n---\n"
    And the file "Index.base" contains:
      """
      filters:
        and:
          - file.ext == "md"
          - '!file.inFolder("_Workshop")'
      properties:
        tipo:
          displayName: Type
      views:
        - type: table
          name: Characters
          filters: tipo == "personaje"
          order:
            - file.name
            - tipo
            - file.tags
        - type: table
          name: Everything
          order:
            - file.basename
            - file.links
          sort:
            - property: file.basename
              direction: DESC
      """

  Scenario: The first view applies the global and view filters
    When the base "Index.base" is queried
    Then the base table has the columns "file.name, Type, file.tags"
    And the base rows are:
      | path              | cells                      |
      | Characters/Ana.md | Ana.md \| personaje \| saga |

  Scenario: Another view is chosen by name and sorted
    When the view "Everything" of the base "Index.base" is queried
    Then the base rows are:
      | path              | cells                     |
      | Places/Tower.md   | Tower \|                  |
      | Characters/Ana.md | Ana \| Tower               |

  Scenario: A malformed base is reported
    Given the file "Broken.base" contains:
      """
      views: [
      """
    When the base "Broken.base" is queried
    Then the operation fails because the base is invalid
