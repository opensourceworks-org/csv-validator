# <img src="img/logo.svg" width="50" alt="CSV Validator Icon"> **Yet Another CSV Validators Combinator**

## TODO!

- [ ] move test validators from cli crate to core crate
- [ ] add more tests

## Introduction
CSV-Validate is a CLI interface to a csv validators combinator: combine multiple simple validators.

The main goal is to validate csv data in a streaming fashion, with the ability to fix the data on the fly.
Errors are reported with their location.

## Usage
You can either define the validators in a config file or pass them as CLI arguments.
When passed at the command line, you're limited to one validator per run.  This allows to chain multiple validators in a shell script, and even add some custom logic in between with ie. awk, sed, jq, ...

On the other hand, when using a config file, you can define multiple validators at once, and run them all in one go, improving performance.

CLI arguments:

```bash
csv-validate ../../tools/output.csv illegal-chars --char "tv=____NO___TV_________"
```

with config file:

```yaml
# config.yaml
common:
  quote_char: '"'
  separator: ';'
  has_header: true

validators:
  - type: illegal_chars
    illegal_chars: ['!', '?', '@', 'tv']
    replace_with: ['_', '.', '-', '!!_________NO____________TV___________!!']
    fix: true
    enabled: true
    common:
      quote_char: '"'
      separator: ';'
      has_header: true

  - type: field_count
    expected: 50
    enabled: true
    common:
      quote_char: '"'
      separator: ';'
      has_header: true

```
