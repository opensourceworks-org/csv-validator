use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

type Validator = dyn Fn(&str) -> Option<&str> + Sync;
type Validators<'a> = &'a Vec<Box<Validator>>;

pub fn infer_separator(csv: &str) -> char {
    let mut separator = ',';
    let mut max_count = 0;

    for &c in &[',', ';', '\t'] {
        let count = csv.chars().filter(|&x| x == c).count();
        if count > max_count {
            max_count = count;
            separator = c;
        }
    }

    separator
}

/// Use this function to infer the separator of a CSV file using statistical analysis.
/// It will return the most likely separator.
/// 
fn infer_multi_char_separator(sample: &str) -> Option<String> {
    let lines: Vec<&str> = sample.lines().collect();

    if lines.len() < 2 {
        return None;
    }

    let mut substr_freq: HashMap<&str, usize> = HashMap::new();

    // Count frequency of substrings up to length 4 across all lines
    for line in &lines {
        for window_size in 1..=4 {
            for i in 0..=line.len().saturating_sub(window_size) {
                let substr = &line[i..i + window_size];
                *substr_freq.entry(substr).or_insert(0) += 1;
            }
        }
    }

    // Collect candidate substrings occurring more than once
    let mut candidates: Vec<&str> = substr_freq
        .iter()
        .filter(|&(_, &count)| count > 1)
        .map(|(&substr, _)| substr)
        .collect();

    // Sort candidates by length (longest first), then frequency
    candidates.sort_by(|a, b| b.len().cmp(&a.len()));

    for candidate in candidates {
        let counts: Vec<usize> = lines.iter().map(|line| line.matches(candidate).count()).collect();
        if counts.windows(2).all(|w| w[0] == w[1] && w[0] > 0) {
            return Some(candidate.to_string());
        }
    }

    None
}


pub fn validate_line_field_count(line: &str, num_fields: usize, separator: char) -> Option<&str> {
    dbg!(line);
    let fields: Vec<&str> = line.split(separator).collect();
    dbg!(&fields);
    dbg!(fields.len());
    if fields.len() != num_fields {
        println!("Not enough fields");
        return None;
    }
    Some(line)
}

pub fn validate_line_separator(line: &str, separator: char) -> Option<&str> {
    if line.contains(separator) {
        return Some(line);
    }
    println!("Separator not found");
    None
}

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

    #[test]
    fn test_infer_multi_char_separator() {
        let sample = "a,b,c\n1,2,3\n4,5,6";
        let result = infer_multi_char_separator(sample);
        assert_eq!(result.unwrap(), ",");

        let sample = "a;b;c\n1;2;3\n4;5;6";
        let result = infer_multi_char_separator(sample);
        assert_eq!(result.unwrap(), ";");

        let sample = "a\tb\tc\n1\t2\t3\n4\t5\t6";
        let result = infer_multi_char_separator(sample);
        assert_eq!(result.unwrap(), "\t");

        let sample = "a##b##c\n1##2##3\n4##5##6";
        let result = infer_multi_char_separator(sample);
        assert_eq!(result.unwrap(), "##");

        let sample = "a#@#b#@#c\n1#@#2#@#3\n4#@#5#@#6";
        let result = infer_multi_char_separator(sample);
        assert_eq!(result.unwrap(), "#@#");

    }
}
