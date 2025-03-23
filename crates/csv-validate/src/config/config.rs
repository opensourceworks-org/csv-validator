use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ValidatorConfig {
    pub common: CommonConfig,
    pub validators: Vec<ValidatorSpec>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommonConfig {
    pub quote_char: char,
    pub separator: Option<String>,
    pub has_header: bool,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidatorSpec {
    IllegalChars {
        illegal_chars: Vec<String>,
        replace_with: Vec<String>,
        fix: bool,
        enabled: bool,
        common: CommonConfig
    },
    LineCount {
        min: Option<usize>,
        max: Option<usize>,
        enabled: bool,
        common: CommonConfig
    },
    FieldCount {
        expected: usize,
        enabled: bool,
        common: CommonConfig
    },
}

pub fn load_config(filename: &str) -> Result<ValidatorConfig, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);
    let config = serde_yaml::from_reader(reader)?;
    Ok(config)
}