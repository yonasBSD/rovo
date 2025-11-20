use crate::parser::AnnotationKind;

/// Severity level for diagnostic messages
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    /// An error that should be fixed
    Error,
    /// A warning that should be addressed
    Warning,
}

/// A diagnostic message indicating an issue with annotations
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Line number where the diagnostic applies (0-indexed, matching LSP protocol)
    ///
    /// Note: This uses 0-based line indexing to align with the LSP specification.
    /// Line 0 is the first line of the file.
    pub line: usize,
    /// Human-readable diagnostic message
    pub message: String,
    /// Severity level of this diagnostic
    pub severity: DiagnosticSeverity,
    /// Character range for the diagnostic (optional, defaults to whole line)
    pub char_start: Option<usize>,
    /// End character position (optional, defaults to end of line)
    pub char_end: Option<usize>,
}

/// Validate Rovo annotations in the given content
///
/// Checks for issues like invalid HTTP status codes and returns a list of diagnostics.
///
/// # Arguments
/// * `content` - The source code content to validate
///
/// # Returns
/// A vector of diagnostics for any validation errors found
pub fn validate_annotations(content: &str) -> Vec<Diagnostic> {
    let annotations = crate::parser::parse_annotations(content);
    let lines: Vec<&str> = content.lines().collect();
    let mut diagnostics = Vec::new();

    for ann in annotations {
        match ann.kind {
            AnnotationKind::Response => {
                if let Some(status) = ann.status {
                    if status < 100 || status > 599 {
                        // Find the position of the status code in the line
                        let (char_start, char_end) = if ann.line < lines.len() {
                            let line = lines[ann.line];
                            let status_str = status.to_string();
                            if let Some(pos) = line.find(&status_str) {
                                (Some(pos), Some(pos + status_str.len()))
                            } else {
                                (None, None)
                            }
                        } else {
                            (None, None)
                        };

                        diagnostics.push(Diagnostic {
                            line: ann.line,
                            message: format!(
                                "Invalid HTTP status code: {}. Must be between 100 and 599.",
                                status
                            ),
                            severity: DiagnosticSeverity::Error,
                            char_start,
                            char_end,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    diagnostics
}
