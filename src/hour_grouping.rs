use connectivity::{SecondsPastMidnight, progress_bar_for_count};
use indicatif::ProgressIterator;
use serde::Serialize;
use std::collections::HashMap;

use super::records::{ActivityFlag, Atco, Day, JourneyHeader, Record};

#[derive(Clone, Debug)]
pub struct TripStop {
    pub atco_code: Atco,
    pub activity_flag: ActivityFlag,
    pub departure_time: Option<SecondsPastMidnight>,
    pub is_first_stop: bool,
}

#[derive(Serialize)]
pub struct HourlyDepartures {
    pub hour_counts: [u32; 24],
    pub hour_counts_journey_starts: [u32; 24],
}

pub fn group(records: Vec<Record>, day: Day) -> HashMap<Atco, HourlyDepartures> {
    // let mut hour_counts: HashMap<Atco, [u32; 24]> = HashMap::new();
    // let mut hour_counts_journey_starts: HashMap<Atco, [u32; 24]> = HashMap::new();
    let mut hourly_departures: HashMap<Atco, HourlyDepartures> = HashMap::new();

    let mut current_trip_header: Option<JourneyHeader> = None;
    let mut current_trip_stops: Vec<TripStop> = Vec::new();

    let progress = progress_bar_for_count(records.len());
    for record in records.iter().progress_with(progress) {
        match record {
            Record::JourneyHeader(header) => {
                push_previous_trip_if_acceptable(
                    &mut hourly_departures,
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
        &current_trip_header,
        &current_trip_stops,
        &day,
    );
    hourly_departures
}

fn push_previous_trip_if_acceptable(
    hourly_departures: &mut HashMap<Atco, HourlyDepartures>,
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
    {
        for stop in current_trip_stops {
            match stop.activity_flag {
                ActivityFlag::PickUpOnly | ActivityFlag::Both => {
                    add_departure_hour_count(hourly_departures, stop);
                }
                _ => {}
            }
        }
    }
}

fn add_departure_hour_count(
    hourly_departures: &mut HashMap<Atco, HourlyDepartures>,
    trip_stop: &TripStop,
) {
    if let Some(departure_time) = trip_stop.departure_time {
        let hour = (departure_time.0 as f64 / 3600.0).floor() as usize;
        let departures = hourly_departures
            .entry(trip_stop.atco_code.clone())
            .or_insert_with(empty_hour_counts);
        departures.hour_counts[hour] += 1;

        // If this is the first stop of the journey, also increment journey starts
        if trip_stop.is_first_stop {
            departures.hour_counts_journey_starts[hour] += 1;
        }
    } else {
        panic!("Trip stop without departure time at stop {:?}", trip_stop);
    }
}

fn empty_hour_counts() -> HourlyDepartures {
    HourlyDepartures {
        hour_counts: [0; 24],
        hour_counts_journey_starts: [0; 24],
    }
}
