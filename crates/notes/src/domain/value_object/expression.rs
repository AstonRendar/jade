#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Null,
    Boolean(bool),
    Number(f64),
    Text(String),
    List(Vec<Expression>),
    Identifier(String),
    Member(Box<Expression>, String),
    Index(Box<Expression>, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    Not(Box<Expression>),
    Negate(Box<Expression>),
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid expression `{0}`")]
pub struct InvalidExpressionError(pub String);

impl Expression {
    pub fn parse(source: &str) -> Result<Self, InvalidExpressionError> {
        let invalid = || InvalidExpressionError(source.to_string());
        let tokens = tokenize(source).ok_or_else(invalid)?;
        let mut parser = Parser {
            tokens,
            position: 0,
        };
        let expression = parser.expression().ok_or_else(invalid)?;
        if parser.position == parser.tokens.len() {
            Ok(expression)
        } else {
            Err(invalid())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Text(String),
    Identifier(String),
    Symbol(&'static str),
}

const SYMBOLS: [&str; 19] = [
    "==", "!=", ">=", "<=", "&&", "||", ">", "<", "!", "+", "-", "*", "/", "%", "(", ")", "[", "]",
    ",",
];

fn tokenize(source: &str) -> Option<Vec<Token>> {
    let characters: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if character.is_whitespace() {
            index += 1;
        } else if character == '"' || character == '\'' {
            let (text, next) = quoted(&characters, index)?;
            tokens.push(Token::Text(text));
            index = next;
        } else if character.is_ascii_digit() {
            let mut end = scan(&characters, index, |c| c.is_ascii_digit());
            if characters.get(end) == Some(&'.')
                && characters.get(end + 1).is_some_and(char::is_ascii_digit)
            {
                end = scan(&characters, end + 1, |c| c.is_ascii_digit());
            }
            let number: String = characters[index..end].iter().collect();
            tokens.push(Token::Number(number.parse().ok()?));
            index = end;
        } else if character.is_alphabetic() || character == '_' {
            let end = scan(&characters, index, |c| c.is_alphanumeric() || c == '_');
            tokens.push(Token::Identifier(characters[index..end].iter().collect()));
            index = end;
        } else if character == '.' {
            tokens.push(Token::Symbol("."));
            index += 1;
        } else {
            let rest: String = characters[index..characters.len().min(index + 2)]
                .iter()
                .collect();
            let symbol = SYMBOLS.iter().find(|symbol| rest.starts_with(**symbol))?;
            tokens.push(Token::Symbol(symbol));
            index += symbol.chars().count();
        }
    }
    Some(tokens)
}

fn scan(characters: &[char], start: usize, accepts: impl Fn(char) -> bool) -> usize {
    characters[start..]
        .iter()
        .position(|&c| !accepts(c))
        .map_or(characters.len(), |offset| start + offset)
}

fn quoted(characters: &[char], start: usize) -> Option<(String, usize)> {
    let quote = characters[start];
    let mut text = String::new();
    let mut index = start + 1;
    while index < characters.len() {
        match characters[index] {
            '\\' => {
                text.push(*characters.get(index + 1)?);
                index += 2;
            }
            character if character == quote => return Some((text, index + 1)),
            character => {
                text.push(character);
                index += 1;
            }
        }
    }
    None
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

const LEVELS: [&[(&str, BinaryOperator)]; 6] = [
    &[("||", BinaryOperator::Or)],
    &[("&&", BinaryOperator::And)],
    &[
        ("==", BinaryOperator::Equal),
        ("!=", BinaryOperator::NotEqual),
    ],
    &[
        ("<=", BinaryOperator::LessOrEqual),
        (">=", BinaryOperator::GreaterOrEqual),
        ("<", BinaryOperator::Less),
        (">", BinaryOperator::Greater),
    ],
    &[("+", BinaryOperator::Add), ("-", BinaryOperator::Subtract)],
    &[
        ("*", BinaryOperator::Multiply),
        ("/", BinaryOperator::Divide),
        ("%", BinaryOperator::Remainder),
    ],
];

impl Parser {
    fn expression(&mut self) -> Option<Expression> {
        self.binary(0)
    }

    fn binary(&mut self, level: usize) -> Option<Expression> {
        let Some(operators) = LEVELS.get(level) else {
            return self.unary();
        };
        let mut left = self.binary(level + 1)?;
        while let Some(operator) = operators
            .iter()
            .find(|(symbol, _)| self.peek_symbol(symbol))
            .map(|(_, operator)| *operator)
        {
            self.position += 1;
            let right = self.binary(level + 1)?;
            left = Expression::Binary(operator, Box::new(left), Box::new(right));
        }
        Some(left)
    }

    fn unary(&mut self) -> Option<Expression> {
        if self.eat_symbol("!") {
            return Some(Expression::Not(Box::new(self.unary()?)));
        }
        if self.eat_symbol("-") {
            return Some(Expression::Negate(Box::new(self.unary()?)));
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Option<Expression> {
        let mut expression = self.primary()?;
        loop {
            if self.eat_symbol(".") {
                let Some(Token::Identifier(name)) = self.next() else {
                    return None;
                };
                expression = Expression::Member(Box::new(expression), name);
            } else if self.eat_symbol("[") {
                let index = self.expression()?;
                self.expect_symbol("]")?;
                expression = Expression::Index(Box::new(expression), Box::new(index));
            } else if self.eat_symbol("(") {
                let arguments = self.list_until(")")?;
                expression = Expression::Call(Box::new(expression), arguments);
            } else {
                return Some(expression);
            }
        }
    }

    fn primary(&mut self) -> Option<Expression> {
        match self.next()? {
            Token::Number(number) => Some(Expression::Number(number)),
            Token::Text(text) => Some(Expression::Text(text)),
            Token::Identifier(name) => Some(match name.as_str() {
                "true" => Expression::Boolean(true),
                "false" => Expression::Boolean(false),
                "null" => Expression::Null,
                _ => Expression::Identifier(name),
            }),
            Token::Symbol("(") => {
                let inner = self.expression()?;
                self.expect_symbol(")")?;
                Some(inner)
            }
            Token::Symbol("[") => Some(Expression::List(self.list_until("]")?)),
            Token::Symbol(_) => None,
        }
    }

    fn list_until(&mut self, closing: &str) -> Option<Vec<Expression>> {
        let mut items = Vec::new();
        if self.eat_symbol(closing) {
            return Some(items);
        }
        loop {
            items.push(self.expression()?);
            if self.eat_symbol(closing) {
                return Some(items);
            }
            self.expect_symbol(",")?;
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position += 1;
        token
    }

    fn peek_symbol(&self, symbol: &str) -> bool {
        matches!(self.tokens.get(self.position), Some(Token::Symbol(found)) if *found == symbol)
    }

    fn eat_symbol(&mut self, symbol: &str) -> bool {
        let found = self.peek_symbol(symbol);
        if found {
            self.position += 1;
        }
        found
    }

    fn expect_symbol(&mut self, symbol: &str) -> Option<()> {
        self.eat_symbol(symbol).then_some(())
    }
}
