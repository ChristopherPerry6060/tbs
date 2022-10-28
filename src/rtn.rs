use csv;
use serde::Deserialize;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

#[derive(Deserialize, Debug)]
struct RemovalShipment {
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
impl RemovalShipment {
    fn match_tracking(&self, tracking: &str) -> bool {
        self.tracking
            // tracking column can contain multiple tracking numbers
            .split(',')
            .map(|x| x.trim())
            .find(|x| x.eq_ignore_ascii_case(tracking))
            .is_some()
    }
    fn match_fnsku(&self, fnsku: &str) -> bool {
        self.fnsku.eq_ignore_ascii_case(fnsku)
    }
}
struct RemovalShipmentReport {
    path: PathBuf,
}
impl RemovalShipmentReport {
    fn new(path: OsString) -> Self {
        RemovalShipmentReport {
            path: PathBuf::from(path),
        }
    }
    fn find_tracking(&self, tracking: &str) -> Option<RemovalShipment> {
        csv::Reader::from_path(&self.path)
            .ok()?
            .deserialize::<RemovalShipment>()
            .filter_map(|x| x.ok())
            .find(|x| x.match_tracking(&tracking))
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
        assert!(RemovalShipmentReport::new(path)
            .find_tracking(AMZL_TRACKING_TEST)
            .is_some())
    }
}
