use jade_notes::domain::value_object::{BinaryOperator, Expression};

fn identifier(name: &str) -> Box<Expression> {
    Box::new(Expression::Identifier(name.to_string()))
}

fn text(value: &str) -> Box<Expression> {
    Box::new(Expression::Text(value.to_string()))
}

#[test]
fn parses_a_comparison_of_a_member_with_a_string() {
    // Arrange
    let source = r#"file.ext == "md""#;

    // Act
    let expression = Expression::parse(source);

    // Assert
    assert_eq!(
        expression,
        Ok(Expression::Binary(
            BinaryOperator::Equal,
            Box::new(Expression::Member(identifier("file"), "ext".to_string())),
            text("md"),
        ))
    );
}

#[test]
fn parses_a_negated_method_call() {
    // Arrange
    let source = r#"!file.inFolder('_Workshop')"#;

    // Act
    let expression = Expression::parse(source);

    // Assert
    assert_eq!(
        expression,
        Ok(Expression::Not(Box::new(Expression::Call(
            Box::new(Expression::Member(
                identifier("file"),
                "inFolder".to_string()
            )),
            vec![Expression::Text("_Workshop".to_string())],
        ))))
    );
}

#[test]
fn gives_multiplication_precedence_over_addition_and_and_over_or() {
    // Arrange
    let source = "a || b && 1 + 2 * 3 > 4";

    // Act
    let expression = Expression::parse(source).expect("valid expression");

    // Assert
    let product = Expression::Binary(
        BinaryOperator::Multiply,
        Box::new(Expression::Number(2.0)),
        Box::new(Expression::Number(3.0)),
    );
    let sum = Expression::Binary(
        BinaryOperator::Add,
        Box::new(Expression::Number(1.0)),
        Box::new(product),
    );
    let comparison = Expression::Binary(
        BinaryOperator::Greater,
        Box::new(sum),
        Box::new(Expression::Number(4.0)),
    );
    let conjunction =
        Expression::Binary(BinaryOperator::And, identifier("b"), Box::new(comparison));
    assert_eq!(
        expression,
        Expression::Binary(BinaryOperator::Or, identifier("a"), Box::new(conjunction))
    );
}

#[test]
fn parses_literals_lists_indexes_and_groups() {
    // Arrange
    let source = r#"[true, false, null, -1.5, "a \"b\""][0] != (x)"#;

    // Act
    let expression = Expression::parse(source).expect("valid expression");

    // Assert
    let list = Expression::List(vec![
        Expression::Boolean(true),
        Expression::Boolean(false),
        Expression::Null,
        Expression::Negate(Box::new(Expression::Number(1.5))),
        Expression::Text("a \"b\"".to_string()),
    ]);
    assert_eq!(
        expression,
        Expression::Binary(
            BinaryOperator::NotEqual,
            Box::new(Expression::Index(
                Box::new(list),
                Box::new(Expression::Number(0.0))
            )),
            identifier("x"),
        )
    );
}

#[test]
fn parses_calls_without_arguments_and_unicode_identifiers() {
    // Arrange
    let source = "año.isEmpty() <= 3 % 2 - 1 / 1";

    // Act
    let expression = Expression::parse(source);

    // Assert
    assert!(expression.is_ok());
}

#[test]
fn rejects_malformed_expressions() {
    // Arrange
    let sources = [
        "", "a ==", "(a", "\"open", "a $ b", "a.", "f(a b)", "a b", "[1, 2",
    ];

    // Act
    let results: Vec<_> = sources
        .iter()
        .map(|source| Expression::parse(source))
        .collect();

    // Assert
    assert!(results.iter().all(Result::is_err), "{results:?}");
}
