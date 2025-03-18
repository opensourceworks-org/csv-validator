use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::char,
};

/// Parses a quoted field.
/// The separator is passed as a parameter even though it isn’t used inside quotes.
fn parse_quoted_field<'a>(
    _separator: &'a str,
    quote: char,
) -> impl FnMut(&'a str) -> IResult<&'a str, String> {
    move |input: &'a str| {
        let (mut input, _) = char(quote)(input)?;
        let mut output = String::new();

        loop {
            // Consume until the next quote.
            let (i, part) = take_until(quote.to_string().as_str())(input)?;
            output.push_str(part);
            input = i;

            // Consume the quote.
            let (i, _) = char(quote)(input)?;
            input = i;

            // If the next character is also a quote, this is an escaped quote.
            if input.starts_with(quote) {
                let (i, _) = char(quote)(input)?;
                output.push(quote);
                input = i;
            } else {
                // End of quoted field.
                break;
            }
        }
        Ok((input, output))
    }
}

/// Parses an unquoted field, stopping when the custom separator is found.
fn parse_unquoted_field<'a>(separator: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, String> {
    move |input: &'a str| {
        // Find the first occurrence of the separator.
        if let Some(pos) = input.find(separator) {
            let (field, rest) = input.split_at(pos);
            Ok((rest, field.to_string()))
        } else {
            // If no separator is found, return the rest of the input.
            Ok(("", input.to_string()))
        }
    }
}

/// Tries to parse a field as either a quoted or unquoted field.
/// Both sub-parsers receive the custom separator.
fn parse_field<'a>(
    separator: &'a str,
    quote: char,
) -> impl Fn(&'a str) -> IResult<&'a str, String> {
    move |input: &'a str| {
        alt((
            parse_quoted_field(separator, quote),
            parse_unquoted_field(separator),
        ))
        .parse(input)
    }
}

/// testing custom separator
/// # Example
///
/// ```
/// use csv_validator_core::utils::csv_utils::line_processor;
///
/// let line = r#"field1$$$"field$$$2"$$$"field3 with ""quo$$$ted"" text"$$$field4"#;
/// let separator = "$$$";
/// let result = line_processor(line, separator, '"');
/// assert_eq!(result.unwrap().len(), 4);
/// ```
pub fn line_processor<'a>(
    line: &'a str,
    separator: &'a str,
    quote_char: char,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // can't use separated_list0 because of custom (multiline) separator
    let mut remaining = line;
    let mut fields = Vec::new();

    while !remaining.is_empty() {
        match parse_field(separator, quote_char)(remaining) {
            Ok((rest, field)) => {
                fields.push(field);
                remaining = rest;
                // if the separator is present at the start of the remainder, consume it.
                if let Ok((rest_after_sep, _)) =
                    tag::<&str, &str, nom::error::Error<&str>>(separator)(remaining)
                {
                    remaining = rest_after_sep;
                } else {
                    // no separator found; we are at the end.
                    break;
                }
            }
            Err(err) => {
                eprintln!("Error parsing field: {:?}", err);
                break;
            }
        }
    }

    // Output parsed fields.
    for (i, field) in fields.iter().enumerate() {
        println!("Field {}: {}", i + 1, field);
    }
    dbg!(&fields);
    Ok(fields)
}

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

pub fn infer_separator_from_file(filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    // read first 5 lines of the file
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().take(5).collect::<Result<Vec<_>, _>>()?;
    dbg!(&lines);

    let sample = lines.join("\n");
    let separator = infer_multi_char_separator(&sample);

    match separator {
        Some(sep) => Ok(sep),
        None => Err("Separator not found".into()),
    }
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
/// todo: use csv crate to parse csv
///     - quote character
///     - escape character
///     - comment character
///     - header
///     - flexible with (quoted) separator
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
