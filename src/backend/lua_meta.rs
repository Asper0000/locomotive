//! Minimal reader for BetterSteamTools-style Lua depot configuration files.
//!
//! Extracts the two facts the library screen needs to group a `.lua` file
//! with the `.manifest` files that belong to the same game:
//!
//!   - the app id, from the single-argument form:   addappid(APP_ID)
//!   - the depot ids, from the three-argument form: addappid(DEPOT_ID, 0, "key")
//!
//! No Lua is executed. The reader is intentionally permissive: it scans
//! for the function name, reads the numeric first argument, and skips
//! anything else. That is enough for grouping and it cannot fail on real
//! files, however they are formatted.

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LuaMeta {
    /// Values of every `addappid` call. Includes the base game and all DLCs.
    pub depots: Vec<String>,
}

/// Reads and parses a Lua file. Returns None if the file cannot be read.
pub(crate) fn read(path: &Path) -> Option<LuaMeta> {
    let text = fs::read_to_string(path).ok()?;
    Some(parse(&text))
}

/// Scans `text` for addappid calls and returns the app id and depot list.
pub(crate) fn parse(text: &str) -> LuaMeta {
    let mut meta = LuaMeta::default();
    let bytes = text.as_bytes();
    let needle = b"addappid";
    let mut search_from = 0usize;

    while search_from < bytes.len() {
        let rel = match text[search_from..].find("addappid") {
            Some(offset) => offset,
            None => break,
        };
        let start = search_from + rel;
        let end = start + needle.len();

        // Left word boundary: the char before must not be alphanumeric or _.
        if start > 0 {
            let prev = bytes[start - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' {
                search_from = end;
                continue;
            }
        }

        // Skip whitespace, then require an opening parenthesis.
        let mut open = end;
        while open < bytes.len() && (bytes[open] == b' ' || bytes[open] == b'\t') {
            open += 1;
        }
        if open >= bytes.len() || bytes[open] != b'(' {
            search_from = end;
            continue;
        }

        // Everything up to the next closing parenthesis is the argument list.
        let close_rel = match text[open..].find(')') {
            Some(offset) => offset,
            None => break,
        };
        let args_str = &text[open + 1..open + close_rel];
        search_from = open + close_rel + 1;

        let args: Vec<&str> = args_str.split(',').map(str::trim).collect();
        match args.len() {
            1 | 3 => {
                if is_digits(args[0]) {
                    meta.depots.push(args[0].to_owned());
                }
            }
            _ => {}
        }
    }

    meta
}

fn is_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|character| character.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn extracts_app_id_and_depots() {
        let source = r#"
            addappid(2651281,0,"9874571b52ba0c01cbdfd32c5d81920d1093f42fbcf76d02a1e4589746a3b497")
            setManifestid(2651281,"2482069238759486759")
            addappid(2651280)
        "#;
        let meta = parse(source);
        assert_eq!(meta.depots, vec!["2651281", "2651280"]);
    }

    #[test]
    fn handles_many_depots_and_whitespace() {
        let source = "addappid( 100 )\naddappid( 200 , 0 , \"a\" )\naddappid(300,0,\"b\")";
        let meta = parse(source);
        assert_eq!(meta.depots, vec!["100", "200", "300"]);
    }

    #[test]
    fn ignores_other_function_calls() {
        let source = "setManifestid(999,\"1\")\naddtoken(1,\"2\")\naddappid(42)";
        let meta = parse(source);
        assert_eq!(meta.depots, vec!["42"]);
    }
}