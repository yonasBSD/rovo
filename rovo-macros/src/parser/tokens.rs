use proc_macro2::TokenStream;

/// Extract the state type from State<T> in function parameters
/// Returns None if no State extractor is found (meaning state type is ())
pub fn extract_state_type(tokens: &TokenStream) -> Option<TokenStream> {
    let token_str = tokens.to_string();

    // Look for pattern "State < SomeType >"
    // This is a simplified parser that looks for State<...>
    if let Some(state_pos) = token_str.find("State") {
        let after_state = &token_str[state_pos..];
        if let Some(open_bracket) = after_state.find('<') {
            // Find the matching closing bracket
            let after_open = &after_state[open_bracket + 1..];
            let mut depth = 1;
            let mut close_pos = 0;

            for (i, ch) in after_open.chars().enumerate() {
                if ch == '<' {
                    depth += 1;
                } else if ch == '>' {
                    depth -= 1;
                    if depth == 0 {
                        close_pos = i;
                        break;
                    }
                }
            }

            if close_pos > 0 {
                let state_type_str = &after_open[..close_pos].trim();
                // Parse the extracted string back into a TokenStream
                if let Ok(state_type) = state_type_str.parse::<TokenStream>() {
                    return Some(state_type);
                }
            }
        }
    }

    None
}

/// Extract doc comment text from an attribute string
pub fn extract_doc_text(attr: &str) -> String {
    // Parse doc = "text" format
    if let Some(start) = attr.find('"') {
        if let Some(end) = attr.rfind('"') {
            if start < end {
                return attr[start + 1..end].to_string();
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_simple_state_type() {
        let tokens: TokenStream = "State<AppState>".parse().unwrap();
        let result = extract_state_type(&tokens);
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "AppState");
    }

    #[test]
    fn extracts_nested_state_type() {
        let tokens: TokenStream = "State<Arc<RwLock<Database>>>".parse().unwrap();
        let result = extract_state_type(&tokens);
        assert!(result.is_some());
        // Token spacing may vary
        let result_str = result.unwrap().to_string();
        assert!(result_str.contains("Arc"));
        assert!(result_str.contains("RwLock"));
        assert!(result_str.contains("Database"));
    }

    #[test]
    fn returns_none_when_no_state() {
        let tokens: TokenStream = "Path<u32>, Json<User>".parse().unwrap();
        let result = extract_state_type(&tokens);
        assert!(result.is_none());
    }

    #[test]
    fn handles_state_with_surrounding_params() {
        let tokens: TokenStream = "Path<u32>, State<AppState>, Json<User>".parse().unwrap();
        let result = extract_state_type(&tokens);
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "AppState");
    }

    #[test]
    fn extracts_doc_text_simple() {
        let attr = r#"doc = "This is a comment""#;
        let result = extract_doc_text(attr);
        assert_eq!(result, "This is a comment");
    }

    #[test]
    fn extracts_doc_text_with_special_chars() {
        let attr = r#"doc = "Contains 'quotes' and \"escapes\"""#;
        let result = extract_doc_text(attr);
        assert_eq!(result, r#"Contains 'quotes' and \"escapes\""#);
    }

    #[test]
    fn returns_empty_string_when_no_quotes() {
        let attr = "doc = no quotes here";
        let result = extract_doc_text(attr);
        assert_eq!(result, "");
    }

    #[test]
    fn returns_empty_string_when_only_one_quote() {
        let attr = r#"doc = "incomplete"#;
        let result = extract_doc_text(attr);
        assert_eq!(result, "");
    }

    #[test]
    fn handles_empty_doc_text() {
        let attr = r#"doc = """#;
        let result = extract_doc_text(attr);
        assert_eq!(result, "");
    }

    #[test]
    fn extracts_multiline_doc_text() {
        let attr = r#"doc = "Line 1\nLine 2\nLine 3""#;
        let result = extract_doc_text(attr);
        assert_eq!(result, r"Line 1\nLine 2\nLine 3");
    }
}
