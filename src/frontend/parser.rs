extern crate csv;

use csv::Reader;
use regex::Regex;
use crate::frontend::table::{Table, Attribute};
use std::io::{Error};

pub fn load_from_file(filename: &str) -> Result<Table, Error> {
    let mut reader = Reader::from_path(filename)?;

    let header = reader.headers()?;

    let re = Regex::new(r"(?P<n>).csv").unwrap();
    let table_name = re.replace(filename, "$n").to_string();

    let mut metadata = Vec::new();
    let mut rank = 0;
    for meta in header {
        metadata.push(Attribute::new(rank, meta.to_string()));
        rank += 1;
    }

    let mut table = Table::new(table_name, metadata);

    for record in reader.records() {
        let record = record?;
        let data = record.iter().map(|s| -> String { s.to_string() }).collect::<Vec<String>>();
        table.add_entry(data);
    }

    Ok(table)
}