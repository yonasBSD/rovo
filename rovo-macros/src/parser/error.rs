use proc_macro2::Span;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    message: String,
    span: Option<Span>,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span: Some(span),
        }
    }

    pub const fn span(&self) -> Option<Span> {
        self.span
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_error_without_span() {
        let error = ParseError::new("test error");
        assert!(error.span().is_none());
        assert_eq!(error.to_string(), "test error");
    }

    #[test]
    fn creates_error_with_span() {
        let span = Span::call_site();
        let error = ParseError::with_span("test error", span);
        assert!(error.span().is_some());
        assert_eq!(error.to_string(), "test error");
    }

    #[test]
    fn display_format_works() {
        let error = ParseError::new("custom message");
        let formatted = format!("{}", error);
        assert_eq!(formatted, "custom message");
    }

    #[test]
    fn debug_format_works() {
        let error = ParseError::new("debug test");
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ParseError"));
    }
}
