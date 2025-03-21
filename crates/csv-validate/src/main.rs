// use csv_validator_core::validators::issue::ValidationResult;
// use csv_validator_core::validators::line_validators::validate_line_separator;
// use csv_validator_core::validators::line_validators::validate_line_field_count;

use std::collections::HashMap;
use std::env;
use atty::Stream;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use clap::Parser;

#[derive(Debug)]
struct ValidationResult {
    message: String,
}


type Validator = dyn Fn(ValidationResult, usize) -> ValidationResult + Send + Sync;
/// Type alias for a list of validators along with their fix flag.
type Validators = Vec<(Box<Validator>, bool)>;

/// Example validator: appends a message indicating it was applied.
fn validator_a(result: ValidationResult, row: usize) -> ValidationResult {
    let mut new_msg = result.message;
    new_msg.push_str(&format!(" [validator_a applied at row {}]", row));
    ValidationResult { message: new_msg }
}

/// Another example validator.
fn validator_b(result: ValidationResult, row: usize) -> ValidationResult {
    let mut new_msg = result.message;
    new_msg.push_str(&format!(" [validator_b applied at row {}]", row));
    ValidationResult { message: new_msg }
}


#[derive(Debug)]
struct ValidatorSpec {
    name: String,
    fix: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Optional input filename.
    #[arg(value_name = "FILE")]
    filename: Option<String>,

    /// Trailing arguments for specifying validators and their options.
    ///
    /// Example:
    ///
    ///   --validator validator_a --fix --validator validator_b
    ///
    /// Note: pass these trailing arguments after a `--` separator.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    validator_args: Vec<String>,
}

fn main() {
    let args = Args::parse();

    // Manually parse the trailing validator arguments.
    let mut validator_specs: Vec<ValidatorSpec> = Vec::new();
    let mut i = 0;
    while i < args.validator_args.len() {
        match args.validator_args[i].as_str() {
            "--validator" => {
                if i + 1 < args.validator_args.len() {
                    let name = args.validator_args[i + 1].clone();
                    let mut fix = false;
                    i += 2;
                    // If the next argument is "--fix", enable fix mode for this validator.
                    if i < args.validator_args.len() && args.validator_args[i] == "--fix" {
                        fix = true;
                        i += 1;
                    }
                    validator_specs.push(ValidatorSpec { name, fix });
                } else {
                    eprintln!("Expected a validator name after --validator");
                    return;
                }
            }
            other => {
                eprintln!("Unrecognized argument: {}", other);
                i += 1;
            }
        }
    }

    // Create a map of available validators.
    let mut available_validators: HashMap<&str, Box<Validator>> = HashMap::new();
    available_validators.insert("validator_a", Box::new(validator_a));
    available_validators.insert("validator_b", Box::new(validator_b));

    // Select the validators based on the parsed specifications.
    let mut selected_validators = Vec::new();
    for spec in validator_specs {
        if let Some(validator) = available_validators.get(spec.name.as_str()) {
            selected_validators.push((spec.name.clone(), validator.clone(), spec.fix));
        } else {
            eprintln!("Validator '{}' is not available", spec.name);
        }
    }
    // Determine if there's piped input.
    let stdin_is_piped = !atty::is(Stream::Stdin);
    if args.filename.is_none() && !stdin_is_piped {
        eprintln!("No input provided. Please pipe data or provide a filename.");
        std::process::exit(1);
    }

    if let Some(filename) = args.filename {
        // Use input from a file.
        let file = File::open(filename).expect("Unable to open file");
        let reader = BufReader::new(file);
        process_input(reader, &selected_validators);
    } else if stdin_is_piped {
        // Use piped input from stdin.
        let stdin = io::stdin();
        let reader = stdin.lock();
        process_input(reader, &selected_validators);
    } else {
        eprintln!("No input provided. Please pipe data or provide a filename.");
    }
}

fn process_input<R: BufRead>(reader: R, validators: &Vec<(String, &Box<Validator>, bool)>) {
    for line in reader.lines() {
        match line {
            Ok(text) => {
                for (validator_name, validator, fix) in validators {
                    println!("Validating: {}", text);
                    println!("Validator: {:?}", validator_name);
                    println!("Fix: {:?}", fix);

                    // Apply any processing logic based on the --fix flag.
                    if *fix {
                        println!("Fixed: {}", text);
                    } else {
                        println!("{}", text);
                    }

                }

            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
}
