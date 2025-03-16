use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub mod utils;
pub mod validators;
use crate::utils::csv_utils::infer_separator;
use crate::validators::line_validators::{validate_line_field_count, validate_line_separator};
use validators::line_validators::{Validator, Validators};

pub fn check_csv(csv_filename: &str) -> Result<char, Box<dyn std::error::Error>> {
    let csv = std::fs::read_to_string(csv_filename)?;
    println!("CSV: {}", csv);
    let separator = infer_separator(&csv);
    println!("Separator: {}", separator);
    Ok(separator)
}

fn check_lines<'a>(lines: &'a [String], funcs: &'a Validators) {
    lines.par_iter().for_each(|line| {
        let mut current = Some(line.as_str());
        for f in funcs.iter() {
            if let Some(s) = current {
                current = f(s);
            } else {
                break;
            }
        }

        if let Some(result) = current {
            println!("Processed: {}", result);
        } else {
            println!("Processing stopped for line: '{}'", line);
        }
    });
}

pub fn validate_csv(
    csv_filename: &str,
    validators: Validators,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(csv_filename)?;
    let reader = BufReader::new(file);

    let batch_size = 10_000;
    let mut buffer = Vec::with_capacity(batch_size);

    for line in reader.lines() {
        buffer.push(line?);

        if buffer.len() >= batch_size {
            check_lines(&buffer, &validators);
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        check_lines(&buffer, &validators);
    }

    Ok(())
}

pub fn main_validate(
    csv_filename: &str,
    num_fields: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let funcs: Vec<Box<Validator>> = vec![
        Box::new(move |input| validate_line_field_count(input, num_fields, ',')),
        Box::new(move |input| validate_line_separator(input, ',')),
    ];

    validate_csv(csv_filename, &funcs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validators::line_validators::validate_line_field_count;

    #[test]
    fn test_check_csv() {
        let csv_filename = "../../examples/with_header.csv";
        let result = check_csv(csv_filename);
        assert_eq!(result.unwrap(), ',');

        let csv_filename = "../../examples/full_quoted_with_header_semicolon.csv";
        let result = check_csv(csv_filename);
        assert_eq!(result.unwrap(), ';');
    }

    #[test]
    fn test_validate_line_field_count() {
        let line = "a,b,c";
        let result = validate_line_field_count(line, 3, ',');
        assert!(result.is_some());

        let line = "a,b";
        let result = validate_line_field_count(line, 3, ',');
        assert!(result.is_none());
    }

    #[test]
    fn test_main_check_validate() {
        let csv_filename = "../../examples/with_header.csv";
        let result = main_validate(csv_filename, 4);
        assert!(result.is_ok());
    }
}
