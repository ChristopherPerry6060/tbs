#![allow(dead_code)]
use serde::{Deserialize, Serialize};

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
}

    }
}
