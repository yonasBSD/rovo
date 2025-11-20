use crate::utils::utf16_pos_to_byte_index;
use serde::{Deserialize, Serialize};

/// Position in a text document
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Character offset in line (0-indexed)
    pub character: usize,
}

/// An auto-completion suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Label shown in completion list
    pub label: String,
    /// Kind of completion item
    pub kind: CompletionItemKind,
    /// Short detail shown alongside label
    pub detail: Option<String>,
    /// Full documentation for this item
    pub documentation: Option<String>,
    /// Text to insert when selected
    pub insert_text: Option<String>,
}

/// Type of completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionItemKind {
    /// A language keyword
    Keyword,
    /// A code snippet
    Snippet,
}

/// Get completion suggestions at the given position
///
/// # Arguments
/// * `content` - The source code content
/// * `position` - Cursor position where completion was requested
///
/// # Returns
/// A vector of completion suggestions
pub fn get_completions(content: &str, position: Position) -> Vec<CompletionItem> {
    let lines: Vec<&str> = content.lines().collect();

    if position.line >= lines.len() {
        return Vec::new();
    }

    let line = lines[position.line];

    // Convert UTF-16 character offset to UTF-8 byte index
    // LSP Position.character is in UTF-16 code units, but Rust strings use UTF-8
    let byte_index = utf16_pos_to_byte_index(line, position.character).unwrap_or(line.len());

    let prefix = &line[..byte_index];

    // Check if we're in a doc comment
    if !prefix.trim_start().starts_with("///") {
        return Vec::new();
    }

    // Handle indented doc comments by trimming whitespace first
    let after_doc = prefix.trim_start().trim_start_matches("///").trim_start();

    // Check for status code completion (after @response or @example)
    if after_doc.starts_with("@response ") || after_doc.starts_with("@example ") {
        let parts: Vec<&str> = after_doc.split_whitespace().collect();
        // If we have only the annotation keyword, or started typing a status code
        if parts.len() == 1 {
            return get_status_code_completions("");
        } else if parts.len() == 2 {
            // Filter by what's been typed
            return get_status_code_completions(parts[1]);
        }
    }

    // Check for security scheme completion (after @security)
    if after_doc.starts_with("@security ") {
        let parts: Vec<&str> = after_doc.split_whitespace().collect();
        if parts.len() == 1 {
            return get_security_scheme_completions("");
        } else if parts.len() == 2 {
            // Filter by what's been typed
            return get_security_scheme_completions(parts[1]);
        }
    }

    // Check if we're typing an annotation (after @)
    if after_doc.starts_with('@') {
        // Get what's been typed after @
        let typed = after_doc.trim_start_matches('@');

        // Return all annotations that match the typed prefix
        let mut completions = Vec::new();

        let annotations = [
            (
                "response",
                "@response ${1:200} ${2:Json<T>} ${3:Description}",
            ),
            ("tag", "@tag ${1:tag_name}"),
            ("security", "@security ${1:bearer}"),
            ("example", "@example ${1:200} ${2:{\"key\": \"value\"}}"),
            ("id", "@id ${1:operation_id}"),
            ("hidden", "@hidden"),
        ];

        for (label, snippet) in annotations {
            if label.starts_with(typed) {
                let full_label = format!("@{}", label);
                let detail = crate::docs::get_annotation_summary(&full_label);
                let documentation = crate::docs::get_annotation_documentation(&full_label);
                completions.push(CompletionItem {
                    label: full_label,
                    kind: CompletionItemKind::Snippet,
                    detail: Some(detail.to_string()),
                    documentation: Some(documentation.to_string()),
                    insert_text: Some(snippet.to_string()),
                });
            }
        }

        completions
    } else {
        Vec::new()
    }
}

fn get_status_code_completions(filter: &str) -> Vec<CompletionItem> {
    let codes = [
        (200, "OK - Request succeeded", "**200 OK**\n\nRequest succeeded. The meaning depends on the HTTP method:\n- **GET**: Resource fetched and transmitted in response body\n- **POST**: Resource created or action performed\n- **PUT**: Resource updated\n- **DELETE**: Resource deleted\n\nThis is the standard response for successful HTTP requests."),
        (201, "Created - Resource created", "**201 Created**\n\nRequest succeeded and a new resource was created as a result.\n\nTypically returned after:\n- **POST** requests that create a resource\n- **PUT** requests that create a new resource\n\nThe `Location` header often contains the URL of the newly created resource."),
        (204, "No Content - Success with no response body", "**204 No Content**\n\nRequest succeeded but there's no content to return.\n\nOften used for:\n- **DELETE** operations (resource deleted successfully)\n- **PUT** operations (resource updated, no content to return)\n- Operations where the result doesn't require a response body\n\nNo response body should be sent with this status."),
        (400, "Bad Request - Invalid input", "**400 Bad Request**\n\nServer cannot process the request due to client error.\n\nCommon causes:\n- Malformed request syntax\n- Invalid request message framing\n- Deceptive request routing\n- Missing required parameters\n- Invalid parameter types\n\nThe client should not repeat the request without modifications."),
        (401, "Unauthorized - Authentication required", "**401 Unauthorized**\n\nClient must authenticate itself to get the requested response.\n\nKey points:\n- The client is **not authenticated**\n- Authentication is required and has either failed or not been provided\n- The `WWW-Authenticate` header typically includes information on how to authenticate\n\nNote: Despite the name, this status means **unauthenticated**, not unauthorized."),
        (403, "Forbidden - Insufficient permissions", "**403 Forbidden**\n\nClient does not have access rights to the content.\n\nKey differences from 401:\n- The client's **identity is known** to the server\n- The client **lacks permission** to access the resource\n- Re-authenticating won't help\n\nUsed when the user is authenticated but doesn't have the required permissions."),
        (404, "Not Found - Resource doesn't exist", "**404 Not Found**\n\nServer cannot find the requested resource.\n\nThis is one of the most famous HTTP status codes.\n\nCommon causes:\n- Resource has been deleted\n- Wrong URL/path\n- Resource never existed\n\nCan also be used to hide a resource's existence for security reasons (instead of 403)."),
        (409, "Conflict - Resource conflict", "**409 Conflict**\n\nRequest conflicts with the current state of the server.\n\nCommon scenarios:\n- Concurrent modification conflicts\n- Version conflicts (optimistic locking)\n- Duplicate resource creation\n- Business rule violations\n\nThe client may be able to resolve the conflict and resubmit."),
        (422, "Unprocessable Entity - Validation error", "**422 Unprocessable Entity**\n\nRequest was well-formed but contains semantic errors.\n\nCommonly used for:\n- **Validation failures** (field constraints, formats)\n- Business logic violations\n- Invalid data combinations\n\nThe request syntax is correct (unlike 400), but the content cannot be processed due to semantic errors."),
        (500, "Internal Server Error - Server error", "**500 Internal Server Error**\n\nServer encountered an unexpected condition that prevented it from fulfilling the request.\n\nThis is a generic error message when:\n- No more specific error message is suitable\n- The server has an unexpected error\n- An unhandled exception occurs\n\nThe issue is on the server side, not the client."),
        (503, "Service Unavailable - Server temporarily unavailable", "**503 Service Unavailable**\n\nServer is not ready to handle the request.\n\nCommon causes:\n- Server **maintenance** or updates\n- Server is **overloaded**\n- Temporary resource exhaustion\n\nThe `Retry-After` header may indicate when to try again.\n\nUnlike 500, this suggests the condition is temporary."),
    ];

    codes
        .iter()
        .filter(|(code, _, _)| {
            // If no filter, show all; otherwise filter by prefix
            filter.is_empty() || code.to_string().starts_with(filter)
        })
        .map(|(code, desc, docs)| CompletionItem {
            label: code.to_string(),
            kind: CompletionItemKind::Keyword,
            detail: Some(desc.to_string()),
            documentation: Some(docs.to_string()),
            insert_text: Some(code.to_string()),
        })
        .collect()
}

fn get_security_scheme_completions(filter: &str) -> Vec<CompletionItem> {
    let schemes = [
        (
            "bearer",
            "Bearer token authentication",
            "**Bearer Authentication**\n\n\"Bearer\" means **whoever holds (bears) this token gets access**.\n\n## How it works\n\nThe token is passed in the `Authorization` header:\n```\nAuthorization: Bearer <token>\n```\n\n## Token types (bearer is the transport, not the format)\n\n### Session tokens\n```\nAuthorization: Bearer abc123sessionid456\n```\n- Random string/UUID\n- Maps to session data in your database\n- Server validates by looking up the session\n\n### JWT (JSON Web Tokens)\n```\nAuthorization: Bearer eyJhbGc...\n```\n- Self-contained with claims\n- Server validates signature\n- Can be stateless (no DB lookup)\n\n### OAuth 2.0 access tokens\n```\nAuthorization: Bearer oauth_token_xyz\n```\n- Obtained from OAuth authorization flow\n- Can be JWT or opaque token\n\n### Custom tokens\n- Any format your API supports\n- The bearer scheme just defines HOW to send it\n\n## Key concept\n**Bearer = Transport mechanism, NOT token format**\n\nYou can use bearer auth with:\n- ✅ Session IDs\n- ✅ JWTs\n- ✅ Random tokens\n- ✅ Any token format\n\n⚠️ **Security**: Always use HTTPS - bearer tokens are credentials!"
        ),
        (
            "basic",
            "Basic HTTP authentication (username/password)",
            "**Basic Authentication**\n\nSimple authentication scheme built into the HTTP protocol.\n\n## How it works\n\nCredentials are sent as base64-encoded `username:password`:\n```\nAuthorization: Basic <base64(username:password)>\n```\n\n## Example\nFor username `user` and password `pass123`:\n```\nAuthorization: Basic dXNlcjpwYXNzMTIz\n```\n\n## Use cases\n- Simple APIs with username/password\n- Internal services\n- Development/testing environments\n\n⚠️ **Security Warning**\n- Credentials are only **base64 encoded**, NOT encrypted\n- **MUST be used with HTTPS** in production\n- Consider using bearer tokens for better security"
        ),
        (
            "apiKey",
            "API key in header, query, or cookie",
            "**API Key Authentication**\n\nAuthentication using a simple API key that can be sent in multiple ways.\n\n## Transmission methods\n\n**Header** (recommended):\n```\nX-API-Key: your-api-key-here\nAPI-Key: your-api-key-here\n```\n\n**Query parameter**:\n```\nGET /api/users?api_key=your-api-key-here\n```\n\n**Cookie**:\n```\nCookie: api_key=your-api-key-here\n```\n\n## Common use cases\n- Public APIs (rate limiting, analytics)\n- Service-to-service authentication\n- Third-party integrations\n- Partner API access\n\n## Best practices\n- Use headers instead of query params (prevents logging)\n- Rotate keys regularly\n- Use different keys per environment\n- Support key revocation"
        ),
        (
            "oauth2",
            "OAuth 2.0 authentication flow",
            "**OAuth 2.0**\n\nIndustry-standard protocol for authorization. Enables applications to obtain limited access to user accounts on an HTTP service.\n\n## Grant Types (Flows)\n\n### Authorization Code\n- **Best for**: Web/mobile apps\n- **Flow**: User → Login → Code → Exchange for token\n- **Most secure** for public clients\n\n### Client Credentials\n- **Best for**: Service-to-service\n- **Flow**: Client → Token (no user interaction)\n- Used for machine-to-machine authentication\n\n### Implicit (Deprecated)\n- **Legacy**: Browser-based apps\n- **Status**: No longer recommended\n- Use Authorization Code with PKCE instead\n\n### Resource Owner Password\n- **Best for**: Highly trusted apps\n- **Flow**: Username/password → Token\n- Only use when you control both client and server\n\n## Key concepts\n- **Scopes**: Limit access to specific resources\n- **Access Token**: Short-lived token for API access\n- **Refresh Token**: Long-lived token to get new access tokens\n- **Token expiration**: Enhances security\n\n## Advantages\n- User never shares password with app\n- Fine-grained permissions (scopes)\n- Token revocation\n- Industry standard"
        ),
    ];

    schemes
        .iter()
        .filter(|(scheme, _, _)| {
            // If no filter, show all; otherwise filter by prefix
            filter.is_empty() || scheme.starts_with(filter)
        })
        .map(|(scheme, desc, docs)| CompletionItem {
            label: scheme.to_string(),
            kind: CompletionItemKind::Keyword,
            detail: Some(desc.to_string()),
            documentation: Some(docs.to_string()),
            insert_text: Some(scheme.to_string()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completes_at_sign() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);
        assert_eq!(completions.len(), 6);
        assert!(completions.iter().any(|c| c.label == "@response"));
        assert!(completions.iter().any(|c| c.label == "@tag"));
    }

    #[test]
    fn test_filters_by_prefix() {
        let content = "/// @r";
        let position = Position {
            line: 0,
            character: 6,
        };
        let completions = get_completions(content, position);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "@response");
    }

    #[test]
    fn test_no_completions_outside_doc_comment() {
        let content = "@response";
        let position = Position {
            line: 0,
            character: 1,
        };
        let completions = get_completions(content, position);
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_status_code_completion_after_response() {
        let content = "/// @response ";
        let position = Position {
            line: 0,
            character: 14,
        };
        let completions = get_completions(content, position);
        // Should offer status codes
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "200"));
        assert!(completions.iter().any(|c| c.label == "404"));
        assert!(completions.iter().any(|c| c.label == "500"));
    }

    #[test]
    fn test_status_code_completion_after_example() {
        let content = "/// @example ";
        let position = Position {
            line: 0,
            character: 13,
        };
        let completions = get_completions(content, position);
        // Should offer status codes for @example too
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "200"));
    }

    #[test]
    fn test_status_code_filtering_by_prefix() {
        let content = "/// @response 4";
        let position = Position {
            line: 0,
            character: 15,
        };
        let completions = get_completions(content, position);
        // Should only show 4xx codes
        assert!(completions.iter().all(|c| c.label.starts_with('4')));
        assert!(completions.iter().any(|c| c.label == "400"));
        assert!(completions.iter().any(|c| c.label == "404"));
        assert!(!completions.iter().any(|c| c.label == "200"));
    }

    #[test]
    fn test_status_code_filtering_specific() {
        let content = "/// @response 20";
        let position = Position {
            line: 0,
            character: 16,
        };
        let completions = get_completions(content, position);
        // Should show 20x codes
        assert!(completions.iter().all(|c| c.label.starts_with("20")));
        assert!(completions.iter().any(|c| c.label == "200"));
        assert!(completions.iter().any(|c| c.label == "201"));
        assert!(completions.iter().any(|c| c.label == "204"));
    }

    #[test]
    fn test_security_scheme_completion() {
        let content = "/// @security ";
        let position = Position {
            line: 0,
            character: 14,
        };
        let completions = get_completions(content, position);
        // Should offer security schemes
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "bearer"));
        assert!(completions.iter().any(|c| c.label == "basic"));
        assert!(completions.iter().any(|c| c.label == "apiKey"));
        assert!(completions.iter().any(|c| c.label == "oauth2"));
    }

    #[test]
    fn test_security_scheme_filtering() {
        let content = "/// @security b";
        let position = Position {
            line: 0,
            character: 15,
        };
        let completions = get_completions(content, position);
        // Should only show schemes starting with 'b'
        assert_eq!(completions.len(), 2); // bearer and basic
        assert!(completions.iter().any(|c| c.label == "bearer"));
        assert!(completions.iter().any(|c| c.label == "basic"));
        assert!(!completions.iter().any(|c| c.label == "oauth2"));
    }

    #[test]
    fn test_security_scheme_filtering_specific() {
        let content = "/// @security be";
        let position = Position {
            line: 0,
            character: 16,
        };
        let completions = get_completions(content, position);
        // Should only show bearer
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "bearer");
    }

    #[test]
    fn test_completion_items_have_documentation() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);

        // All completions should have documentation
        for completion in &completions {
            assert!(
                completion.documentation.is_some(),
                "Completion '{}' missing documentation",
                completion.label
            );
            assert!(!completion.documentation.as_ref().unwrap().is_empty());
        }
    }

    #[test]
    fn test_status_code_completions_have_details() {
        let content = "/// @response ";
        let position = Position {
            line: 0,
            character: 14,
        };
        let completions = get_completions(content, position);

        // All status codes should have detail and documentation
        for completion in &completions {
            assert!(completion.detail.is_some());
            assert!(completion.documentation.is_some());
        }
    }

    #[test]
    fn test_security_scheme_completions_have_details() {
        let content = "/// @security ";
        let position = Position {
            line: 0,
            character: 14,
        };
        let completions = get_completions(content, position);

        // All security schemes should have detail and documentation
        for completion in &completions {
            assert!(completion.detail.is_some());
            assert!(completion.documentation.is_some());
            assert!(!completion.documentation.as_ref().unwrap().is_empty());
        }
    }

    #[test]
    fn test_handles_indented_doc_comments() {
        let content = "    /// @";
        let position = Position {
            line: 0,
            character: 9,
        };
        let completions = get_completions(content, position);
        // Should work with indented comments
        assert_eq!(completions.len(), 6);
    }

    #[test]
    fn test_handles_utf16_positions() {
        // Content with multibyte characters - just ensure it doesn't crash
        let content = "/// 世界 @response";
        let position = Position {
            line: 0,
            character: 12, // Somewhere in the line
        };
        let completions = get_completions(content, position);
        // Should not crash with UTF-16 positions
        // (may or may not offer completions depending on exact position)
        assert!(completions.len() <= 11); // At most all status codes
    }

    #[test]
    fn test_out_of_bounds_line() {
        let content = "/// @";
        let position = Position {
            line: 100, // Way out of bounds
            character: 0,
        };
        let completions = get_completions(content, position);
        // Should return empty, not crash
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_empty_filter_shows_all_annotations() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);
        // Should show all 6 annotations
        assert_eq!(completions.len(), 6);
    }

    #[test]
    fn test_partial_annotation_filters() {
        let content = "/// @s";
        let position = Position {
            line: 0,
            character: 6,
        };
        let completions = get_completions(content, position);
        // Should only show @security
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "@security");
    }

    #[test]
    fn test_completion_has_insert_text() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);

        // All completions should have insert text
        for completion in &completions {
            assert!(completion.insert_text.is_some());
        }
    }

    #[test]
    fn test_response_completion_has_snippet() {
        let content = "/// @r";
        let position = Position {
            line: 0,
            character: 6,
        };
        let completions = get_completions(content, position);

        assert_eq!(completions.len(), 1);
        let insert_text = completions[0].insert_text.as_ref().unwrap();
        // Should have snippet placeholders
        assert!(insert_text.contains("${1"));
        assert!(insert_text.contains("${2"));
    }

    #[test]
    fn test_multiline_content() {
        let content = "/// First line\n/// @";
        let position = Position {
            line: 1,
            character: 5,
        };
        let completions = get_completions(content, position);
        // Should work on second line
        assert_eq!(completions.len(), 6);
    }

    #[test]
    fn test_no_completion_after_complete_annotation() {
        let content = "/// @response 200 Json<T> Success";
        let position = Position {
            line: 0,
            character: 20, // In the middle
        };
        let completions = get_completions(content, position);
        // Should not offer completions after the annotation is complete
        assert_eq!(completions.len(), 0);
    }

    #[test]
    fn test_empty_status_filter() {
        let completions = get_status_code_completions("");
        // Should return all status codes
        assert_eq!(completions.len(), 11); // 200, 201, 204, 400, 401, 403, 404, 409, 422, 500, 503
    }

    #[test]
    fn test_status_5xx_filter() {
        let completions = get_status_code_completions("5");
        // Should only return 5xx codes
        assert!(completions.iter().all(|c| c.label.starts_with('5')));
        assert!(completions.iter().any(|c| c.label == "500"));
        assert!(completions.iter().any(|c| c.label == "503"));
    }

    #[test]
    fn test_empty_security_filter() {
        let completions = get_security_scheme_completions("");
        // Should return all security schemes
        assert_eq!(completions.len(), 4); // bearer, basic, apiKey, oauth2
    }

    #[test]
    fn test_security_oauth_filter() {
        let completions = get_security_scheme_completions("o");
        // Should only return oauth2
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "oauth2");
    }

    #[test]
    fn test_completion_item_kind() {
        let content = "/// @";
        let position = Position {
            line: 0,
            character: 5,
        };
        let completions = get_completions(content, position);

        // Annotations should be snippets
        for completion in &completions {
            assert!(matches!(completion.kind, CompletionItemKind::Snippet));
        }
    }

    #[test]
    fn test_status_code_kind() {
        let completions = get_status_code_completions("");

        // Status codes should be keywords
        for completion in &completions {
            assert!(matches!(completion.kind, CompletionItemKind::Keyword));
        }
    }

    #[test]
    fn test_security_scheme_kind() {
        let completions = get_security_scheme_completions("");

        // Security schemes should be keywords
        for completion in &completions {
            assert!(matches!(completion.kind, CompletionItemKind::Keyword));
        }
    }
}
