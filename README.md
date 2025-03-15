# <img src="img/logo.svg" width="50" alt="CSV Validator Icon"> **Yet Another CSV Validators Combinator**
## Introduction

CSV-Validator is a csv validators combinator: combine multiple simple validators.

### How it works:
- read csv data from file or stream into stream buffers
- process lines from the buffers in parallel
- apply validator functions on each line from a list of validator functions
- chain validators: each validator will either return the original line (or fixed line) or None
- when None, the chaining stops

### Why chaining and not parallel?
In case of an error in the source data, you can choose to let your validator fix the data and pass the fixed data
downstream. In that case, the validators run chained.  When exclusively checking and not fixing, then the validators will run concurrently.

## Goals

## Usage

## TODO

- [x] maximize local parallelism using rayon
- [ ] heuristic vs statistical analysis for separator
- [ ] use a ray cluster for further parallelizing
- [ ] stream vs batch processing (ie. from kafka)
- [ ] implement kafka as sink for ie. spark streaming processing
- [ ] implement python interface
- [ ] add web example using wasm (data stays local)
