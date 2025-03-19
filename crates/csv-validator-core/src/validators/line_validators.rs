use crate::utils::csv_utils::line_processor;
use crate::validators::issue::{ValidationIssue, ValidationResult};

// pub(crate) type Validator = dyn Fn(&str) -> Option<&str> + Sync;
pub type Validator = dyn Fn(ValidationResult, usize) -> ValidationResult + Send + Sync;

pub(crate) type Validators<'a> = &'a Vec<Box<Validator>>;

/// Validator: validate the number of fields in a line of a CSV file.
/// It will return the line if the number of fields is equal to the expected number.
///
///
/// # Example
///
/// ```
/// use csv_validator_core::validators::issue::ValidationResult;
/// use csv_validator_core::validators::line_validators::validate_line_field_count;
///
/// let line = "a,b,c";
/// let validation_result = ValidationResult::new(line.to_string());
/// let result = validate_line_field_count(validation_result, 3, &",".to_string(), Some('"'), 0);
/// assert_eq!(result.issues, Vec::new());
///
/// let line = "a,b";
/// let validation_result = ValidationResult::new(line.to_string());
/// let result = validate_line_field_count(validation_result, 3, &",".to_string(), Some('"'), 0);
/// assert_eq!(result.issues.len(), 1);
/// assert!(result.issues[0].message.contains("Incorrect field count"));
/// ```
pub fn validate_line_field_count<'a>(
    input: ValidationResult,
    num_fields: usize,
    separator: &String,
    quote_char: Option<char>,
    line_number: usize,
) -> ValidationResult {
    let line = &input.line;
    let fields_result = line_processor(line, separator, quote_char);
    let fields = match fields_result {
        Ok(fields) => fields,
        Err(_) => {
            let issue = ValidationIssue {
                line_number,
                position: None,
                message: "Error parsing fields".to_string(),
                fixed: false,
            };
            return ValidationResult {
                line: line.to_string(),
                issues: vec![issue],
            };
        }
    };
    if fields.len() > num_fields {
        // “fix” it by trimming extra fields
        let fixed_line = fields[..num_fields].join(&separator.to_string());
        let issue = ValidationIssue {
            line_number,
            position: None,
            message: format!(
                "Incorrect field count: expected {}, got {}. Fixed by trimming.",
                num_fields,
                fields.len()
            ),
            fixed: true,
        };
        ValidationResult {
            line: fixed_line,
            issues: {
                let mut v = input.issues;
                v.push(issue);
                v
            },
        }
    } else if fields.len() < num_fields {
        let fixed_line = format!("{},{}", line, " ".repeat(num_fields - fields.len()));
        let issue = ValidationIssue {
            line_number,
            position: None,
            message: format!(
                "Incorrect field count: expected {}, got {}.",
                num_fields,
                fields.len()
            ),
            fixed: true,
        };
        ValidationResult {
            line: fixed_line.to_string(),
            issues: {
                let mut v = input.issues;
                v.push(issue);
                v
            },
        }
    } else {
        input
    }

}

/// Validator: validate the presence of a separator in a line of a CSV file.
/// It will return the line if the separator is found.
///
///
/// # Example
///
/// ```
/// use csv_validator_core::validators::issue::ValidationResult;
/// use csv_validator_core::validators::line_validators::validate_line_separator;
///
/// let line = "a,b,c";
/// let validation_result = ValidationResult::new(line.to_string());
/// let result = validate_line_separator(validation_result, ',', 0);
/// assert_eq!(result.issues, Vec::new());
///
/// let line = "a;b;c";
/// let validation_result = ValidationResult::new(line.to_string());
/// let result = validate_line_separator(validation_result, ',', 0);
/// assert_eq!(result.issues.len(), 1);
/// dbg!(&result);
/// assert!(result.issues[0].message.contains("Expected separator"));
/// ```
pub fn validate_line_separator(
    input: ValidationResult,
    expected_sep: char,
    line_number: usize,
) -> ValidationResult {
    // todo: check if the separator is present in the line and is in fact the separator
    if !input.line.contains(expected_sep) {
        let issue = ValidationIssue {
            line_number,
            position: None,
            message: format!("Expected separator '{}' not found.", expected_sep),
            fixed: false,
        };
        input.add_issue(issue)
    } else {
        input
    }
}

// pub fn validate_line_separator(line: &str, separator: char) -> Option<&str> {
//     if line.contains(separator) {
//         return Some(line);
//     }
//     println!("Separator not found");
//     None
// }
