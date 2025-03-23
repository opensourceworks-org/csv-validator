
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
};
use std::io::Write;
use std::ops::Deref;
use aho_corasick::AhoCorasick;
use clap::{Parser, ArgGroup, Subcommand, Args};
use serde::Deserialize;
use serde_yaml::Value;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
pub mod config;

use config::config::{load_config, CommonConfig, ValidatorSpec};
use log::error;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct Replacement {
    pub pattern: String,
    pub replace_with: Option<String>,
}


#[derive(Debug)]
struct ValidationResult {
    original: String,
    fixed: String,
    modified: bool,
    message: String,
}

trait Validator: Send + Sync {
    fn validate(&self, input: &str, row: usize) -> ValidationResult;
    fn finalize(&self) {}
    fn should_fix(&self) -> bool {
        false
    }
}


#[derive(Parser, Debug)]
#[command(author, version, about)]
// #[command(group(ArgGroup::new("mode").required(true).args(&["config", "validator"])))]
pub struct Cli {
    #[arg(long)]
    config: Option<String>,

    /// Specify a validator to run in streaming mode
    #[command(subcommand)]
    validator: Option<ValidatorCmd>,

    /// Input file (or use '-' or omit for stdin)
    #[arg(value_name = "FILE")]
    filename: Option<String>,

    #[arg(long)]
    output: Option<String>,

    #[arg(long, value_parser = parse_char_replacement)]
    char: Vec<Replacement>,

    #[arg(long, default_value_t = false)]
    report: bool,

    #[arg(long, global = true)]
    pub separator: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct CommonArgs {
    /// Field separator character (default: ',')
    #[arg(long, default_value = ",")]
    pub separator: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum ValidatorCmd {
    IllegalChars {
        /// e.g. '@=_', '?=.', 'x' (removes if no =)
        #[arg(long, value_parser = parse_char_replacement)]
        char: Vec<Replacement>,

        #[arg(long, default_value_t = false)]
        fix: bool,

        #[command(flatten)]
        common: CommonArgs,
    },

    FieldCount {
        #[arg(long)]
        expected: usize,

        #[command(flatten)]
        common: CommonArgs,
    },
}


impl From<&CommonArgs> for CommonConfig {
    fn from(args: &CommonArgs) -> Self {
        CommonConfig {
            separator: args.clone().separator,
            quote_char: '"',
            has_header: false,

        }
    }
}

type ValidatorFactory = Box<dyn Fn(Value) -> Box<dyn Validator>>;

fn build_registry() -> HashMap<String, ValidatorFactory> {
    let mut reg: HashMap<String, ValidatorFactory> = HashMap::new();

    reg.insert("illegal_chars".into(), Box::new(|args| {
        let cfg: IllegalCharsConfig = serde_yaml::from_value(args).unwrap();
        Box::new(IllegalChars::new(cfg))
    }));

    reg.insert("field_count".into(), Box::new(|args| {
        let cfg: FieldCountConfig = serde_yaml::from_value(args).unwrap();
        Box::new(FieldCount::new(cfg))
    }));

    reg
}


#[derive(Debug, Deserialize)]
struct IllegalCharsConfig {
    illegal_chars: Vec<String>,
    replace_with: Vec<String>,
    fix: bool,
    common: CommonConfig
}

struct IllegalChars {
    cfg: IllegalCharsConfig,
    pub matcher: AhoCorasick,
}

impl IllegalChars {
    fn new(cfg: IllegalCharsConfig) -> Self {
        let matcher = AhoCorasick::new(&cfg.illegal_chars).expect("Failed to build matcher");
        Self { cfg, matcher }
    }
}

// first attempt, slow
// impl Validator for IllegalChars {
//     fn validate(&mut self, input: &str, row: usize) -> ValidationResult {
//         let mut fixed = input.to_string();
//         let mut modified = false;
//         let mut message = String::new();
//
//
//         for (i, c) in self.cfg.illegal_chars.iter().enumerate() {
//             let positions: Vec<usize> = fixed.match_indices(c.as_str()).map(|(i, _)| i).collect();
//             if positions.is_empty() {
//
//                 continue;
//             } else {
//                 modified = true;
//                 let string_positions = positions.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ");
//                 message.push_str(&format!("Illegal char |-> {} <-| found on line {} at positions: {}\n", c, row, string_positions));
//                 if !self.cfg.fix {
//                     modified = true;
//                     continue;
//                 }
//                 modified = true;
//                 let rep = self.cfg.replace_with.get(i).cloned().unwrap_or_default();
//                 fixed = fixed.replace(c.as_str(), &rep.to_string());
//             }
//
//
//             // a lot faster if we don't care about positions and number of occurrences
//             // if fixed.contains(c.as_str()) {
//             //     //println!("Illegal char found!: {}", c);
//             //     if !self.cfg.fix {
//             //         modified = true;
//             //         continue;
//             //     }
//             //     modified = true;
//             //     let rep = self.cfg.replace_with.get(i).cloned().unwrap_or_default();
//             //     fixed = fixed.replace(c.as_str(), &rep.to_string());
//             // }
//         }
//
//         ValidationResult {
//             original: input.to_string(),
//             fixed: fixed.clone(),
//             modified,
//             message,
//         }
//     }
// }


// second attempt using Aho-Corasick, faster for more patterns, but possibly slower for few patterns
impl Validator for IllegalChars {
    fn validate(&self, input: &str, row: usize) -> ValidationResult {
        let mut fixed = input.to_string();
        let mut modified = false;
        let mut message = String::new();

        let mut pattern_matches: Vec<Vec<usize>> = vec![vec![]; self.cfg.illegal_chars.len()];
        for mat in self.matcher.find_iter(input) {
            pattern_matches[mat.pattern()].push(mat.start());
        }

        for (i, positions) in pattern_matches.into_iter().enumerate() {
            if positions.is_empty() {
                continue;
            }

            modified = true;
            let pattern = &self.cfg.illegal_chars[i];
            let string_positions = positions
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            message.push_str(&format!(
                "Illegal char found on row {} at positions: |-> {} <-|:{}\n",
                row, pattern, string_positions
            ));

            if self.cfg.fix {
                let rep = self.cfg.replace_with.get(i).cloned().unwrap_or_default();
                fixed = fixed.replace(pattern, &rep);
            }
        }

        ValidationResult {
            original: input.to_string(),
            fixed,
            modified,
            message,
        }
    }

    fn should_fix(&self) -> bool {
        self.cfg.fix
    }
}

#[derive(Debug, Deserialize)]
struct FieldCountConfig {
    expected: usize,
    common: CommonConfig,
}

struct FieldCount {
    cfg: FieldCountConfig,
}

impl FieldCount {
    fn new(cfg: FieldCountConfig) -> Self {
        Self { cfg }
    }
}

impl Validator for FieldCount {
    fn validate(&self, input: &str, _row: usize) -> ValidationResult {
        let sep = self.cfg.common.separator.as_deref().unwrap_or(",");
        let actual = input.split(sep).count();
        let mismatch = actual != self.cfg.expected;

        ValidationResult {
            original: input.to_string(),
            fixed: input.to_string(),
            modified: false,
            message: if mismatch {
                format!("Expected {}, found {}", self.cfg.expected, actual)
            } else {
                String::new()
            },
        }
    }
    fn should_fix(&self) -> bool {
       false
    }
}



pub fn parse_char_replacement(s: &str) -> Result<Replacement, String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    let pattern = parts[0].to_string();

    if pattern.is_empty() {
        return Err("Pattern must not be empty".into());
    }

    let replace_with = if parts.len() == 2 {
        Some(parts[1].to_string())
    } else {
        None
    };

    Ok(Replacement {
        pattern,
        replace_with,
    })
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let report = args.report;

    // TODO: move this to core!
    let registry = build_registry();

    let stdin_is_piped = !atty::is(atty::Stream::Stdin);

    let reader: Box<dyn BufRead> = match &args.filename {
        Some(path) if path != "-" => {
            let file = File::open(path)?;
            Box::new(BufReader::new(file))
        }
        Some(_) | None if stdin_is_piped => {
            Box::new(BufReader::new(io::stdin()))
        }
        _ => {
            eprintln!("Error: No input provided. Please provide a filename or pipe stdin.");
            std::process::exit(1);
        }
    };

    let mut writer: Box<dyn Write> = match &args.output {
        Some(path) if path != "-" => {
            let file = File::create(path)?;
            Box::new(io::BufWriter::new(file))
        }
        _ => Box::new(io::BufWriter::new(io::stdout())),
    };


    let mut validators: Vec<Box<dyn Validator>> = Vec::new();



    match (&args.config, &args.validator) {
        (Some(cfg_path), None) => {
            // let file = File::open(cfg_path)?;
            // let config: ValidatorConfig = serde_yaml::from_reader(file)?;

            let config = load_config(cfg_path)?;


            for spec in config.validators.into_iter().filter(|v| match v {
                ValidatorSpec::IllegalChars { enabled, common, .. } => *enabled,
                ValidatorSpec::FieldCount { enabled, common, .. } => *enabled,
                ValidatorSpec::LineCount { .. } => todo!(),
            }) {
                match spec {
                    ValidatorSpec::IllegalChars {
                        illegal_chars,
                        replace_with,
                        fix,
                        common,
                        ..
                    } => {
                        validators.push(Box::new(IllegalChars::new(IllegalCharsConfig {
                            illegal_chars,
                            replace_with,
                            fix,
                            common,
                        })));
                    }
                    ValidatorSpec::FieldCount { expected, common, .. } => {
                        validators.push(Box::new(FieldCount::new(FieldCountConfig { expected, common })));
                    }
                   ValidatorSpec::LineCount { .. } => todo!(),
                }
            }
        }
        (None, Some(ValidatorCmd::IllegalChars { char, fix, common, .. })) => {

            let (illegal_chars, replace_with): (Vec<_>, Vec<_>) = char
                .into_iter()
                .map(|r| (r.clone().pattern, r.clone().replace_with.unwrap_or_default()))
                .unzip();            let fix = *fix;
            let common = common.into();
            validators.push(Box::new(IllegalChars::new(IllegalCharsConfig {
                illegal_chars,
                replace_with,
                fix,
                common,
            })));
        }

        (None, Some(ValidatorCmd::FieldCount { expected, common, .. })) => {
            let expected = *expected;
            let common = common.into();
            validators.push(Box::new(FieldCount::new(FieldCountConfig { expected, common })));
        }

        _ => unreachable!("Clap guarantees one mode"),
    }


    process_input(reader, &mut validators, &mut writer, report)?;

    Ok(())
}

fn print_report(messages: &[String]) {
    if messages.is_empty() {
        return;
    }

    eprintln!("\nErrors:");
    for msg in messages {
        eprintln!("  {}", msg);
    }
}

// replaced this with a parallel version
//
// fn process_input<R: BufRead, W: Write>(
//     reader: R,
//     validators: &mut [Box<dyn Validator>],
//     writer: &mut W,
// ) -> std::io::Result<()> {
//     let mut error_messages: Vec<String> = Vec::with_capacity(1000);
//     for (row, line) in reader.lines().enumerate() {
//         let line = line?;
//
//         let mut result = ValidationResult {
//             original: line.clone(),
//             fixed: line.clone(),
//             modified: false,
//             message: String::new(),
//         };
//
//         for v in validators.iter_mut() {
//             let updated = v.validate(&result.fixed, row + 1);
//             result.fixed = updated.fixed;
//             result.modified |= updated.modified;
//             if !updated.message.is_empty() {
//                 result.message.push_str(&updated.message);
//                 result.message.push(' ');
//                 error_messages.push(updated.message)
//             }
//         }
//
//         if result.modified {
//             writeln!(writer, "{}", result.fixed)?;
//         } else if !result.message.trim().is_empty() {
//             writeln!(writer, "-> {}", result.message.trim())?;
//         } else {
//             writeln!(writer, "{}", result.original)?;
//         }
//
//
//     }
//
//     writer.flush()?;
//     print_report(&error_messages);
//     Ok(())
// }
//
fn process_input<R: BufRead, W: Write>(
    reader: R,
    validators: &[Box<dyn Validator>],
    writer: &mut W,
    report: bool,
) -> std::io::Result<()> {
    let fix_enabled = validators.iter().any(|v| v.should_fix());

    let lines: Vec<_> = reader.lines().collect::<Result<_, _>>()?;

    let error_messages = Arc::new(Mutex::new(Vec::with_capacity(1000)));

    let results: Vec<String> = lines
        .into_par_iter()    // this preserves the order of the input
        .enumerate()
        .map(|(row, line)| {
            let mut result = ValidationResult {
                original: line.clone(),
                fixed: line.clone(),
                modified: false,
                message: String::new(),
            };

            if fix_enabled {
                for v in validators.iter() {
                    let updated = v.validate(&result.fixed, row + 1);
                    result.fixed = updated.fixed;
                    result.modified |= updated.modified;
                    if !updated.message.is_empty() {
                        result.message.push_str(&updated.message);
                        result.message.push(' ');
                        error_messages.lock().unwrap().push(updated.message);
                    }
                }
            } else {
                let updates: Vec<_> = validators
                    .par_iter()
                    .map(|v| v.validate(&result.fixed, row + 1))
                    .collect();
                for updated in updates {
                    result.modified |= updated.modified;
                    if !updated.message.is_empty() {
                        result.message.push_str(&updated.message);
                        result.message.push(' ');
                        error_messages.lock().unwrap().push(updated.message);
                    }
                }
            }

            if result.modified {
                result.fixed
            } else if !result.message.trim().is_empty() {
                format!("-> {}", result.message.trim())
            } else {
                result.original
            }
        })
        .collect();

    for line in results {
        writeln!(writer, "{}", line)?;
    }

    writer.flush()?;
    if report {
        print_report(&error_messages.lock().unwrap());
    }
    Ok(())
}
