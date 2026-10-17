//! Small JSON string extractor used for Steam API app metadata.
//!
//! Steam responses are only used as optional display metadata, so a minimal
//! string reader keeps the app self-contained without coupling library scans
//! to a second JSON dependency.

/// Returns the first non-empty JSON string value associated with a key.
pub(super) fn json_string_for_key(source: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let mut search_from = 0;
    while let Some(offset) = source.get(search_from..)?.find(&needle) {
        let key_start = search_from + offset;
        let mut index = key_start + needle.len();
        let bytes = source.as_bytes();
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes.get(index) != Some(&b':') {
            search_from = key_start + needle.len();
            continue;
        }
        index += 1;
        while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
            index += 1;
        }
        if bytes.get(index) == Some(&b'"') {
            if let Some(value) = parse_json_string(source, index) {
                if !value.trim().is_empty() {
                    return Some(value);
                }
            }
        }
        search_from = key_start + needle.len();
    }
    None
}

fn parse_json_string(source: &str, quote_index: usize) -> Option<String> {
    let bytes = source.as_bytes();
    if bytes.get(quote_index) != Some(&b'"') {
        return None;
    }
    let mut output = String::new();
    let mut index = quote_index + 1;

    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Some(output),
            b'\\' => {
                index += 1;
                match *bytes.get(index)? {
                    b'"' => output.push('"'),
                    b'\\' => output.push('\\'),
                    b'/' => output.push('/'),
                    b'b' => output.push('\u{0008}'),
                    b'f' => output.push('\u{000C}'),
                    b'n' => output.push('\n'),
                    b'r' => output.push('\r'),
                    b't' => output.push('\t'),
                    b'u' => {
                        let first = read_hex_quad(bytes, index + 1)?;
                        index += 4;
                        let codepoint = if (0xD800..=0xDBFF).contains(&first)
                            && bytes
                                .get(index + 1..index + 3)
                                .is_some_and(|sequence| sequence == b"\\u")
                        {
                            let second = read_hex_quad(bytes, index + 3)?;
                            if !(0xDC00..=0xDFFF).contains(&second) {
                                return None;
                            }
                            index += 6;
                            0x10000 + (((first as u32 - 0xD800) << 10) | (second as u32 - 0xDC00))
                        } else {
                            first as u32
                        };
                        output.push(char::from_u32(codepoint)?);
                    }
                    _ => return None,
                }
            }
            byte if byte < 0x20 => return None,
            _ => {
                let character = source.get(index..)?.chars().next()?;
                output.push(character);
                index += character.len_utf8() - 1;
            }
        }
        index += 1;
    }
    None
}

fn read_hex_quad(bytes: &[u8], start: usize) -> Option<u16> {
    let slice = bytes.get(start..start + 4)?;
    let mut value = 0u16;
    for &byte in slice {
        value = value.checked_mul(16)? + match byte {
            b'0'..=b'9' => (byte - b'0') as u16,
            b'a'..=b'f' => (byte - b'a' + 10) as u16,
            b'A'..=b'F' => (byte - b'A' + 10) as u16,
            _ => return None,
        };
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::{json_string_for_key, parse_json_string};

    #[test]
    fn reads_game_name_from_steam_api_json() {
        let body = r#"{"123":{"success":true,"data":{"name":"A Game: The Sequel"}}}"#;
        assert_eq!(json_string_for_key(body, "name").as_deref(), Some("A Game: The Sequel"));
    }


    #[test]
    fn reads_nested_steamcmd_app_info_name() {
        let body = r#"{"data":{"620":{"common":{"name":"Portal 2","type":"Game"}}},"status":"success"}"#;
        assert_eq!(json_string_for_key(body, "name").as_deref(), Some("Portal 2"));
    }

    #[test]
    fn decodes_json_escapes_and_unicode() {
        assert_eq!(parse_json_string(r#""A \"quoted\" title""#, 0).as_deref(), Some("A \"quoted\" title"));
        assert_eq!(parse_json_string(r#""Pok\u00e9mon""#, 0).as_deref(), Some("Pokémon"));
    }
}
