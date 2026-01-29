use indicatif::ProgressIterator;
use serde::Serialize;
use std::collections::HashMap;

use super::public_transport_mode::PublicTransportMode;
use super::records::{ActivityFlag, Atco, Day, JourneyHeader, Record, SecondsPastMidnight};
use super::stops::StationName;
use super::utils::progress_bar_for_count;

#[derive(Clone, Debug)]
pub struct TripStop {
    pub atco_code: Atco,
    pub activity_flag: ActivityFlag,
    pub departure_time: Option<SecondsPastMidnight>,
    pub is_first_stop: bool,
}

#[derive(Debug, Serialize)]
pub struct HourlyDepartures {
    pub atco_code: Atco,
    pub hour_counts: [u32; 24],
    pub hour_counts_journey_starts: [u32; 24],
    pub next_stop_atco: Vec<Vec<Atco>>,
}

pub fn group(
    records: Vec<Record>,
    lookup: &HashMap<Atco, StationName>,
    day: Day,
) -> HashMap<StationName, HourlyDepartures> {
    let mut hourly_departures: HashMap<StationName, HourlyDepartures> = HashMap::new();

    let mut current_trip_header: Option<JourneyHeader> = None;
    let mut current_trip_stops: Vec<TripStop> = Vec::new();

    let progress = progress_bar_for_count(records.len());
    for record in records.iter().progress_with(progress) {
        match record {
            Record::JourneyHeader(header) => {
                push_previous_trip_if_acceptable(
                    &mut hourly_departures,
                    lookup,
                    &current_trip_header,
                    &current_trip_stops,
                    &day,
                );
                current_trip_header = Some(header.clone());
                current_trip_stops.clear();
            }
            Record::JourneyRecordStop(stop) => {
                if stop.activity_flag == ActivityFlag::Neither {
                    continue;
                }
                current_trip_stops.push(TripStop {
                    atco_code: stop.atco_code.clone(),
                    activity_flag: stop.activity_flag.clone(),
                    departure_time: stop.departure_time,
                    is_first_stop: stop.is_first_stop,
                });
            }
            _ => {
                // Ignore other records
                continue;
            }
        }
    }

    // Push the last trip if applicable
    push_previous_trip_if_acceptable(
        &mut hourly_departures,
        lookup,
        &current_trip_header,
        &current_trip_stops,
        &day,
    );
    hourly_departures
}

fn exclude_non_rail_based(
    header: &JourneyHeader, 
    stops: &[TripStop], 
    lookup: &HashMap<Atco, StationName>
) -> bool {
    match header.vehicle_type {
        PublicTransportMode::Bus | PublicTransportMode::Coach | PublicTransportMode::Ferry => {
            let first_stop_name = &lookup[&stops[0].atco_code];
            if first_stop_name.contains_erroneous_name() {
                return true;
            }
            false

        },
        _ => true,
    }
}

fn push_previous_trip_if_acceptable(
    hourly_departures: &mut HashMap<StationName, HourlyDepartures>,
    lookup: &HashMap<Atco, StationName>,
    current_trip_header: &Option<JourneyHeader>,
    current_trip_stops: &[TripStop],
    operating_day: &Day,
) {
    if current_trip_stops.len() > 1
        && current_trip_header
            .as_ref()
            .unwrap()
            .operating_days
            .contains(operating_day)
        && current_trip_header.as_ref().unwrap().status.is_operating()
        && exclude_non_rail_based(current_trip_header.as_ref().unwrap(), current_trip_stops, lookup)
    {
        for (index, stop) in current_trip_stops.iter().enumerate() {
            match stop.activity_flag {
                ActivityFlag::PickUpOnly | ActivityFlag::Both => {
                    let next_stop_atco: Option<Atco> = if index < current_trip_stops.len() - 1 {
                        Some(current_trip_stops[index + 1].atco_code.clone())
                    } else {
                        None
                    };
                    add_departure_hour_count(hourly_departures, lookup, stop, next_stop_atco);
                }
                _ => {}
            }
        }
    }
}

fn add_departure_hour_count(
    hourly_departures: &mut HashMap<StationName, HourlyDepartures>,
    lookup: &HashMap<Atco, StationName>,
    trip_stop: &TripStop,
    next_stop_atco: Option<Atco>,
) {
    if let Some(departure_time) = trip_stop.departure_time {
        let hour = (departure_time.0 as f64 / 3600.0).floor() as usize;
        let departures = hourly_departures
            .entry(lookup.get(&trip_stop.atco_code).unwrap().clone())
            .or_insert_with(empty_hour_counts);
        departures.hour_counts[hour] += 1;
        if let Some(next_atco) = next_stop_atco {
            departures.next_stop_atco[hour].push(next_atco);
        }

        // If this is the first stop of the journey, also increment journey starts
        if trip_stop.is_first_stop {
            departures.hour_counts_journey_starts[hour] += 1;
        }

        // Ensure atco_code is set
        departures.atco_code = trip_stop.atco_code.clone();
    } else {
        panic!("Trip stop without departure time at stop {:?}", trip_stop);
    }
}

fn empty_hour_counts() -> HourlyDepartures {
    // A little hacky way to create a Vec of Vecs
    let mut next_stop_atco: Vec<Vec<Atco>> = Vec::new();
    for _ in 0..24 {
        next_stop_atco.push(Vec::new());
    }
    HourlyDepartures {
        atco_code: Atco("".to_string()), // A bit hacky also
        hour_counts: [0; 24],
        hour_counts_journey_starts: [0; 24],
        next_stop_atco,
    }
}
