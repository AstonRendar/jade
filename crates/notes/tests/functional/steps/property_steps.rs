use cucumber::gherkin::Step;
use cucumber::{then, when};
use jade_notes::application::list_properties::{ListProperties, ListPropertiesRequest};
use jade_notes::application::read_properties::{ReadProperties, ReadPropertiesRequest};
use jade_notes::application::set_property_type::{SetPropertyType, SetPropertyTypeRequest};
use jade_notes::application::write_properties::{WriteProperties, WritePropertiesRequest};
use jade_notes::domain::value_object::{Data, PropertyType};
use jade_notes::infrastructure::{FsNoteRepository, YamlDataFormat};
use jade_shared::application::use_case::UseCase;

use super::link_steps::table_rows;
use crate::world::NotesWorld;

#[when(expr = "the properties of {string} are set to:")]
fn the_properties_are_set(world: &mut NotesWorld, body: String, step: &Step) {
    let properties: Vec<(String, Data)> = table_rows(step)
        .into_iter()
        .map(|row| {
            let value = match row[0].as_str() {
                "anyo" => Data::Number(row[1].parse().expect("a number")),
                "aliases" => Data::List(
                    row[1]
                        .split(", ")
                        .map(|item| Data::Text(item.to_string()))
                        .collect(),
                ),
                _ => Data::Text(row[1].clone()),
            };
            (row[0].clone(), value)
        })
        .collect();
    let written = WriteProperties
        .execute(WritePropertiesRequest {
            content: body,
            properties: properties.clone(),
        })
        .expect("properties are written");
    world.properties = properties;
    world.written = written;
}

#[then(expr = "reading them back gives the same properties and the body {string}")]
fn reading_them_back(world: &mut NotesWorld, body: String) {
    let read = ReadProperties::new(YamlDataFormat)
        .execute(ReadPropertiesRequest {
            content: world.written.clone(),
        })
        .expect("properties are read");
    assert_eq!(read.properties, world.properties);
    assert_eq!(&world.written[read.body_start..], body);
}

#[when("the properties of the vault are listed")]
fn the_properties_are_listed(world: &mut NotesWorld) {
    let result =
        ListProperties::new(FsNoteRepository, YamlDataFormat).execute(ListPropertiesRequest {
            vault_root: world.root(),
        });
    world.summaries = world.record(result).unwrap_or_default();
}

#[then("the vault properties are:")]
fn the_vault_properties_are(world: &mut NotesWorld, step: &Step) {
    let actual: Vec<Vec<String>> = world
        .summaries
        .iter()
        .map(|summary| {
            vec![
                summary.name.clone(),
                type_name(summary.kind).to_string(),
                summary.count.to_string(),
                summary.values.join(", "),
            ]
        })
        .collect();
    assert_eq!(actual, table_rows(step));
}

fn type_name(kind: PropertyType) -> &'static str {
    match kind {
        PropertyType::Text => "text",
        PropertyType::List => "list",
        PropertyType::Number => "number",
        PropertyType::Checkbox => "checkbox",
        PropertyType::Date => "date",
        PropertyType::DateTime => "datetime",
        PropertyType::Tags => "tags",
        PropertyType::Aliases => "aliases",
    }
}

#[when(expr = "the type of the property {string} is set to {string}")]
fn the_type_is_set(world: &mut NotesWorld, name: String, kind: String) {
    let result =
        SetPropertyType::new(FsNoteRepository, YamlDataFormat).execute(SetPropertyTypeRequest {
            vault_root: world.root(),
            name,
            kind,
        });
    world.record(result).expect("the type is stored");
}

#[then(expr = "the file {string} reads:")]
fn the_file_reads(world: &mut NotesWorld, path: String, step: &Step) {
    let expected = step
        .docstring
        .as_deref()
        .expect("the step has a text block");
    let actual = std::fs::read_to_string(format!("{}/{path}", world.root())).expect("file is read");
    assert_eq!(actual.trim(), expected.trim());
}
