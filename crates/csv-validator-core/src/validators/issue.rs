#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    pub line_number: usize,
    pub position: Option<usize>,
    pub message: String,
    pub fixed: bool,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub line: String,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn new(line: String) -> Self {
        Self { line, issues: Vec::new() }
    }

    /// Append an issue
    pub fn add_issue(mut self, issue: ValidationIssue) -> Self {
        self.issues.push(issue);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validation_issue() {
        let test_issue = ValidationIssue {
            line_number: 1,
            position: Some(5),
            message: "Invalid field".to_string(),
            fixed: false,
        };

        assert_eq!(test_issue.line_number, 1);

        let test_result = ValidationResult::new("a,b,c".to_string());
        assert_eq!(test_result.line, "a,b,c");

        let test_result = test_result.add_issue(test_issue.clone());
        assert_eq!(test_result.issues.len(), 1);


    }

    #[test]
    fn test_validation_result() {
        let test_result = ValidationResult::new("a,b,c".to_string());
        assert_eq!(test_result.line, "a,b,c");

        let test_issue = ValidationIssue {
            line_number: 1,
            position: Some(5),
            message: "Invalid field".to_string(),
            fixed: false,
        };

        let test_result = test_result.add_issue(test_issue.clone());
        assert_eq!(test_result.issues.len(), 1);
    }
}
