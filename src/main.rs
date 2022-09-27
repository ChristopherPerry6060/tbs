#![allow(dead_code)]
use csv::Reader;
use serde::Deserialize;
use std::path::Path;

fn main() {
    println!("Hello, world!");
}
struct Plan {
    cartons: Vec<Cartons>,
}
impl Plan {
    fn from_path(path: &Path) -> Result<(), csv::Error> {
        csv::Reader::from_path(path)?
            .deserialize::<PlanEntry>()
            .for_each(|_entry| {});

        Ok(())
    }
}
enum Cartons {
    Packed(Carton),
    Loose(Carton),
}
struct Carton {
    group: Option<String>,
    contents: Vec<Line>,
    dims: Option<Dimensions>,
}
struct Line {
    item: Unit,
    amt: u32,
}
struct Dimensions {
    length: f32,
    width: f32,
    height: f32,
    weight: f32,
}
struct Unit {
    fnsku: String,
    weight: f32,
}
#[derive(Deserialize)]
struct PlanEntry {
    #[serde(alias = "Info")]
    info: Option<u32>,
    #[serde(alias = "FNSKU")]
    fnsku: Option<String>,
    #[serde(alias = "Quantity")]
    units: Option<u32>,
    #[serde(alias = "Pack Type")]
    pack: Option<String>,
    #[serde(alias = "Staging Group")]
    group: Option<String>,
    #[serde(alias = "Unit Weight")]
    u_weight: Option<f32>,
    #[serde(alias = "Case QT")]
    c_units: Option<u32>,
    #[serde(alias = "Case Length")]
    length: Option<f32>,
    #[serde(alias = "Case Width")]
    width: Option<f32>,
    #[serde(alias = "Case Height")]
    height: Option<f32>,
    #[serde(alias = "Case Weight")]
    weight: Option<f32>,
}
impl PlanEntry {
    fn into_carton(self) -> Option<()> {
        match &*self.pack? {
            "Loose" => _loose(self),
            "Packed" => (),
            _ => (),
        }
        fn _loose(pe: PlanEntry) -> Option<Carton> {
            let unit = pe.fnsku.as_ref().and_then(|_| pe.u_weight).and_then(|_| {
                Some(Unit {
                    fnsku: pe.fnsku?,
                    weight: pe.u_weight?,
                })
            });
            let line = pe.units.and_then(|_| {
                Some(Line {
                    item: unit?,
                    amt: pe.units?,
                })
            });
            Some(Carton {
                group: pe.group,
                contents: vec![line?],
                dims: None,
            })
        }
    }
}
