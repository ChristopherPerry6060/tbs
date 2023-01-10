#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
/**
Csv **Rem**oval **Ship**ment Parser

Helper for dealing with Amazon's Removal Shipment reports.
This structure accounts for a single row within the report.
*/
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
struct CsvRemShipParser {
    #[serde(alias = "carrier")]
    carrier: String,
    #[serde(alias = "disposition")]
    disposition: String,
    #[serde(alias = "fnsku")]
    fnsku: String,
    #[serde(alias = "sku")]
    merchant_sku: String,
    #[serde(alias = "order-id")]
    order_id: String,
    #[serde(alias = "removal-order-type")]
    removal_type: String,
    #[serde(alias = "request-date")]
    request_date: String,
    #[serde(alias = "shipment-date")]
    shipment_date: String,
    #[serde(alias = "shipped-quantity")]
    shipped_quantity: u32,
    #[serde(alias = "tracking-number")]
    tracking: String,
}
impl CsvRemShipParser {
    fn from_csv_record(csv_record: csv::StringRecord) -> Result<Self, csv::Error> {
        let hdr = vec![
            "request-date",
            "order-id",
            "shipment-date",
            "sku",
            "fnsku",
            "disposition",
            "shipped-quantity",
            "carrier",
            "tracking-number",
            "removal-order-type",
        ];
        let hdr_str = csv::StringRecord::from(hdr);
        csv_record.deserialize(Some(&hdr_str))
    }
    /**
    Splits tracking by '`,`'. Returning the entire string if there is no '`,`'

    This function will also run `trim` on each resulting string.
    */
    fn split_tracking_numbers(&self) -> Vec<&str> {
        let tracking = &self.tracking;
        tracking
            .split(',')
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|tracking| tracking.trim())
            .collect::<Vec<_>>()
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use csv::Reader;
    fn load_rem_shipment_report_csv() -> Vec<CsvRemShipParser> {
        static TEST_REMOVAL_SHIPMENT_RECORD: &str = "tests/data/RemovalShipments.csv";
        let rdr = Reader::from_path(TEST_REMOVAL_SHIPMENT_RECORD).unwrap();
        rdr.into_records()
            .filter_map(|wrapped_row| {
                let Ok(row) = wrapped_row else {
                return None
            };
                CsvRemShipParser::from_csv_record(row).ok()
            })
            .collect::<Vec<CsvRemShipParser>>()
    }
    #[test]
    fn load_removal_shipment_csv() {
        static TEST_REMOVAL_SHIPMENT_RECORD: &str = "tests/data/RemovalShipments.csv";
        let rdr = Reader::from_path(TEST_REMOVAL_SHIPMENT_RECORD).unwrap();
        for item in rdr.into_records() {
            let Ok(row) = item else {
                continue;
            };
            match CsvRemShipParser::from_csv_record(row) {
                Ok(_) => assert!(true),
                Err(_) => assert!(false),
            };
        }
    }
    #[test]
    fn split_tracking_numbers() {
        let vrp = load_rem_shipment_report_csv();
        for i in vrp.iter() {
            let splits = i.split_tracking_numbers();
            assert!(!splits.is_empty());
        }
    }
}
