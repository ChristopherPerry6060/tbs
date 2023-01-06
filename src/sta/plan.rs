#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use crate::sta::entry::Entry;
use crate::sta::result::Result;
use anyhow::anyhow;
use serde::Serialize;
use serde_json;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Default)]
struct Plan {
    entries: Vec<Entry>,
}

impl Plan {
    /// Creates a new [`Plan`].
    fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
    /// Push an [`Entry`] into the [`Plan`].
    fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
    }
    /**
    Sorts the [`Plan`] in-place.

    Sort order
    * Pack type
    * FNSKU
    * Case Length
    * Case Width
    * Case Height
    * Case Weight
    * Group Name

    */
    fn sort(&mut self) {
        self.entries.sort_unstable_by_key(|entry| {
            (
                entry.is_loose(),
                entry.get_fnsku().to_string(),
                entry.try_case_length(),
                entry.try_case_width(),
                entry.try_case_height(),
                entry.try_case_gram_weight(),
                entry.try_group_name().unwrap_or_default().to_string(),
            )
        });
    }
}
#[derive(Debug, Default)]
/**
Convenient builder for a [`Plan`].

Comes with various default options, all of which can be changed prior to
building.

Options:
* `keep_error`: default `false`
    * Discards all errors
*/
struct PlanBuilder {
    entries: Vec<Result<Entry>>,
    keep_error: bool,
}

impl PlanBuilder {
    /**
    Push a `Result<Entry>` to the plan

    The builder holds `Result` wrapped entries to have the control over
    which options are discarded prior to building.
    */
    fn push(&mut self, e: Result<Entry>) {
        self.entries.push(e)
    }

    /**
    Construct a [`Plan`] from a path that points to a CSV.

    # Errors

    This function will return an error if the CSV format is incorrect, or
    deserialization fails to return a valid entry.
    */
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
    /**
    Consume the [`PlanBuilder`] and return the generate [`Plan`].

    # Errors

    This function will return an error if the resulting [`Plan`] is empty once
    all of the errors are removed.
    */
    fn build(mut self) -> std::result::Result<Plan, anyhow::Error> {
        if self.keep_error {
            self.remove_entries_without_fnskus();
        };

        let entry_vec = self
            .entries
            .into_iter()
            .filter_map(|x| x.ok())
            .collect::<Vec<Entry>>();
        let plan = Plan::new(entry_vec);
        if plan.entries.is_empty() {
            Err(anyhow!("Plan was built, but it is empty."))
        } else {
            Ok(plan)
        }
    }
    /// Remove any [`Entry`] that is missing FNSKUs.
    fn remove_entries_without_fnskus(&mut self) {
        use crate::sta::result::ErrorKind; // TODO get rid of this
        self.entries.drain_filter(|x| {
            x.as_ref()
                .is_err_and(|x| matches!(x, ErrorKind::MissingFnsku))
        });
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn import_csv_to_plan_builder() {
        static TEST_PLAN: &str = "tests/data/STAPlan.csv";
        PlanBuilder::from_csv_path(TEST_PLAN).unwrap();
    }
    #[test]
    fn build_and_remove_blank_fnsku() {
        static TEST_PLAN: &str = "tests/data/STAPlan.csv";
        let p = PlanBuilder::from_csv_path(TEST_PLAN).unwrap();
        let builds = p.build();
        dbg!(&builds);
    }
}
