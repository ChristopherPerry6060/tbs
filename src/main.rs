#![allow(dead_code)]
#![allow(unused_variables)]
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

fn main() {}
/// Container for `Entry`s.
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
    /// Returns general information regarding contained `Entry`
    ///
    /// No immediate use implemented at the moment beyond QOL.
    /// TODO This should be extracted to `PlanSummary::new()` impl
    fn summarize(&self) -> PlanSummary {
        let skus = self.entries().len();

        // Collect FNSKUs into a set, removing duplicates
        let unique_fnsku = self
            .entries()
            .iter()
            .map(|x| &x.fnsku)
            .collect::<HashSet<_>>();

        // Collect id into a set, and count
        let entry_count = self
            .entries()
            .iter()
            .map(|x| &x.id)
            .collect::<HashSet<_>>()
            .len();

        // Count unique FNSKU
        let fnsku_count = unique_fnsku.len();

        // Check if each FNSKU is of valid length (10 chars)
        // TODO this should be returning on Option containing invalid_skus
        // or a similiar implementation
        let valid_fnsku = unique_fnsku.iter().all(|x| x.chars().count() == 10);

        let loose_count = self
            .entries()
            .iter()
            .filter(|x| x.pack_type.is_loose())
            .count();

        let packed_count = self
            .entries()
            .iter()
            .filter(|x| x.pack_type.is_packed())
            .count();

        PlanSummary::new(
            skus,
            entry_count,
            valid_fnsku,
            // TODO implement ParentName for the plan that is derived from
            // the origianl deserde source
            packed_count,
            loose_count,
        )
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
    /// `PlanSummary` constructor
    fn new(
        skus: usize,
        entry_count: usize,
        valid_fnskus: bool,
        packed_count: usize,
        loose_count: usize,
    ) -> Self {
        PlanSummary {
            skus,
            entry_count,
            valid_fnskus,
            packed_count,
            loose_count,
        }
    }
    /// Convenience method that calls `summarize()` from the caller.
    ///
    /// Function identically to Plan::summarize
    fn summary(plan: Plan) -> Self {
        plan.summarize()
    }
}
impl Iterator for Plan {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }
}
#[derive(Deserialize, Debug)]
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
#[derive(Debug)]
/// A `Vec` containing a reference to the underlying `Entry` for each
/// expected physical case.
///
/// See `from_entry` method for details on implementation.
struct ExpandedEntry<'a> {
    cont: Vec<&'a Entry>,
}
impl<'a> ExpandedEntry<'a> {
    /// Creates a reference to the `Entry` for each expected physical case.
    ///
    /// If `Entry` contains `Packconfig::Loose`, the resulting expansion occurs
    /// a single time. Otherwise, expansion occurs
    /// `Entry.quantity.checked_div(Entry.case_qt)`. If the division fails,
    /// expansion occurs a single time.
    fn from_entry(entry: &'a Entry) -> Self {
        (0..entry.cases()).map(|_| entry).collect::<Self>()
    }
}
// For convenience in accessing the underlying iterator
impl<'a> Iterator for ExpandedEntry<'a> {
    type Item = &'a Entry;
    fn next(&mut self) -> Option<Self::Item> {
        self.cont.pop()
    }
}
// For convenience in using collect::<Self>() in from_entry
impl<'a> FromIterator<&'a Entry> for ExpandedEntry<'a> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Entry>,
    {
        ExpandedEntry {
            cont: Vec::from_iter(iter),
        }
    }
}
/// A single line from a Plan.
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
impl Entry {
    /// Returns `true` if the containing `PackType` is `PackType::Packed`
    fn is_packed(&self) -> bool {
        self.pack_type.is_packed()
    }
    /// Returns `true` if the containing `PackType` is `PackType::Loose`
    fn is_loose(&self) -> bool {
        self.pack_type.is_packed()
    }
    fn cases(&self) -> u32 {
        // This is a very messy way to implement this
        // TODO Return a result type instead of u32
        match self.case_qt {
            Some(x) => self.quantity.checked_div(x).unwrap_or(1),
            _ => 1,
        }
    }
    /// Creates a reference to the `Entry` for each expected physical case.
    /// Wraps `ExpandedEntry::from_entry`.
    ///
    /// If `Entry` contains `Packconfig::Loose`, the resulting expansion occurs
    /// a single time. Otherwise, expansion occurs
    /// `Entry.quantity.checked_div(Entry.case_qt)`. If the division fails,
    /// expansion occurs a single time.
    ///
    /// TODO try to extract this into a trait
    fn as_expanded(&self) -> ExpandedEntry {
        ExpandedEntry::from_entry(self)
    }
}
