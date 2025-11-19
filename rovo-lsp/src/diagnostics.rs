use crate::parser::AnnotationKind;

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

pub fn validate_annotations(content: &str) -> Vec<Diagnostic> {
    let annotations = crate::parser::parse_annotations(content);
    let mut diagnostics = Vec::new();

    for ann in annotations {
        match ann.kind {
            AnnotationKind::Response => {
                if let Some(status) = ann.status {
                    if status < 100 || status > 599 {
                        diagnostics.push(Diagnostic {
                            line: ann.line,
                            message: format!(
                                "Invalid HTTP status code: {}. Must be between 100 and 599.",
                                status
                            ),
                            severity: DiagnosticSeverity::Error,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    diagnostics
}
