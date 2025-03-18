pub(crate) type Validator = dyn Fn(&str) -> Option<&str> + Sync;
pub(crate) type Validators<'a> = &'a Vec<Box<Validator>>;

/// Validator: validate the number of fields in a line of a CSV file.
/// It will return the line if the number of fields is equal to the expected number.
/// Otherwise, it will return None.
///
/// # Example
///
/// ```
/// use csv_validator_core::validators::line_validators::validate_line_field_count;
///
/// let line = "a,b,c";
/// let result = validate_line_field_count(line, 3, &",".to_string());
/// assert!(result.is_some());
///     
/// let line = "a,b";
/// let result = validate_line_field_count(line, 3, &",".to_string());
/// assert!(result.is_none());
/// ```
pub fn validate_line_field_count<'a, 'b>(line: &'a str, num_fields: usize, separator: &'b String) -> Option<&'a str> {
    dbg!(line);
    let fields: Vec<&str> = line.split(separator).collect();
    dbg!(&fields);
    dbg!(fields.len());
    if fields.len() != num_fields {
        println!("Incorrect number of fields: expected {}, found {}", num_fields, fields.len());
        return None;
    }
    Some(line)
}

/// Validator: validate the presence of a separator in a line of a CSV file.
/// It will return the line if the separator is found.
/// Otherwise, it will return None.
///
/// # Example
///
/// ```
/// use csv_validator_core::validators::line_validators::validate_line_separator;
///
/// let line = "a,b,c";
/// let result = validate_line_separator(line, ',');
/// assert!(result.is_some());
///
/// let line = "a;b;c";
/// let result = validate_line_separator(line, ',');
/// assert!(result.is_none());
/// ```
pub fn validate_line_separator(line: &str, separator: char) -> Option<&str> {
    if line.contains(separator) {
        return Some(line);
    }
    println!("Separator not found");
    None
}

#[cfg(test)]
mod tests {
    // use super::*;
    use crate::validators::line_validators::validate_line_field_count;

    #[test]
    fn test_validate_line_field_count() {
        let line = "a,b,c";
        let result = validate_line_field_count(line, 3, &",".to_string());
        assert!(result.is_some());

        let line = "a,b";
        let result = validate_line_field_count(line, 3, &",".to_string());
        dbg!(&result);
        assert!(result.is_none());
    }
}
