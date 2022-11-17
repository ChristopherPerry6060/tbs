#![allow(dead_code)]
use serde::Deserialize;
use std::{collections::HashSet, path::Path};

/// A single record from a RemovalShipmentReport
#[derive(Deserialize)]
struct RemShipmentRecord {
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

impl RemShipmentRecord {
    /// Returns a reference of the underlying FNSKU
    fn fnsku(&self) -> &str {
        self.fnsku.as_ref()
    }
    /// Returns true if the contained FNSKU matches
    fn match_fnsku(&self, fnsku: &str) -> bool {
        self.fnsku.eq_ignore_ascii_case(fnsku)
    }
    /// Returns true if any of the contained tracking matches the input
    fn match_tracking(&self, tracking: &str) -> bool {
        if self.tracking().contains(',') {
            self.tracking()
                .split(',')
                .any(|x| x.eq_ignore_ascii_case(tracking))
        } else {
            self.tracking().eq_ignore_ascii_case(tracking)
        }
    }
    /// Returns a reference to the underlying tracking
    fn tracking(&self) -> &str {
        self.tracking.as_ref()
    }
}
/// A container for RemovalShipmentRecord
type RemShipmentReport = Vec<RemShipmentRecord>;

impl RemovalShipment for RemShipmentReport {
    /// Returns any unique tracking numbers within the report
    ///
    /// This *DOES NOT* split records containing multiple tracking numbers.
    fn unique_tracking(&self) -> Vec<&str> {
        self.iter()
            .map(|x| x.tracking())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    }
    /// Constructs a `RemovalShipmentReport` from a path
    fn from_path<P>(path: P) -> Result<Self, csv::Error>
    where
        P: AsRef<Path>,
    {
        Ok(csv::Reader::from_path(path)?
            .deserialize::<RemShipmentRecord>()
            .filter_map(|records| records.ok())
            .collect())
    }
    /// Returns any contained records that match the given tracking
    /// This *DOES NOT* split records containing multiple tracking numbers.
    fn filter_by_tracking(self, tracking: &str) -> RemShipmentReport {
        self.into_iter()
            .filter(|records| records.match_tracking(tracking))
            .collect()
    }
}

/// The interface for dealing with RemovalShipments
trait RemovalShipment {
    fn unique_tracking(&self) -> Vec<&str>;

    fn from_path<P>(p: P) -> Result<RemShipmentReport, csv::Error>
    where
        P: AsRef<Path>;

    fn filter_by_tracking(self, tracking: &str) -> RemShipmentReport;
}

#[cfg(test)]
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
        assert!(!RemShipmentReport::from_path(REMOVAL_SHIPMENT_TEST_PATH)
            .unwrap()
            .filter_by_tracking(&AMZL_TRACKING_TEST)
            .is_empty());
    }
    #[test]
    fn match_ups_tracking() {
        assert_eq!(
            RemShipmentReport::from_path(REMOVAL_SHIPMENT_TEST_PATH)
                .unwrap()
                .filter_by_tracking(&UPS_TRACKING_TEST)
                .is_empty(),
            false
        );
    }
    #[test]
    fn unique_tracking_numbers() {
        assert_eq!(
            RemShipmentReport::from_path(REMOVAL_SHIPMENT_TEST_PATH)
                .unwrap()
                .unique_tracking()
                .is_empty(),
            false
        );
    }
    #[test]
    // use `cargo test -- --nocapture' to see output
    fn list_unique_tracking_numbers() {
        let report = RemShipmentReport::from_path(REMOVAL_SHIPMENT_TEST_PATH).unwrap();
        dbg!(report.unique_tracking());
    }
}
