


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

pub fn check_csv (csv_filename: &str) -> Result<char, Box<dyn std::error::Error>> {
    
    let csv = std::fs::read_to_string(csv_filename).unwrap();
    println!("CSV: {}", csv);
    let separator = infer_separator(&csv);
    println!("Separator: {}", separator);
    Ok(separator)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_validate_csv() {
        let csv_filename = "../../examples/with_header.csv";
        let result = check_csv(csv_filename);
        assert_eq!(result.unwrap(), ',');

        let csv_filename = "../../examples/full_quoted_with_header_semicolon.csv";
        let result = check_csv(csv_filename);
        assert_eq!(result.unwrap(), ';');
    }
}
