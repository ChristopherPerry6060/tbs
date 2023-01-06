#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use crate::sta::entry::Entry;
use crate::sta::result::Result;
use serde::Serialize;
use serde_json;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Default)]
struct Plan {
    entries: Vec<Entry>,
}

impl Plan {
    fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
    fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }
}
#[derive(Debug, Default)]
struct PlanBuilder {
    entries: Vec<Result<Entry>>,
    keep_error: bool,
}

impl PlanBuilder {
    fn push(&mut self, e: Result<Entry>) {
        self.entries.push(e)
    }
    fn from_csv_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut pb = Self::default();
        let csv_reader = csv::Reader::from_path(path)?;
        for wrapped_record in csv_reader.into_records() {
            let record = wrapped_record?;
            pb.push(Entry::from_csv_record(record));
        }
        Ok(pb)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn import_csv_to_plan_builder() {
        static TEST_PLAN: &str = "tests/data/STAPlan.csv";
        let p = PlanBuilder::from_csv_path(TEST_PLAN).unwrap();
        dbg!(&p);
    }
}
