use serde::Deserialize;
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

/// A single record from a RemovalShipmentReport
#[derive(Deserialize, Debug)]
struct RemovalShipmentRecord {
    #[serde(alias = "request-date")]
    request_date: String,
    #[serde(alias = "order-id")]
    order_id: String,
    #[serde(alias = "shipment-date")]
    shipment_date: String,
    #[serde(alias = "sku")]
    msku: String,
    #[serde(alias = "fnsku")]
    fnsku: String,
    #[serde(alias = "shipped-quantity")]
    shipped_quantity: u32,
    #[serde(alias = "carrier")]
    carrier: String,
    #[serde(alias = "tracking-number")]
    tracking: String,
}
impl RemovalShipmentRecord {
    /// Returns a reference of the underlying FNSKU
    fn fnsku(&self) -> &str {
        self.fnsku.as_ref()
    }
    /// Returns true if the contained FNSKU matches
    fn match_fnsku(&self, fnsku: &str) -> bool {
        self.fnsku.eq_ignore_ascii_case(fnsku)
    }
    /// Returns true if the tracking matches what is supplied
    ///
    /// WARN: This does nothing with removal records that have
    /// multiple tracking records separated by a comma
    /// FIXME: This cannot be used to check for a single tracking
    /// if record is one of these "multi tracking" versions
    fn match_tracking(&self, tracking: &str) -> bool {
        self.tracking().eq_ignore_ascii_case(tracking)
    }
    /// Returns a reference to the underlying tracking
    fn tracking(&self) -> &str {
        self.tracking.as_ref()
    }
}
/// A container for RemovalShipmentRecord
///
/// Working with RemovalShipmentReport can be more intuitive than individual
/// records. Reports are sourced from Amazon's fulfillment reports, and utilized
/// as a simple csv "database".
struct RemovalShipmentReport {
    records: Vec<RemovalShipmentRecord>,
}
impl RemovalShipmentReport {
    /// Returns a reference to the underlying records
    fn records(&self) -> &Vec<RemovalShipmentRecord> {
        &self.records
    }
    /// Returns any unique tracking numbers within the report
    ///
    /// This *DOES NOT* split records containing multiple tracking numbers.
    fn unique_tracking_numbers(&self) -> Vec<&str> {
        self.records()
            .iter()
            .map(|x| x.tracking())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    }
    /// Constructs a `RemovalShipmentReport` from a path
    /// TODO: This should take a generic argument
    fn from_path(path: OsString) -> Result<Self, csv::Error> {
        let records = csv::Reader::from_path(path)?
            .deserialize::<RemovalShipmentRecord>()
            .filter_map(|x| x.ok())
            .collect::<Vec<_>>();
        Ok(RemovalShipmentReport { records })
    }
    /// Returns any contained records that match the given tracking
    /// This *DOES NOT* split records containing multiple tracking numbers.
    fn records_matching_tracking(&self, tracking: &str) -> Vec<&RemovalShipmentRecord> {
        let values = self
            .records()
            .iter()
            .filter(|records| records.match_tracking(tracking))
            .collect::<Vec<&RemovalShipmentRecord>>();
        values
    }
}
impl<'a> RemovalShipmentReport {
    fn as_removal_sheet(&'a self) -> RemovalSheets<'a> {
        RemovalSheets::from_removal_shipment_report(self)
    }
}
#[derive(Debug)]
struct RemovalSheets<'a>(Vec<RemovalSheetsIter<'a>>);

impl<'a> RemovalSheets<'a> {
    fn from_removal_shipment_report(rsr: &'a RemovalShipmentReport) -> Self {
        rsr.unique_tracking_numbers()
            .into_iter()
            .map(move |tracking| {
                RemovalSheetsIter::from_removal_record(
                    tracking,
                    rsr.records_matching_tracking(tracking),
                )
            })
            .collect()
    }
}
impl<'a> RemovalSheets<'a> {
    fn from_path<P>(path: P) -> Result<(), csv::Error>
    where
        P: Into<OsString>,
    {
        RemovalShipmentReport::from_path(path.into())?.as_removal_sheet();
        Ok(())
    }
}

impl<'a> Iterator for RemovalSheets<'a> {
    type Item = RemovalSheetsIter<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}
impl<'a> FromIterator<RemovalSheetsIter<'a>> for RemovalSheets<'a> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = RemovalSheetsIter<'a>>,
    {
        RemovalSheets(Vec::from_iter(iter))
    }
}

#[derive(Debug)]
struct RemovalSheetsIter<'a> {
    trackings: &'a str,
    records: Vec<&'a RemovalShipmentRecord>,
}

impl<'a> RemovalSheetsIter<'a> {
    fn from_removal_record(trackings: &'a str, records: Vec<&'a RemovalShipmentRecord>) -> Self {
        RemovalSheetsIter { trackings, records }
    }
}

mod tests {
    use super::*;

    static AMZL_TRACKING_TEST: &str = "TBA303300920917";
    static UPS_TRACKING_TEST: &str = "1Z55E7R34231089600";
    static REMOVAL_SHIPMENT_TEST_PATH: &str = ".\\tests\\data\\RemovalShipment.csv";

    #[test]
    fn csv_path() {
        assert!(Path::new(REMOVAL_SHIPMENT_TEST_PATH).exists())
    }
    #[test]
    fn match_amzl_tracking() {
        let path = OsString::from(REMOVAL_SHIPMENT_TEST_PATH);
        assert_eq!(
            RemovalShipmentReport::from_path(path)
                .unwrap()
                .records_matching_tracking(&AMZL_TRACKING_TEST)
                .is_empty(),
            false
        );
    }
    #[test]
    fn match_ups_tracking() {
        let path = OsString::from(REMOVAL_SHIPMENT_TEST_PATH);
        assert_eq!(
            RemovalShipmentReport::from_path(path)
                .unwrap()
                .records_matching_tracking(&UPS_TRACKING_TEST)
                .is_empty(),
            false
        );
    }
    #[test]
    fn unique_tracking_numbers() {
        let path = OsString::from(REMOVAL_SHIPMENT_TEST_PATH);
        assert_eq!(
            RemovalShipmentReport::from_path(path)
                .unwrap()
                .unique_tracking_numbers()
                .is_empty(),
            false
        );
    }
    #[test]
    // use `cargo test -- --nocapture' to see output
    fn list_unique_tracking_numbers() {
        let path = OsString::from(REMOVAL_SHIPMENT_TEST_PATH);
        let report = RemovalShipmentReport::from_path(path).unwrap();
        dbg!(report.as_removal_sheet());
    }
}
