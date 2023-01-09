#![allow(dead_code)]
#![allow(unused_imports)]
use csv::Reader;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct CustomerReturn {
    #[serde(alias = "return-date")]
    return_date: String,
    #[serde(alias = "order-id")]
    order_id: String,
    #[serde(alias = "sku")]
    msku: String,
    #[serde(alias = "asin")]
    asin: String,
    #[serde(alias = "fnsku")]
    fnsku: String,
    #[serde(alias = "product-name")]
    description: String,
    #[serde(alias = "quantity")]
    units: u32,
    #[serde(alias = "fulfillment-center-id")]
    fc_id: String,
    #[serde(alias = "detailed-disposition")]
    disposition: String,
    #[serde(alias = "reason")]
    reason: String,
    #[serde(alias = "status")]
    status: String,
    #[serde(alias = "license-plate-number")]
    lpn: String,
    #[serde(alias = "customer-comments")]
    customer_comments: Option<String>,
}
impl CustomerReturn {
    fn from_csv_record(csv_record: csv::StringRecord) -> Result<Self, csv::Error> {
        let hdr = vec![
            "return-date",
            "order-id",
            "sku",
            "asin",
            "fnsku",
            "product-name",
            "quantity",
            "fulfillment-center-id",
            "detailed-disposition",
            "reason",
            "status",
            "license-plate-number",
            "customer-comments",
        ];
        let hdr_str = csv::StringRecord::from(hdr);
        csv_record.deserialize(Some(&hdr_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn load_customer_return_csv_report() -> Vec<CustomerReturn> {
        static TEST_REMOVAL_SHIPMENT_RECORD: &str = "tests/data/CustomerReturns.csv";
        let rdr = Reader::from_path(TEST_REMOVAL_SHIPMENT_RECORD).unwrap();
        rdr.into_records()
            .filter_map(|wrapped_row| {
                let Ok(row) = wrapped_row else {
                return None
            };
                CustomerReturn::from_csv_record(row).ok()
            })
            .collect::<Vec<_>>()
    }
    #[test]
    fn load_customer_return_csv() {
        assert!(!load_customer_return_csv_report().is_empty());
    }
}
