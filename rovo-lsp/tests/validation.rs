use rovo_lsp::diagnostics::{validate_annotations, DiagnosticSeverity};

#[test]
fn reports_invalid_status_code() {
    let content = r#"
/// @response 999 Json<User> Invalid
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Invalid HTTP status"));
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
}

#[test]
fn reports_status_code_too_low() {
    let content = r#"
/// @response 99 Json<User> Too low
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Invalid HTTP status"));
}

#[test]
fn reports_status_code_too_high() {
    let content = r#"
/// @response 600 Json<User> Too high
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("Invalid HTTP status"));
}

#[test]
fn accepts_valid_status_codes() {
    let content = r#"
/// @response 200 Json<User> OK
/// @response 404 Json<Error> Not found
/// @response 500 Json<Error> Server error
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn accepts_all_standard_ranges() {
    let content = r#"
/// @response 100 Json<Continue> Informational
/// @response 200 Json<Success> Success
/// @response 301 Json<Redirect> Redirection
/// @response 404 Json<Error> Client error
/// @response 500 Json<Error> Server error
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn reports_multiple_errors() {
    let content = r#"
/// @response 999 Json<User> Invalid
/// @response 50 Json<Error> Also invalid
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 2);
}

#[test]
fn no_diagnostics_for_non_response_annotations() {
    let content = r#"
/// @tag users
/// @security bearer
/// @id get_user
#[rovo]
async fn handler() {}
"#;
    let diagnostics = validate_annotations(content);
    assert_eq!(diagnostics.len(), 0);
}
