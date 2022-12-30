#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use crate::sta::entry::{Entry, EntryFormat, EntryParser};
use serde::Serialize;
use serde_json;

#[derive(Serialize, Debug)]
struct Plan {
    inner: Vec<EntryFormat>,
}

impl Plan {
    fn new(inner: Vec<EntryFormat>) -> Self {
        Self { inner }
    }
    fn to_buffer(&self) {
        // This should work, even if it is surprising
        let mut buffer: Vec<&EntryFormat> = Vec::new();
        self.inner.iter().map(|inner| {
            let pe_inner = match inner {
                EntryFormat::Packed(pe) => pe.as_expanded(),
                _ => None,
            };
            if let Some(expanded_inner) = pe_inner {
                let i = expanded_inner.range();
                for _ in 0..i {
                    buffer.push(inner)
                }
            }
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    static TEST_PLAN: &str = "tests/data/STAPlan.csv";

    #[test]
    fn deserialize_plan_csv() {
        let rdr = csv::Reader::from_path(&TEST_PLAN);
        let parsed_entries = rdr
            .unwrap()
            .into_records()
            .map(|x| x.unwrap())
            .map(|x| EntryParser::from_string_record(x).unwrap().build());
        let ok_entries = parsed_entries.filter_map(|x| x.ok()).collect::<Vec<_>>();
    }
}
