#![allow(dead_code)]
use csv::Reader;
use serde::Deserialize;
use std::path::Path;

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
/// The iterator that is produced by the [`ReturnsBucket`] struct.
#[derive(Debug)]
pub struct ReturnsBucketIter(CustomerReturn);

/**
A container of customer return records.

Iterate over this struct to reach the contained data.
*/
#[derive(Default, Debug)]
pub struct ReturnsBucket {
    vec: Vec<ReturnsBucketIter>,
}

impl ReturnsBucket {
    /// Creates a new [`ReturnsBucket`].
    pub fn new(vec: Vec<ReturnsBucketIter>) -> Self {
        Self { vec }
    }
    /// Push an item onto the [`ReturnsBucket`].
    fn push(&mut self, rb: ReturnsBucketIter) {
        self.vec.push(rb)
    }
    /**
    Creates a [`ReturnsBucket`] from a Customer Returns Csv.

    # Errors

    This function will error if it comes across any issue that may arise during
    general IO / CSV reading. See [`csv::Error`] as any [`std::io::Error`] will
    propagate through it.

    Whichever path is passed to this function is not tested for existence.
    */
    pub fn from_csv_path<P>(path: P) -> Result<Self, csv::Error>
    where
        P: AsRef<Path>,
    {
        let mut rb = ReturnsBucket::default();
        let mut rdr = Reader::from_path(path)?;
        for row in rdr.records() {
            let cr = CustomerReturn::from_csv_record(row?)?;
            let rbi = ReturnsBucketIter(cr);
            rb.push(rbi);
        }
        Ok(rb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static TEST_REMOVAL_SHIPMENT_RECORD: &str = "tests/data/CustomerReturns.csv";

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
    #[test]
    fn create_returns_bucket() {
        let rb = ReturnsBucket::from_csv_path(TEST_REMOVAL_SHIPMENT_RECORD);
        match rb {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        }
    }
}
