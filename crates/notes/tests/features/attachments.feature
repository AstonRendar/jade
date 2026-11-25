Feature: Attachments and embedded files
  Background:
    Given an empty vault
    And the file ".obsidian/app.json" contains:
      """
      {"attachmentFolderPath": "Recursos"}
      """

  Scenario: A pasted image is saved in the attachment folder without overwriting
    Given the note "Lore/Torre.md" contains "una torre"
    When the image "Pasted image.png" is attached to "Lore/Torre.md"
    And the image "Pasted image.png" is attached to "Lore/Torre.md"
    Then the attachment was saved as "Recursos/Pasted image 1.png" and linked as "Pasted image 1.png"
    And "Pasted image.png" resolves to the file "Recursos/Pasted image.png"
