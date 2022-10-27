use csv;
use serde::Deserialize;
use std::path::Path;

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
fn find_tracking_in_removal<P>(tracking: &str, path: P) -> Option<RemovalShipment>
where
    P: AsRef<Path>,
{
    csv::Reader::from_path(path)
        .ok()?
        .deserialize::<RemovalShipment>()
        .filter_map(|x| x.ok())
        .find(|x| x.match_tracking(&tracking))
}
#[test]
fn find_match() {
    find_tracking_in_removal(
        "1Z77YA874237990210",
        "C:\\Users\\Chris\\Downloads\\635656019284.csv",
    )
    .unwrap();
}
