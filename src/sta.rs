#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    gram_weight: u32,
    group: String,
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
    #[error("Row is declared as Loose with StagingGroup missing")]
    MissingGroup,
    #[error("Row is declared as Loose with UnitWeight missing")]
    MissingUnitWeight,
}
impl EntryParser {
    /// Determines an [`EntryFormat`], returning the built [`EntryFormat`].
    ///
    /// [`EntryFormat`] contains variants based on which pieces of information
    /// are available within the entry. Entries with "Loose" pack types will be
    /// built into [`EntryFormat::Loose`]. Entries with "Packed" pack types will
    /// be built into [`EntryFormat::Packed`].
    ///
    /// Currently there is no build path for the [`EntryFormat::Bare`] variant.
    /// Future implementations are planned to use this variant as a "planning"
    /// type, skipping over a few requirements in favor or ease of use.
    fn build(&self) -> Result<EntryFormat, ErrorKind> {
        // Check if Bare entry can be created
        self.check_bare_validity()?;

        let pack_type: &str = &self.pack_type.as_ref().unwrap();
        // Control flow determined by the declared type rather than
        // some other method. This could be problematic once other inputs
        // are considered for staging plans.
        let entry = match pack_type {
            // WARN: I don't know if this is case sensitive
            "Packed" => EntryFormat::Packed(self.build_packed()?),
            "Loose" => EntryFormat::Loose(self.build_loose()?),
            _ => Err(ErrorKind::InvalidPackType)?,
        };
        Ok(entry)
    }
    /// Build a [`LooseEntry`] from the [`EntryParser`]
    ///
    /// Passing an Entry without [`LooseEntry`] fields will cause the build
    /// to fail, returning a [`ErrorKind::MissingGroup`], or
    /// [`ErrorKind::MissingUnitWeight`] Error.
    fn build_loose(&self) -> Result<LooseEntry, ErrorKind> {
        // Check if the bare information is there
        self.check_bare_validity()?;

        let fnsku = self.fnsku.as_ref().unwrap();
        let group = self.staging_group.as_ref().unwrap();
        let weight = self.unit_weight.unwrap();

        // this converts the weight into grams, rounding up after conversion
        // entirely due to the fact that I would prefer to work with u32
        let gram_weight = (weight * 453.6).ceil() as u32;

        Ok(LooseEntry {
            id: self.id.unwrap(),
            fnsku: fnsku.to_string(),
            units: self.units.unwrap(),
            gram_weight,
            group: group.to_string(),
        })
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
        // Check if the bare information is there
        self.check_bare_validity()?;

        // A bunch of this should be extracted into a function
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
        let rdr = csv::Reader::from_path(&TEST_PLAN);
        let ep = rdr
            .unwrap()
            .into_records()
            .map(|x| x.unwrap())
            .map(|x| EntryParser::from_string_record(x).unwrap().build());
        ep.into_iter()
            .filter(|x| x.is_ok())
            .for_each(|x| println!("{:?}", x.unwrap()));
    }
}
