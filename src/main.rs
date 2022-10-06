#![allow(dead_code)]
#![allow(unused_variables)]
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

fn main() {}
/// Container for `Entry`s.
#[derive(Debug, Serialize)]
struct Plan {
    entries: Vec<Entry>,
}
impl Plan {
    fn as_json_string(&self) {
        println!(
            "{:?}",
            serde_json::to_string(self).expect("failed to write json")
        );
    }
    /// Returns a reference to a `Plan`'s individual entries.
    ///
    /// `Plan` implements `Iterator` if ownership is needed.
    fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }
    /// Expands each [`Entry`] contained within the plan to mimic how the physical
    /// items will be once shipped
    ///
    /// An [`Entry`] that is described as 5 packed units with a case quantity
    /// of 1 will be expanded to 5 identical entries
    fn expand_entries(&mut self) {
        let new_entries = self
            .entries()
            .iter()
            .filter(|entry| entry.is_packed())
            .flat_map(|packed_entries| {
                (0..packed_entries.cases().unwrap_or(1))
                    .map(|_| packed_entries.clone())
                    .collect::<Self>()
            })
            .collect::<Self>();
        self.entries.retain(|entry| entry.is_loose());
        self.entries.extend(new_entries);
    }
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

    /// Returns the `usize` of the contained `Vec<Entry>`
    fn len(&self) -> usize {
        self.entries.len()
    }
    /// Counts the number of loose [`Entry`]
    fn loose_count(&self) -> usize {
        self.entries()
            .iter()
            .filter(|x| x.pack_type.is_loose())
            .count()
    }
    /// General constructor for a Vec containing `Entry`
    ///
    /// `Plan` has no implementation for `Default` due to a `Plan` being
    /// considered invalid when it contains no `Entry`.
    fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
    /// Counts the number of packed [`Entry`]
    fn packed_count(&self) -> usize {
        self.entries()
            .iter()
            .filter(|x| x.pack_type.is_packed())
            .count()
    }
    /// Mutates the [`Plan`] by sorting the individual [`Entry`]s
    ///
    /// Sorting order is as follows:
    /// 1. PackType
    /// 2. FNSKU (requires cloning)
    /// 3. Case Qt
    /// 4. Case Length
    /// 4. Case Width
    /// 4. Case Height
    ///
    ///
    fn sort_in_place(&mut self) {
        self.entries.sort_unstable_by_key(|entry| {
            (
                entry.is_loose(),
                entry.fnsku().to_owned(),
                entry.case_qt,
                entry.case_length,
                entry.case_width,
                entry.case_height,
            )
        });
    }
    fn summarize(&self) -> PlanSummary {
        PlanSummary::from_plan(self)
    }
    fn unique_fnskus(&self) -> HashSet<&str> {
        self.entries()
            .iter()
            .map(|x| x.fnsku.as_str())
            .collect::<HashSet<_>>()
    }
    fn valid_fnskus(&self) -> bool {
        self.unique_fnskus().iter().all(|x| x.chars().count() == 10)
    }
}
/// Describes various details of a `Plan`
#[derive(Debug)]
struct PlanSummary {
    skus: usize,
    entry_count: usize,
    valid_fnskus: bool,
    packed_count: usize,
    loose_count: usize,
}
impl PlanSummary {
    /// Returns general information regarding contained `Entry`
    fn from_plan(plan: &Plan) -> PlanSummary {
        PlanSummary {
            skus: plan.unique_fnskus().len(),
            entry_count: plan.len(),
            valid_fnskus: plan.valid_fnskus(),
            packed_count: plan.packed_count(),
            loose_count: plan.loose_count(),
        }
    }
}
impl Iterator for Plan {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }
}
impl FromIterator<Entry> for Plan {
    fn from_iter<I: IntoIterator<Item = Entry>>(iter: I) -> Self {
        let mut plan = Plan {
            entries: Vec::new(),
        };
        for i in iter {
            plan.entries.push(i)
        }
        plan
    }
}
#[derive(Deserialize, Debug, Clone, Serialize)]
/// Describes physical packaging for an `Entry'
enum PackConfig {
    Loose,
    Packed,
}
impl PackConfig {
    /// Returns `true` if the containing `PackType` is `PackType::Packed`
    fn is_packed(&self) -> bool {
        match self {
            PackConfig::Loose => false,
            PackConfig::Packed => true,
        }
    }
    /// Returns `true` if the containing `PackType` is `PackType::Loose`
    fn is_loose(&self) -> bool {
        match self {
            PackConfig::Loose => true,
            PackConfig::Packed => false,
        }
    }
}
/// A single line from a Plan.
///
/// Supports deserialization from the GoogleSheets csv
#[derive(Deserialize, Debug, Clone, Serialize)]
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
impl Entry {
    /// Returns `true` if the containing `PackType` is `PackType::Packed`
    fn is_packed(&self) -> bool {
        self.pack_type.is_packed()
    }
    /// Returns `true` if the containing `PackType` is `PackType::Loose`
    fn is_loose(&self) -> bool {
        self.pack_type.is_loose()
    }
    /// Returns the cases of this [`Entry`].
    fn cases(&self) -> Option<u32> {
        match self.case_qt {
            Some(x) => self.quantity.checked_div(x),
            _ => None,
        }
    }
}
impl<'a> Entry {
    fn fnsku(&'a self) -> &'a str {
        &self.fnsku
    }
}
