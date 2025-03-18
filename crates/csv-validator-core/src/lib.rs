use rayon::prelude::*;
pub mod readers;

pub mod utils;
pub mod validators;
use crate::readers::csv_readers::{CsvBatchIterator, RawBatchIterator};
use crate::utils::csv_utils::{
    infer_multi_char_separator, infer_separator, infer_separator_from_file,
};
use crate::validators::line_validators::{validate_line_field_count, validate_line_separator};
use validators::line_validators::{Validator, Validators};

pub fn check_csv(csv_filename: &str) -> Result<char, Box<dyn std::error::Error>> {
    let csv = std::fs::read_to_string(csv_filename)?;
    println!("CSV: {}", csv);
    let separator = infer_separator(&csv);
    println!("Separator: {}", separator);
    Ok(separator)
}

fn check_buffered_lines<'a>(lines: &'a [&'a str], funcs: &'a Validators) {
    lines.par_iter().for_each(|&line| {
        let result = funcs.iter().try_fold(line, |current, f| {
            // If current is Some, call the function; otherwise keep None.
            f(current)
        });

        match result {
            Some(final_line) => println!("Processed: {}", final_line),
            None => println!("Processing stopped for line: '{}'", line),
        }
    });
}

pub fn validate_file(
    csv_filename: &str,
    validators: Validators,
) -> Result<(), Box<dyn std::error::Error>> {
    let iterator = RawBatchIterator::new(csv_filename, 5)?;

    for batch in iterator {
        println!("Batch of {} records:", batch.len());
        let batch_refs: Vec<&str> = batch.iter().map(|s| s.as_str()).collect();
        check_buffered_lines(&batch_refs, &validators);
    }

    Ok(())
}

pub fn main_validate(
    csv_filename: &str,
    num_fields: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let separator = infer_separator_from_file(csv_filename)?;
    dbg!(&separator);
    let sep = separator.clone();
    let funcs: Vec<Box<Validator>> = vec![
        Box::new(move |input| validate_line_field_count(input, num_fields, &sep.clone()) ),
        Box::new(move |input| validate_line_separator(input, ';') ),
    ];

    validate_file(csv_filename, &funcs)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_main_check_validate() {
        let csv_filename = "../../examples/full_quoted_with_header_semicolon.csv";
        let result = main_validate(csv_filename, 7);
        assert!(result.is_err());
    }
}
