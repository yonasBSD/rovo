use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationKind {
    Response,
    Tag,
    Security,
    Example,
    Id,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub kind: AnnotationKind,
    pub line: usize,

    // Response fields
    pub status: Option<u16>,
    pub response_type: Option<String>,
    pub description: Option<String>,

    // Tag fields
    pub tag_name: Option<String>,

    // Security fields
    pub security_scheme: Option<String>,

    // Example fields
    pub example_value: Option<String>,

    // ID fields
    pub operation_id: Option<String>,
}

impl Annotation {
    fn new(kind: AnnotationKind, line: usize) -> Self {
        Self {
            kind,
            line,
            status: None,
            response_type: None,
            description: None,
            tag_name: None,
            security_scheme: None,
            example_value: None,
            operation_id: None,
        }
    }
}

/// Check if a given position (line number) is near a #[rovo] attribute
pub fn is_near_rovo_attribute(content: &str, target_line: usize) -> bool {
    let lines: Vec<&str> = content.lines().collect();

    // Look ahead up to 20 lines to find a #[rovo] attribute
    for i in target_line..std::cmp::min(target_line + 20, lines.len()) {
        if lines[i].trim() == "#[rovo]" || lines[i].contains("#[") && lines[i].contains("rovo") {
            return true;
        }
        // Stop if we hit a non-comment, non-attribute line
        if !lines[i].trim().starts_with("///")
            && !lines[i].trim().starts_with("#[")
            && !lines[i].trim().is_empty()
        {
            break;
        }
    }

    false
}

pub fn parse_annotations(content: &str) -> Vec<Annotation> {
    let lines: Vec<&str> = content.lines().collect();
    let mut annotations = Vec::new();

    // Find all #[rovo] attributes
    let mut rovo_positions = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if line.trim() == "#[rovo]" {
            rovo_positions.push(idx);
        }
    }

    // For each #[rovo], look backwards for doc comments
    for rovo_pos in rovo_positions {
        let mut i = rovo_pos;
        while i > 0 {
            i -= 1;
            let line = lines[i].trim();

            // Stop if we hit a non-doc-comment line
            if !line.starts_with("///") {
                break;
            }

            // Parse annotation
            if let Some(ann) = parse_annotation_line(line, i) {
                annotations.insert(0, ann);
            }
        }
    }

    annotations
}

fn parse_annotation_line(line: &str, line_num: usize) -> Option<Annotation> {
    // Remove /// prefix and trim
    let content = line.trim_start_matches("///").trim();

    // Check if it starts with @
    if !content.starts_with('@') {
        return None;
    }

    // Parse different annotation types
    if content.starts_with("@response") {
        parse_response(content, line_num)
    } else if content.starts_with("@tag") {
        parse_tag(content, line_num)
    } else if content.starts_with("@security") {
        parse_security(content, line_num)
    } else if content.starts_with("@example") {
        parse_example(content, line_num)
    } else if content.starts_with("@id") {
        parse_id(content, line_num)
    } else if content.starts_with("@hidden") {
        Some(Annotation::new(AnnotationKind::Hidden, line_num))
    } else {
        None
    }
}

fn parse_response(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @response STATUS TYPE DESCRIPTION
    let re = Regex::new(r"@response\s+(\d+)\s+(\S+)\s*(.*)").unwrap();

    if let Some(captures) = re.captures(content) {
        let status: u16 = captures.get(1)?.as_str().parse().ok()?;
        let response_type = captures.get(2)?.as_str().to_string();
        let description = captures.get(3).map(|m| m.as_str().to_string());

        let mut ann = Annotation::new(AnnotationKind::Response, line_num);
        ann.status = Some(status);
        ann.response_type = Some(response_type);
        ann.description = description;

        Some(ann)
    } else {
        None
    }
}

fn parse_tag(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @tag NAME
    let re = Regex::new(r"@tag\s+(\S+)").unwrap();

    if let Some(captures) = re.captures(content) {
        let tag_name = captures.get(1)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Tag, line_num);
        ann.tag_name = Some(tag_name);

        Some(ann)
    } else {
        None
    }
}

fn parse_security(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @security SCHEME
    let re = Regex::new(r"@security\s+(\S+)").unwrap();

    if let Some(captures) = re.captures(content) {
        let security_scheme = captures.get(1)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Security, line_num);
        ann.security_scheme = Some(security_scheme);

        Some(ann)
    } else {
        None
    }
}

fn parse_example(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @example STATUS JSON
    let re = Regex::new(r"@example\s+(\d+)\s+(.+)").unwrap();

    if let Some(captures) = re.captures(content) {
        let status: u16 = captures.get(1)?.as_str().parse().ok()?;
        let example_value = captures.get(2)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Example, line_num);
        ann.status = Some(status);
        ann.example_value = Some(example_value);

        Some(ann)
    } else {
        None
    }
}

fn parse_id(content: &str, line_num: usize) -> Option<Annotation> {
    // Format: @id OPERATION_ID
    let re = Regex::new(r"@id\s+(\S+)").unwrap();

    if let Some(captures) = re.captures(content) {
        let operation_id = captures.get(1)?.as_str().to_string();

        let mut ann = Annotation::new(AnnotationKind::Id, line_num);
        ann.operation_id = Some(operation_id);

        Some(ann)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response() {
        let line = "/// @response 200 Json<User> Success";
        let ann = parse_annotation_line(line, 0).unwrap();
        assert_eq!(ann.kind, AnnotationKind::Response);
        assert_eq!(ann.status, Some(200));
        assert_eq!(ann.response_type, Some("Json<User>".to_string()));
        assert_eq!(ann.description, Some("Success".to_string()));
    }

    #[test]
    fn test_parse_tag() {
        let line = "/// @tag users";
        let ann = parse_annotation_line(line, 0).unwrap();
        assert_eq!(ann.kind, AnnotationKind::Tag);
        assert_eq!(ann.tag_name, Some("users".to_string()));
    }
}
