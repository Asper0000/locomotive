//! Small Valve KeyValues reader for Steam's local `.acf` and `.vdf` files.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    /// An ordinary quoted or unquoted string.
    Scalar(String),
    /// A nested KeyValues object.
    Object(Vec<(String, Value)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Text(String),
    Open,
    Close,
}

/// Parses KeyValues while tolerating comments and escaped Windows paths.
pub(crate) fn parse(source: &str) -> Vec<(String, Value)> {
    let tokens = tokenize(source);
    let mut index = 0;
    parse_nodes(&tokens, &mut index, false)
}

/// Finds a key without depending on Valve's inconsistent key casing.
pub(crate) fn find<'a>(nodes: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    nodes
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}

/// Returns a scalar value for a key.
pub(crate) fn scalar<'a>(nodes: &'a [(String, Value)], key: &str) -> Option<&'a str> {
    match find(nodes, key)? {
        Value::Scalar(value) => Some(value),
        Value::Object(_) => None,
    }
}

/// Returns a nested object for a key.
pub(crate) fn object<'a>(
    nodes: &'a [(String, Value)],
    key: &str,
) -> Option<&'a [(String, Value)]> {
    match find(nodes, key)? {
        Value::Object(children) => Some(children),
        Value::Scalar(_) => None,
    }
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(character) = chars.next() {
        match character {
            '{' => tokens.push(Token::Open),
            '}' => tokens.push(Token::Close),
            '/' if chars.peek() == Some(&'/') => {
                chars.next();
                for next in chars.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
            }
            character if character.is_whitespace() => {}
            '"' => {
                let mut value = String::new();
                while let Some(next) = chars.next() {
                    match next {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('\\') => value.push('\\'),
                            Some('"') => value.push('"'),
                            Some(other) => {
                                value.push('\\');
                                value.push(other);
                            }
                            None => value.push('\\'),
                        },
                        other => value.push(other),
                    }
                }
                tokens.push(Token::Text(value));
            }
            first => {
                let mut value = String::from(first);
                while let Some(next) = chars.peek().copied() {
                    if next.is_whitespace() || next == '{' || next == '}' {
                        break;
                    }
                    value.push(next);
                    chars.next();
                }
                tokens.push(Token::Text(value));
            }
        }
    }

    tokens
}

fn parse_nodes(
    tokens: &[Token],
    index: &mut usize,
    nested: bool,
) -> Vec<(String, Value)> {
    let mut nodes = Vec::new();

    while *index < tokens.len() {
        if tokens[*index] == Token::Close {
            *index += 1;
            if nested {
                break;
            }
            continue;
        }

        let Token::Text(key) = &tokens[*index] else {
            *index += 1;
            continue;
        };
        let key = key.clone();
        *index += 1;
        if *index >= tokens.len() {
            break;
        }

        match &tokens[*index] {
            Token::Open => {
                *index += 1;
                nodes.push((key, Value::Object(parse_nodes(tokens, index, true))));
            }
            Token::Text(value) => {
                nodes.push((key, Value::Scalar(value.clone())));
                *index += 1;
            }
            Token::Close => {
                *index += 1;
            }
        }
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::{Value, find, object, parse, scalar};

    #[test]
    fn parses_app_state_and_nested_depots() {
        let nodes = parse(
            r#""AppState" { "appid" "480" "name" "Spacewar" "InstalledDepots" { "123" { "manifest" "456" } } }"#,
        );
        let app_state = object(&nodes, "AppState").expect("appstate object should parse");
        assert_eq!(scalar(app_state, "appid"), Some("480"));
        assert_eq!(scalar(app_state, "name"), Some("Spacewar"));
        let depots = object(app_state, "InstalledDepots").expect("depots object should parse");
        assert!(matches!(find(depots, "123"), Some(Value::Object(_))));
    }

    #[test]
    fn decodes_escaped_windows_paths() {
        let nodes = parse(r#""library" { "0" { "path" "C:\\Steam Library" } }"#);
        let library = object(&nodes, "library").expect("library object should parse");
        let first = object(library, "0").expect("first library should parse");
        assert_eq!(scalar(first, "path"), Some("C:\\Steam Library"));
    }
}
