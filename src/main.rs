#![allow(dead_code)]
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

fn main() {
    let x = Plan::from_csv_path(&"test.csv");
    println!("{:?}", x);
}
/// Container for instances of `Entry`.
#[derive(Debug)]
struct Plan {
    entries: Vec<Entry>,
}
impl Plan {
    /// Constructor which wraps around `serde` for deserialization of CSV files.
    ///
    /// Returns None when there is no valid `Entry`, and when IO fails to reach
    /// the CSV file.
    fn from_csv_path<P>(path: P) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        // Check if path points to a file
        Path::try_exists(path.as_ref()).ok()?;

        // Call CSV reader with a referenced path
        let mut entry_vec: Vec<Entry> = csv::Reader::from_path(path)
            // Propagate IO / CSV error
            .ok()?
            .deserialize::<Entry>()
            // Remove instances where deserializtion fails
            .filter_map(|x| x.ok())
            .collect::<Vec<Entry>>();

        // Return None in cases where no entries are deserialized
        // TODO implement returning Err
        match entry_vec.is_empty() {
            true => None,
            false => {
                // Sort entries by FNSKU prior to initializing
                // I think this takes some overhead, but will save multiple
                // sorts in the future. This SHOULD be the same as sorting by msku
                entry_vec.sort_by(|x, y| x.fnsku.cmp(&y.fnsku));
                Some(Plan::new(entry_vec))
            }
        }
    }

    /// General constructor for a Vec containing `Entry`
    ///
    /// `Plan` has no implementation for `Default` due to a `Plan` being
    /// considered invalid when it contains no `Entry`.
    fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }

    /// Returns a reference to a `Plan`'s individual entries.
    ///
    /// `Plan` implements `Iterator` if ownership is needed.
    fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }
    /// Returns the `usize` of the contained `Vec<Entry>`
    fn len(&self) -> usize {
        self.entries.len()
    }
    fn summary<T>(&self) -> PlanSummary<T> {
        let fnsku_count = self
            .entries()
            .iter()
            .map(|x| &x.fnsku)
            .collect::<HashSet<_>>()
            .len();
        todo!()
    }
}
struct PlanSummary<T> {
    skus: u32,
    entry_count: u32,
    invalid_fnskus: Option<T>,
    packed_count: u32,
    loose_count: u32,
    parent_plan: Option<T>,
}
impl Iterator for Plan {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }
}
#[derive(Deserialize, Debug)]
enum PackConfig {
    Loose,
    Packed,
}
/// A single record from a Plan.
///
/// Supports deserialization from the GoogleSheets csv
#[derive(Deserialize, Debug)]
struct Entry {
    #[serde(alias = "Info")]
    id: u32,

    #[serde(alias = "FNSKU")]
    fnsku: String,

    #[serde(alias = "Quantity")]
    quantity: u32,

    #[serde(alias = "Pack Type")]
    pack_type: PackConfig,

    #[serde(alias = "Staging Group")]
    staging_group: Option<String>,

    #[serde(alias = "Unit Weight")]
    unit_weight: Option<u32>,

    #[serde(alias = "Case QT")]
    case_qt: Option<u32>,

    #[serde(alias = "Case Length")]
    case_length: Option<u32>,

    #[serde(alias = "Case Width")]
    case_width: Option<u32>,

    #[serde(alias = "Case Height")]
    case_height: Option<u32>,

    #[serde(alias = "Case Weight")]
    case_weight: Option<u32>,

    #[serde(alias = "Total Cases")]
    total_cases: Option<u32>,
}
