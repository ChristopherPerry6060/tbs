#![allow(dead_code)]
use crate::sta::result::{ErrorKind, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Copy)]
struct Case {
    length: u32,
    width: u32,
    height: u32,
    gram_weight: u32,
}
impl Case {
    fn from_sorted_dims(length: u32, width: u32, height: u32, weight: f32) -> Self {
        Case {
            length,
            width,
            height,
            gram_weight: weight as u32,
        }
    }
}
/**
A single "packed" record from a shipping plan.

Packed represents __box(es)__ that will contain a single Sku.
*/
#[derive(Debug, Serialize, Clone)]
pub struct Packed {
    id: u32,
    fnsku: String,
    units: u32,
    per_case: u32,
    case: Case,
}
impl EntryFormat for Packed {
    fn get_fnsku(&self) -> &str {
        &self.fnsku
    }

    fn get_units(&self) -> u32 {
        self.units
    }

    fn get_weight(&self) -> u32 {
        self.case.gram_weight
    }

    fn try_num_of_cases(&self) -> Option<u32> {
        self.units.checked_div_euclid(self.per_case)
    }

    fn try_case_dimensions(&self) -> Option<Case> {
        Some(self.case)
    }

    fn try_group_name(&self) -> Option<&str> {
        None
    }

    fn try_per_case(&self) -> Option<u32> {
        Some(self.per_case)
    }
}
/**
A single "loose" record from a shipping plan.

Loose represents items that will likely be packed with different items.
*/
#[derive(Debug, Serialize, Clone)]
pub struct Loose {
    id: u32,
    fnsku: String,
    units: u32,
    gram_weight: u32,
    group: String,
}
impl EntryFormat for Loose {
    fn get_fnsku(&self) -> &str {
        &self.fnsku
    }

    fn get_units(&self) -> u32 {
        self.units
    }

    fn get_weight(&self) -> u32 {
        self.gram_weight
    }

    fn try_num_of_cases(&self) -> Option<u32> {
        Some(1)
    }

    fn try_case_dimensions(&self) -> Option<Case> {
        None
    }

    fn try_group_name(&self) -> Option<&str> {
        Some(&self.group)
    }

    fn try_per_case(&self) -> Option<u32> {
        None
    }
}
trait EntryFormat {
    fn get_fnsku(&self) -> &str;
    fn get_units(&self) -> u32;
    fn get_weight(&self) -> u32;
    fn try_num_of_cases(&self) -> Option<u32>;
    fn try_case_dimensions(&self) -> Option<Case>;
    fn try_group_name(&self) -> Option<&str>;
    fn try_per_case(&self) -> Option<u32>;
}

/**
Various representations of a single row on a "shipping plan".

* [`Entry::Loose`] represents an item destined for a mixed box.
* [`Entry::Packed`] represent an item destined for a case packing
*/
#[derive(Debug, Serialize, Clone)]
pub enum Entry {
    Loose(Loose),
    Packed(Packed),
}
impl Entry {
    /// Returns a reference to the contained FNSKU string of this [`Entry`].
    pub fn get_fnsku(&self) -> &str {
        match self {
            Entry::Loose(l) => l.get_fnsku(),
            Entry::Packed(p) => p.get_fnsku(),
        }
    }
    /**
    Returns a reference to the contained group name string of this [`Entry`].

    This function will always return `None` If the entry is [`Self::Packed`].
    */
    pub fn try_group_name(&self) -> Option<&str> {
        match self {
            Entry::Loose(inner) => inner.try_group_name(),
            Entry::Packed(inner) => inner.try_group_name(),
        }
    }
    /**
    Returns the contained case length of the [`Entry`].

    This function will always return `None` If the entry is [`Self::Loose`].
    */
    pub fn try_case_length(&self) -> Option<u32> {
        Some(match self {
            Entry::Loose(inner) => inner.try_case_dimensions()?.length,
            Entry::Packed(inner) => inner.try_case_dimensions()?.length,
        })
    }
    /**
    Returns the contained case width of the [`Entry`].

    This function will always return `None` If the entry is [`Self::Loose`].
    */
    pub fn try_case_width(&self) -> Option<u32> {
        Some(match self {
            Entry::Loose(inner) => inner.try_case_dimensions()?.width,
            Entry::Packed(inner) => inner.try_case_dimensions()?.width,
        })
    }
    /**
    Returns the contained case height of the [`Entry`].

    This function will always return `None` If the entry is [`Self::Loose`].
    */
    pub fn try_case_height(&self) -> Option<u32> {
        Some(match self {
            Entry::Loose(inner) => inner.try_case_dimensions()?.height,
            Entry::Packed(inner) => inner.try_case_dimensions()?.height,
        })
    }
    /**
    Returns the contained case weight (in grams) of the [`Entry`].

    This function will always return `None` If the entry is [`Self::Loose`].
    */
    pub fn try_case_gram_weight(&self) -> Option<u32> {
        Some(match self {
            Entry::Loose(inner) => inner.try_case_dimensions()?.gram_weight,
            Entry::Packed(inner) => inner.try_case_dimensions()?.gram_weight,
        })
    }
    /**
    Attemps to build an [`Entry`] from a single CSV record.

    # Errors

    This function will return an error if the record is unable to be
    deserialized into an Entry.
    */
    pub fn from_csv_record(str_rec: csv::StringRecord) -> Result<Self> {
        EntryParser::from_string_record(str_rec)?.build()
    }
    /**
    Returns the num of cases of this [`Entry`].

    # Errors

    This function will return an error if
    * Total `units` is not evenly divisible by `per_case`.
    * Total `units == 0`.
    * `per_case == 0`.

    */
    fn num_of_cases(&self) -> Result<u32> {
        // Destructure if Packed
        let Entry::Packed(packed_entry) = self else {
            // Return 1 if Loose
            return Ok(1);
        };
        if !is_evenly_packed(packed_entry) {
            return Err(ErrorKind::NonDivisibleCaseQt);
        };
        let case_qt = packed_entry.per_case;
        let total_qt = packed_entry.units;
        let Some(range) =  total_qt.checked_div_euclid(case_qt) else {
            return Err(ErrorKind::NonDivisibleCaseQt);
        };
        Ok(range)
    }
    /**
    Returns `true` if the [`Entry`] is [`Packed`].
    */
    pub fn is_packed(&self) -> bool {
        matches!(self, Entry::Packed(_))
    }
    /**
    Returns `true` if the [`Entry`] is [`Loose`].
    */
    pub fn is_loose(&self) -> bool {
        matches!(self, Entry::Loose(_))
    }
}

/// Returns `true` if `p.units / p.per_case` has a remainder `== 0`.
fn is_evenly_packed(p: &Packed) -> bool {
    let units = &p.units;
    units
        .checked_rem(p.per_case)
        // check the unit quantity is evenly divisible by case quantity
        .is_some_and(|remainder| remainder == 0)
}
/**
A helper for dealing with input strings, mostly CSV shipping plans.

You should be using [`Entry::from_csv_record`] instead.

Holds parsing and deserialization logic for reading in csv plan records.
use [`EntryParser::from_string_record`] to load a [`csv::StringRecord`],
then call [`EntryParser::build`] to build.
*/
#[derive(Deserialize, Debug)]
pub struct EntryParser {
    #[serde(alias = "Info")]
    id: Option<u32>,
    #[serde(alias = "FNSKU")]
    fnsku: Option<String>,
    #[serde(alias = "Quantity")]
    #[serde(alias = "Total Quantity")]
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

impl EntryParser {
    /// Attempt to build an [`Entry`] from an [`EntryParser`]
    fn build(&self) -> Result<Entry> {
        // Check if Bare entry can be created
        self.check_bare_validity()?;

        match &self.pack_type {
            Some(pt) if pt == "Packed" => Ok(Entry::Packed(self.build_packed()?)),
            Some(pt) if pt == "Loose" => Ok(Entry::Loose(self.build_loose()?)),
            _ => Err(ErrorKind::InvalidPackType)?,
        }
    }
    /**
    Build a [`Entry::Loose`] from the [`EntryParser`]

    Passing an Entry without [`Loose`] fields will cause the build
    to fail, returning a [`ErrorKind::MissingGroup`], or
    [`ErrorKind::MissingUnitWeight`] Error.
    */
    fn build_loose(&self) -> Result<Loose> {
        // Check if the bare information is there
        self.check_bare_validity()?;

        let fnsku = self.fnsku.as_ref().unwrap();
        let group = self.staging_group.as_ref().unwrap();
        let weight = self.unit_weight.unwrap();

        // this converts the weight into grams, rounding up after conversion
        // entirely due to the fact that I would prefer to work with u32
        let gram_weight = (weight * 453.6).ceil() as u32;

        Ok(Loose {
            id: self.id.unwrap(),
            fnsku: fnsku.to_string(),
            units: self.units.unwrap(),
            gram_weight,
            group: group.to_string(),
        })
    }
    // Checks to see if essential components are present to build the entry
    fn check_bare_validity(&self) -> Result<()> {
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
    /**
    Build a [`PackedEntry`] from the [EntryParser]

    Passing an Entry without [`PackedEntry`] fields will cause the build
    to fail, returning a [`ErrorKind::MissingPackedWeight`], or
    [`ErrorKind::MissingPackedDimensions`] Error.
    */
    fn build_packed(&self) -> Result<Packed> {
        // Check if the bare information is there
        self.check_bare_validity()?;

        // Check if the CaseQt is missing
        let Some(case_qt) = self.case_qt else {
            return Err(ErrorKind::MissingCaseQt)
        };
        if case_qt == 0 {
            return Err(ErrorKind::MissingCaseQt);
        };

        // Div the total units by the CaseQt
        self.units
            // This is fine to do since it will just cause the following method
            // to return None anyways.
            .unwrap_or(0)
            .checked_rem(case_qt)
            .ok_or(ErrorKind::NonDivisibleCaseQt)?;

        let weight = self.case_weight.ok_or(ErrorKind::MissingPackedWeight)?;

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

        dims_ref.sort_unstable();

        let case = Case::from_sorted_dims(
            // length
            dims_ref.pop().unwrap(),
            // width
            dims_ref.pop().unwrap(),
            // height
            dims_ref.pop().unwrap(),
            weight * 453.6,
        );

        let fnsku = self.fnsku.as_ref().unwrap();

        Ok(Packed {
            id: self.id.unwrap(),
            fnsku: fnsku.to_string(),
            units: self.units.unwrap(),
            per_case: self.case_qt.unwrap(),
            case,
        })
    }
    pub fn from_string_record(str_rec: csv::StringRecord) -> Result<EntryParser> {
        let binding = csv::StringRecord::from(vec![
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
        let hdr = Some(&binding);
        Ok(str_rec.deserialize::<Self>(hdr)?)
    }
}
#[allow(unused_must_use)]
#[cfg(test)]
mod tests {

    use super::*;

    static TEST_PLAN: &str = "tests/data/STAPlan.csv";

    fn isolate_ok_entries() -> Result<Vec<Entry>> {
        let rdr = csv::Reader::from_path(TEST_PLAN);
        let parsed_entries = rdr?
            .into_records()
            .map(|x| x.unwrap())
            .map(|x| EntryParser::from_string_record(x).unwrap().build());
        Ok(parsed_entries
            .filter_map(|x| x.ok())
            .collect::<Vec<Entry>>())
    }
    #[test]
    fn check_ranges() {
        let expect = vec![5, 1, 5, 1, 1, 1, 1, 1, 1, 1, 1, 3];
        let ok_entries = isolate_ok_entries().unwrap();
        let results: Vec<u32> = ok_entries
            .iter()
            .map(|entry_formats| entry_formats.num_of_cases().unwrap())
            .collect();
        assert_eq!(expect, results);
    }
}
