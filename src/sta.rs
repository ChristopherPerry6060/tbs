#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

/// Container for `Entry`s.
#[derive(Debug, Serialize)]
struct Plan {
    entries: Vec<Entry>,
}
impl Plan {
    /// Mutates the inner Entrys so that measurements decrease in the order of
    /// length, width, and height. Needed for normalizing dimensions among similar
    /// skus.
    fn sort_dimensions(&mut self) {
        // Borrow check railed me here, don't mess with it.
        for mut rec in &mut self.entries {
            let mut dims = vec![rec.case_length, rec.case_width, rec.case_height];

            if dims.iter().all(|x| x.is_some()) {
                dims.sort_unstable_by_key(move |x| x.unwrap() as u32);
                rec.case_length = dims.pop().unwrap();
                rec.case_width = dims.pop().unwrap();
                rec.case_height = dims.pop().unwrap();
            }
        }
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
                entry.fnsku().to_string(),
                entry.case_qt,
                { entry.case_length.map_or_else(|| 0, |l| l as u32) },
                { entry.case_width.map_or_else(|| 0, |l| l as u32) },
                { entry.case_height.map_or_else(|| 0, |l| l as u32) },
                { entry.case_weight.map_or_else(|| 0, |l| l as u32) },
            )
        });
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

impl Iterator for Plan {
    type Item = Entry;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }
}
impl FromIterator<Entry> for Plan {
    fn from_iter<I: IntoIterator<Item = Entry>>(iter: I) -> Self {
        Plan {
            entries: Vec::from_iter(iter),
        }
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
    unit_weight: Option<f32>,
    #[serde(alias = "Case QT")]
    case_qt: Option<u32>,
    #[serde(alias = "Case Length")]
    case_length: Option<f32>,
    #[serde(alias = "Case Width")]
    case_width: Option<f32>,
    #[serde(alias = "Case Height")]
    case_height: Option<f32>,
    #[serde(alias = "Case Weight")]
    case_weight: Option<f32>,
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
#[derive(Debug)]
struct Case {
    length: u32,
    width: u32,
    height: u32,
    weight: f32,
}
impl Case {
    fn from_sorted_dims(length: u32, width: u32, height: u32, weight: f32) -> Self {
        Case {
            length,
            width,
            height,
            weight,
        }
    }
}
#[derive(Debug)]
struct PackedEntry {
    id: u32,
    fnsku: String,
    units: u32,
    per_case: u32,
    case: Case,
}
#[derive(Debug)]
struct LooseEntry {
    id: u32,
    fnsku: String,
    units: u32,
    unit_weight: u32,
    group: u32,
}
/// The smallest amount of information required to describe an Entry
///
/// Offers it's utility when prototyping plans. The "unit-like" nature of [`self`]
/// can be easier compared to the more precise [`EntryFormat`] variants.
#[derive(Debug)]
struct BareEntry {
    id: u32,
    fnsku: String,
    units: u32,
}
#[derive(Debug)]
enum EntryFormat {
    Packed(PackedEntry),
    Loose(LooseEntry),
    Bare(BareEntry),
}
/// Parses individual records from a shipping plan
///
/// This currently supports reading from a Csv plan that originates from
/// the "GoogleDrive Shipping Plans".
#[derive(Deserialize, Debug, Serialize)]
struct EntryParser {
    #[serde(alias = "Info")]
    id: Option<u32>,
    #[serde(alias = "FNSKU")]
    fnsku: Option<String>,
    #[serde(alias = "Quantity")]
    units: Option<u32>,
    #[serde(alias = "Pack Type")]
    pack_type: Option<String>,
    #[serde(alias = "Staging Group")]
    staging_group: Option<String>,
    #[serde(alias = "Unit Weight")]
    unit_weight: Option<f32>,
    #[serde(alias = "Case QT")]
    case_qt: Option<u32>,
    #[serde(alias = "Case Length")]
    case_length: Option<f32>,
    #[serde(alias = "Case Width")]
    case_width: Option<f32>,
    #[serde(alias = "Case Height")]
    case_height: Option<f32>,
    #[serde(alias = "Case Weight")]
    case_weight: Option<f32>,
    #[serde(alias = "Total Cases")]
    total_cases: Option<u32>,
}
#[derive(Debug, Error)]
enum ErrorKind {
    #[error("Row is missing an Id")]
    MissingId,
    #[error("Row is missing an Fnsku")]
    MissingFnsku,
    #[error("Row is missing the PackType")]
    MissingPackType,
    #[error("Row is missing the unit quantity")]
    MissingUnits,
    #[error("Row is declared as packed with dimensions missing")]
    MissingPackedDimensions,
    #[error("Row is declared as packed with weight missing")]
    MissingPackedWeight,
    #[error("A PackType is included, but cannot be recognized")]
    InvalidPackType,
}
impl EntryParser {
    fn build(&self) -> Result<EntryFormat, ErrorKind> {
        self.check_bare_validity()?;
        let pack_type: &str = &self.pack_type.as_ref().unwrap();
        let entry = match pack_type {
            "Packed" => EntryFormat::Packed(self.build_packed()?),
            "Loose" => EntryFormat::Loose(self.build_loose()?),
            _ => Err(ErrorKind::InvalidPackType)?,
        };
        Ok(entry)
    }
    fn build_loose(&self) -> Result<LooseEntry, ErrorKind> {
        todo!();
    }
    /// Errors if [`Self`] lacks fields needed for building a [`BareEntry`].
    ///
    /// [EntryParser::check_bare_validity] can be utilized prior to creating the
    /// other [`EntryFormat`] structs. [`BareEntry`] fiels are the "bare minimum"
    /// needed to build an [`EntryFormat`]
    fn check_bare_validity(&self) -> Result<(), ErrorKind> {
        let Some(_id) =  self.id else {
            return Err(ErrorKind::MissingId)
        };
        let Some(_fnsku) = &self.fnsku else {
            return Err(ErrorKind::MissingFnsku)
        };
        let Some(_pack_type) = &self.pack_type else {
            return Err(ErrorKind::MissingPackType)
        };
        let Some(_units) = self.units else {
            return Err(ErrorKind::MissingUnits)
        };
        Ok(())
    }
    /// Build a [`PackedEntry`] from the [EntryParser]
    ///
    /// Passing an Entry without [`PackedEntry`] fields will cause the build
    /// to fail, returning a [`ErrorKind::MissingPackedWeight`], or
    /// [`ErrorKind::MissingPackedDimensions`] Error.
    fn build_packed(&self) -> Result<PackedEntry, ErrorKind> {
        let weight = self
            .case_weight
            .ok_or_else(|| ErrorKind::MissingPackedWeight)?;

        // Create a vec from the dimensions for iteration
        let dims = vec![self.case_length, self.case_height, self.case_weight];

        // Look if all dimensions are Some() > 0
        if !dims.iter().all(|dim| dim.is_some_and(|dim| dim > 0.0)) {
            return Err(ErrorKind::MissingPackedDimensions);
        };

        let dims_ref = &mut dims
            .iter()
            // Unwrap each dimension, round up, cast to u32
            .map(|x| x.unwrap().ceil() as u32)
            .collect::<Vec<u32>>();

        // Sort the dimensions so they can be popped off to l x w x h & weight
        dims_ref.sort_unstable();

        let case = Case::from_sorted_dims(
            // length
            dims_ref.pop().unwrap(),
            // width
            dims_ref.pop().unwrap(),
            // height
            dims_ref.pop().unwrap(),
            weight,
        );
        // Check if the bare information is there
        self.check_bare_validity()?;

        // I think there is a way to not clone the string
        // this works for now
        let fnsku = self.fnsku.as_ref().unwrap();

        Ok(PackedEntry {
            id: self.id.unwrap(),
            fnsku: fnsku.to_string(),
            units: self.units.unwrap(),
            per_case: self.case_qt.unwrap(),
            case,
        })
    }
    fn from_string_record(str_rec: csv::StringRecord) -> Result<EntryParser, csv::Error> {
        let header = csv::StringRecord::from(vec![
            "Info",
            "FNSKU",
            "Quantity",
            "Pack Type",
            "Staging Group",
            "Unit Weight",
            "Case QT",
            "Case Length",
            "Case Width",
            "Case Height",
            "Case Weight",
            "Total Cases",
        ]);
        str_rec.deserialize::<Self>(Some(&header))
    }
}
#[allow(unused_must_use)]
#[cfg(test)]
mod tests {
    use super::*;
    static TEST_PLAN: &str = "tests/data/STAPlan.csv";

    #[test]
    fn deserialize_plan_csv() {
        Plan::from_csv_path(TEST_PLAN).unwrap();
    }
    #[test]
    fn invalid_fnsku() {
        assert!(!Plan::from_csv_path(TEST_PLAN).unwrap().valid_fnskus());
    }
    #[test]
    fn sort_dimensions() {
        let mut plan = Plan::from_csv_path(TEST_PLAN).unwrap();
        plan.sort_dimensions();
        plan.into_iter().for_each(|rec| {
            let dimensions = vec![rec.case_height, rec.case_width, rec.case_length];
            if dimensions.iter().all(|x| x.is_some()) {
                assert!(dimensions
                    .into_iter()
                    .map(|x| x.unwrap() as u32)
                    .collect::<Vec<_>>()
                    .is_sorted());
            };
        });
    }
}
