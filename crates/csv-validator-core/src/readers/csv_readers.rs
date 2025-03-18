use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

use crate::utils::csv_utils::line_processor;
use std::io;

pub struct RawBatchIterator {
    lines: Lines<BufReader<File>>,
    batch_size: usize,
}

impl RawBatchIterator {
    pub fn new(filename: &str, batch_size: usize) -> io::Result<Self> {
        let file = File::open(filename)?;
        let rdr = BufReader::new(file);
        let lines = rdr.lines();
        Ok(Self { lines, batch_size })
    }
}

impl Iterator for RawBatchIterator {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut batch = Vec::with_capacity(self.batch_size);

        for _ in 0..self.batch_size {
            match self.lines.next() {
                Some(Ok(line)) => batch.push(line),
                Some(Err(_)) => continue, // skip errors
                None => break,
            }
        }

        if batch.is_empty() { None } else { Some(batch) }
    }
}

pub struct CsvBatchIterator {
    lines: Lines<BufReader<File>>,
    batch_size: usize,
    separator: String,
    quote_char: char,
    has_header: bool,
}

impl CsvBatchIterator {
    pub fn new(
        filename: &str,
        batch_size: usize,
        separator: String,
        quote_char: char,
        has_header: bool,
    ) -> io::Result<Self> {
        let file = File::open(filename)?;
        let rdr = BufReader::new(file);
        let lines = rdr.lines();
        Ok(Self {
            lines,
            batch_size,
            separator,
            quote_char,
            has_header,
        })
    }
}

impl Iterator for CsvBatchIterator {
    type Item = Vec<Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut batch = Vec::with_capacity(self.batch_size);

        for _ in 0..self.batch_size {
            match self.lines.next() {
                Some(Ok(line)) => {
                    let split_line =
                        line_processor(&line, &self.separator.clone(), self.quote_char).ok()?;
                    batch.push(split_line)
                }
                Some(Err(_)) => continue, // skip errors
                None => break,
            }
        }

        if batch.is_empty() { None } else { Some(batch) }
    }
}

// pub fn validate_file(csv_filename: &str, validators: Validators) -> Result<(), Box<dyn std::error::Error>> {
//     let iterator = CsvBatchIterator::new(csv_filename, 5)?;
//
//     for batch in iterator {
//         println!("Batch of {} records:", batch.len());
//         for record in batch {
//             println!("validating ---> {:?}", record);
//         }
//     }
//
//     Ok(())
// }

// pub fn validates_csv(
//     csv_filename: &str,
//     validators: Validators,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let file = File::open(csv_filename)?;
//     let reader = BufReader::new(file);
//
//     let batch_size = 10_000;
//     let mut buffer = Vec::with_capacity(batch_size);
//
//     for line in reader.lines() {
//         buffer.push(line?);
//
//         if buffer.len() >= batch_size {
//             check_buffered_lines(&buffer, &validators);
//             buffer.clear();
//         }
//     }
//
//     if !buffer.is_empty() {
//         check_buffered_lines(&buffer, &validators);
//     }
//
//     Ok(())
// }
