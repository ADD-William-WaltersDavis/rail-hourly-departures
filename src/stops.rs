use indicatif::ProgressIterator;
use serde::Serialize;
use std::collections::HashMap;

use super::records::{Atco, Record};
use super::utils::progress_bar_for_count;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct StationName(pub String);

impl StationName {
    /// There are some erroneous labelling of vehicle types in CIF data which
    /// results in some trams and such being in the bus dataset. This checks for
    /// common erroneous stop names to help identify these.
    pub fn contains_erroneous_name(&self) -> bool {
        let erroneous_names = [
            "Manchester Metrolink",
            "Tram Stop",
            "Tramway",
            "Edinburgh Trams",
            "Supertram",
            "Metro Stop",
            "Air-Rail",
            "Kinneil Railway",
            "SPT Subway Station",
        ];
        for erroneous_name in erroneous_names {
            if self.0.contains(erroneous_name) {
                return true;
            }
        }
        false
    }
}
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
