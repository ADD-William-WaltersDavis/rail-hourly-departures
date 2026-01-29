use indicatif::ProgressIterator;
use std::collections::{HashMap, HashSet};

use super::records::{Record, ThreeAlphaCode, Tiploc};
use super::utils::progress_bar_for_count;

pub fn create_lookup(
    records: &[Record],
    gb_station_three_alpha_codes: &[ThreeAlphaCode],
) -> HashMap<Tiploc, ThreeAlphaCode> {
    let stanox_lookup = create_stanox_lookup(records, gb_station_three_alpha_codes);

    let mut rail_stop_lookup: HashMap<Tiploc, ThreeAlphaCode> = HashMap::new();

    println!("Creating rail stop lookup");
    let progress = progress_bar_for_count(records.len());
    for record in records.iter().progress_with(progress) {
        if let Record::Stop(stop) = record {
            if let Some(three_alpha_code) = stanox_lookup.get(&stop.stanox) {
                rail_stop_lookup
                    .entry(stop.tiploc.clone())
                    .or_insert_with(|| three_alpha_code.clone());
            }
        }
    }

    println!("Rail Stop Lookup len: {:?}", rail_stop_lookup.len());
    rail_stop_lookup
}

fn create_stanox_lookup(
    records: &[Record],
    gb_station_three_alpha_codes: &[ThreeAlphaCode],
) -> HashMap<String, ThreeAlphaCode> {
    let mut stanox_lookup: HashMap<String, ThreeAlphaCode> = HashMap::new();

    let three_alpha_code_set: HashSet<&ThreeAlphaCode> =
        gb_station_three_alpha_codes.iter().collect();

    println!("Creating stanox lookup");
    let progress = progress_bar_for_count(records.len());
    for record in records.iter().progress_with(progress) {
        if let Record::Stop(stop) = record {
            if let Some(three_alpha_code) = &stop.three_alpha_code {
                if !three_alpha_code_set.contains(three_alpha_code) {
                    continue;
                }
                stanox_lookup
                    .entry(stop.stanox.clone())
                    .or_insert_with(|| three_alpha_code.clone());
            }
        }
    }

    println!("Stanox Lookup len: {:?}", stanox_lookup.len());
    stanox_lookup
}
