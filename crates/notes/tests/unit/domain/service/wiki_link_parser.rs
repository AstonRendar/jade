use jade_notes::domain::service::{LocatedLink, WikiLinkParser};
use jade_notes::domain::value_object::WikiLink;

#[test]
fn parses_a_plain_link() {
    // Arrange
    let content = "See [[Ideas]] later.";

    // Act
    let links = WikiLinkParser.parse(content);

    // Assert
    assert_eq!(links, vec![located(1, link("Ideas", None, None, false))]);
}

#[test]
fn parses_aliases_headings_and_embeds() {
    // Arrange
    let content = "[[Plan|the plan]] [[Plan#Goals]] [[Plan#Goals|goals]] ![[diagram.png]]";

    // Act
    let links = WikiLinkParser.parse(content);

    // Assert
    assert_eq!(
        links,
        vec![
            located(1, link("Plan", None, Some("the plan"), false)),
            located(1, link("Plan", Some("Goals"), None, false)),
            located(1, link("Plan", Some("Goals"), Some("goals"), false)),
            located(1, link("diagram.png", None, None, true)),
        ]
    );
}

#[test]
fn reports_the_line_of_each_link() {
    // Arrange
    let content = "first\n[[A]]\nthird [[B]]";

    // Act
    let lines: Vec<usize> = WikiLinkParser
        .parse(content)
        .iter()
        .map(|l| l.line)
        .collect();

    // Assert
    assert_eq!(lines, vec![2, 3]);
}

#[test]
fn trims_spaces_inside_the_brackets() {
    // Arrange
    let content = "[[  Plan  #  Goals  |  goals  ]]";

    // Act
    let links = WikiLinkParser.parse(content);

    // Assert
    assert_eq!(
        links,
        vec![located(
            1,
            link("Plan", Some("Goals"), Some("goals"), false)
        )]
    );
}

#[test]
fn ignores_links_inside_code() {
    // Arrange
    let content = "```\n[[InFence]]\n```\n`[[InlineCode]]` and [[Real]]\n~~~md\n[[Tilde]]\n~~~";

    // Act
    let targets: Vec<String> = WikiLinkParser
        .parse(content)
        .into_iter()
        .map(|l| l.link.target().to_string())
        .collect();

    // Assert
    assert_eq!(targets, vec!["Real"]);
}

#[test]
fn ignores_empty_or_unclosed_links() {
    // Arrange
    let content = "[[]] [[   ]] [[#Heading]] [[Unclosed and [[Closed]]";

    // Act
    let targets: Vec<String> = WikiLinkParser
        .parse(content)
        .into_iter()
        .map(|l| l.link.target().to_string())
        .collect();

    // Assert
    assert_eq!(targets, vec!["Closed"]);
}

fn link(target: &str, heading: Option<&str>, alias: Option<&str>, embed: bool) -> WikiLink {
    WikiLink::new(target, heading, alias, embed).expect("valid link")
}

fn located(line: usize, link: WikiLink) -> LocatedLink {
    LocatedLink { line, link }
}
