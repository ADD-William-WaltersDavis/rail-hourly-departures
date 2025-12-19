use connectivity::progress_bar_for_count;
use indicatif::ProgressIterator;
use serde::Serialize;
use std::collections::HashMap;

use super::records::{Atco, Record};

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct StationName(pub String);

pub fn create_lookup(records: &[Record]) -> HashMap<Atco, StationName> {
    let mut pt_stop_lookup: HashMap<Atco, StationName> = HashMap::new();

    println!("Adding stop names...");
    let progress = progress_bar_for_count(records.len());
    for record in records.iter().progress_with(progress) {
        if let Record::StopName(stop_name) = record {
            pt_stop_lookup
                .entry(stop_name.atco_code.clone())
                .or_insert_with(|| StationName(stop_name.name.clone()));
        }
    }

    println!("PT Stop Lookup len: {:?}", pt_stop_lookup.len());
    pt_stop_lookup
}
