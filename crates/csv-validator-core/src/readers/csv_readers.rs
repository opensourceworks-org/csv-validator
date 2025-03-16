use crate::check_buffered_lines;
use crate::validators::line_validators::Validators;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
            check_buffered_lines(&buffer, &validators);
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        check_buffered_lines(&buffer, &validators);
    }

    Ok(())
}
