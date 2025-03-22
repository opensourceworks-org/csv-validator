use csv_validate::Validator;
use csv_validate::{process_input, validator_a, validator_b};
use divan::black_box;
use divan::Bencher;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

fn main() {
    let filename = "../../tools/output.csv";
    let outfilename = "../../tools/test_output.csv";

    // Optional: Measure memory before (requires jemalloc)
    #[cfg(feature = "jemalloc")]
    let mem_before = jemalloc_ctl::stats::allocated::read().unwrap();

    let start = Instant::now();

    let file = File::open(filename).expect("Unable to open input file");
    let reader: Box<dyn BufRead> = Box::new(BufReader::new(file));
    let output_file = File::create(outfilename).expect("Unable to create output file");
    let mut writer: Box<dyn Write> = Box::new(BufWriter::new(output_file));

    let mut available_validators: HashMap<&str, Box<Validator>> = HashMap::new();
    available_validators.insert("validator_a", Box::new(validator_a));
    available_validators.insert("validator_b", Box::new(validator_b));

    let mut validators = Vec::new();
    validators.push((
        "validator_a".to_string(),
        available_validators.get("validator_a").unwrap(),
        true,
    ));

    process_input(black_box(reader), &black_box(validators), black_box(writer));

    let duration = start.elapsed();
    println!("Duration: {:.2?}", duration);

    #[cfg(feature = "jemalloc")]
    {
        let mem_after = jemalloc_ctl::stats::allocated::read().unwrap();
        println!("Memory used: {} bytes", mem_after - mem_before);
    }
}
