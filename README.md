# <img src="img/logo.svg" width="50" alt="CSV Validator Icon"> **Yet Another CSV Validator**
## Introduction

CSV-Validator is a csv validators combinator:  you can combine multiple simple validators.

### How it works:
- read a csv file into streaming buffers
- process lines from the buffers in parallel using rayon 
- apply validator functions on each line from a list of validator functions
- chain validators: each validator will either return the original line or None
- when None, the chaining stops

### Why chaining?
In case of an error in the source data, you can choose to let your validator fix the data and pass the fixed data
downstream.

## Goals

## Usage

## TODO

[x] maximize local parallelism using rayon
[ ] use a ray cluster for further parallelizing
[ ] stream vs batch processing (ie. from kafka)
[ ] implement kafka as sink for ie. spark streaming processing