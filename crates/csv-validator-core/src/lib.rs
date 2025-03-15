use std::fs::File;
use std::io::{BufRead, BufReader};
use rayon::{array, prelude::*};

type Validators<'a> = &'a Vec<Box<dyn Fn(&str) -> Option<&str> + Sync + 'a>>;


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


pub fn check_csv (csv_filename: &str) -> Result<char, Box<dyn std::error::Error>> {

    let csv = std::fs::read_to_string(csv_filename)?;
    println!("CSV: {}", csv);
    let separator = infer_separator(&csv);
    println!("Separator: {}", separator);
    Ok(separator)
}


fn check_lines<'a>(lines: &'a[String], funcs: &'a Validators) {

    lines.par_iter().for_each(|line| {
        let current = Some(line.as_str());

        
        funcs
            .par_iter()
            .map(|f| 
                if let Some(s) = current {
                    f(s)
                } else {
                    None
                }
            )
            .collect::<Vec<_>>();

        if let Some(result) = current {
            println!("Processed: {}", result);
        } else {
            println!("Processing stopped for line: '{}'", line);
        }
    });
}


fn try_fix_lines<'a>(lines: &'a[String], funcs: &'a Validators) {
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




pub fn validate_csv(csv_filename: &str, validators: Validators, try_fix: bool) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(csv_filename)?;
    let reader = BufReader::new(file);

    let batch_size = 10_000;
    let mut buffer = Vec::with_capacity(batch_size);

    for line in reader.lines() {
        buffer.push(line?);

        if buffer.len() >= batch_size {
            if try_fix {
                try_fix_lines(&buffer, &validators);
            } else {
                check_lines(&buffer, &validators);
            }
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        if try_fix {
            try_fix_lines(&buffer, &validators);
        } else {
            check_lines(&buffer, &validators);
        }
    }

    Ok(())
}

pub fn main_validate(csv_filename: &str, num_fields: usize, try_fix: bool) -> Result<(), Box<dyn std::error::Error>> {

    let funcs: Vec<Box<dyn Fn(&str) -> Option<&str> + Sync>> = vec![
        Box::new(move |input| validate_line_field_count(input, num_fields, ',')),
        Box::new(move |input| validate_line_separator(input, ',')),
    ];

    validate_csv(csv_filename, &funcs, try_fix)

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
        let try_fix = false;
        let result = main_validate(csv_filename, 4, try_fix);
        assert!(result.is_ok());
    }

    #[test]
    fn test_main_try_fix_validate() {
        let csv_filename = "../../examples/with_header.csv";
        let try_fix = true;
        let result = main_validate(csv_filename, 4, try_fix);
        assert!(result.is_ok());
    }

}
