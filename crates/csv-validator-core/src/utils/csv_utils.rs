use std::collections::HashMap;

/// Use this function to infer the separator of a CSV file using statistical analysis,
/// based on the number of occurrences of the most common separators.
/// It will return the most likely separator.
///
/// # Example
///
/// ```
/// use csv_validator_core::utils::csv_utils::infer_separator;
///
/// let csv = "a,b,c\n1,2,3\n4,5,6";
/// let separator = infer_separator(csv);
/// assert_eq!(separator, ',');
/// ```
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
/// # Example
///
/// ```
/// use csv_validator_core::utils::csv_utils::infer_multi_char_separator;
///
/// let csv = "a,b,c\n1,2,3\n4,5,6";
/// let separator = infer_multi_char_separator(csv);
/// assert_eq!(separator, Some(",".into()));
/// ```
pub fn infer_multi_char_separator(sample: &str) -> Option<String> {
    let lines: Vec<&str> = sample.lines().collect();

    if lines.len() < 2 {
        return None;
    }

    let mut substr_freq: HashMap<&str, usize> = HashMap::new();

    // max sep length 4
    for line in &lines {
        for window_size in 1..=4 {
            for i in 0..=line.len().saturating_sub(window_size) {
                let substr = &line[i..i + window_size];
                *substr_freq.entry(substr).or_insert(0) += 1;
            }
        }
    }

    let mut candidates: Vec<&str> = substr_freq
        .iter()
        .filter(|&(_, &count)| count > 1)
        .map(|(&substr, _)| substr)
        .collect();

    candidates.sort_by_key(|b| std::cmp::Reverse(b.len()));

    for candidate in candidates {
        let counts: Vec<usize> = lines
            .iter()
            .map(|line| line.matches(candidate).count())
            .collect();
        if counts.windows(2).all(|w| w[0] == w[1] && w[0] > 0) {
            return Some(candidate.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

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
