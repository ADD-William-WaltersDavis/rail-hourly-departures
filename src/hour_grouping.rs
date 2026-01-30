use indicatif::ProgressIterator;
use serde::Serialize;
use std::collections::HashMap;

use super::records::{
    ActivityFlag, Date, Day, JourneyHeader, Record, SecondsPastMidnight, ThreeAlphaCode, Tiploc,
    TrainCategory,
};
use super::utils::progress_bar_for_count;

#[derive(Clone, Debug)]
pub struct TripStop {
    pub three_alpha_code: ThreeAlphaCode,
    pub activity_flag: ActivityFlag,
    pub departure_time: Option<SecondsPastMidnight>,
    pub is_first_stop: bool,
}

#[derive(Debug, Serialize)]
pub struct HourlyDepartures {
    pub three_alpha_code: ThreeAlphaCode,
    pub hour_counts: [u32; 24],
    pub hour_counts_journey_starts: [u32; 24],
    pub next_stop_three_alpha_code: Vec<Vec<ThreeAlphaCode>>,
}

pub fn group(
    records: Vec<Record>,
    lookup: &HashMap<Tiploc, ThreeAlphaCode>,
    day: &Day,
    date: &Date,
) -> HashMap<ThreeAlphaCode, HourlyDepartures> {
    let mut hourly_departures: HashMap<ThreeAlphaCode, HourlyDepartures> = HashMap::new();

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
                    day,
                    date,
                );
                current_trip_header = Some(header.clone());
                current_trip_stops.clear();
            }
            Record::JourneyRecordStop(stop) => {
                if stop.activity_flag == ActivityFlag::Neither {
                    continue;
                }
                if let Some(three_alpha_code) = lookup.get(&stop.tiploc) {
                    current_trip_stops.push(TripStop {
                        three_alpha_code: three_alpha_code.clone(),
                        activity_flag: stop.activity_flag.clone(),
                        departure_time: stop.departure_time,
                        is_first_stop: stop.is_first_stop,
                    });
                } else {
                    continue;
                }
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
        day,
        date,
    );
    hourly_departures
}

fn date_in_scope(operating_date: &Date, start_date: &Date, end_date: &Date) -> bool {
    operating_date.0 >= start_date.0 && operating_date.0 <= end_date.0
}

fn push_previous_trip_if_acceptable(
    hourly_departures: &mut HashMap<ThreeAlphaCode, HourlyDepartures>,
    current_trip_header: &Option<JourneyHeader>,
    current_trip_stops: &[TripStop],
    operating_day: &Day,
    operating_date: &Date,
) {
    if current_trip_stops.len() > 1
        && current_trip_header
            .as_ref()
            .unwrap()
            .operating_days
            .contains(operating_day)
        && current_trip_header.as_ref().unwrap().status.is_operating()
        && current_trip_header.as_ref().unwrap().category == TrainCategory::Passenger
        && date_in_scope(
            operating_date,
            &current_trip_header.as_ref().unwrap().date_runs_from,
            &current_trip_header.as_ref().unwrap().date_runs_to,
        )
    {
        for (index, stop) in current_trip_stops.iter().enumerate() {
            match stop.activity_flag {
                ActivityFlag::PickUpOnly | ActivityFlag::Both => {
                    let next_stop_three_alpha_code: Option<ThreeAlphaCode> =
                        if index < current_trip_stops.len() - 1 {
                            Some(current_trip_stops[index + 1].three_alpha_code.clone())
                        } else {
                            None
                        };
                    add_departure_hour_count(hourly_departures, stop, next_stop_three_alpha_code);
                }
                _ => {}
            }
        }
    }
}

fn add_departure_hour_count(
    hourly_departures: &mut HashMap<ThreeAlphaCode, HourlyDepartures>,
    trip_stop: &TripStop,
    next_stop_three_alpha_code: Option<ThreeAlphaCode>,
) {
    if let Some(departure_time) = trip_stop.departure_time {
        let hour = (departure_time.0 as f64 / 3600.0).floor() as usize;
        let departures = hourly_departures
            .entry(trip_stop.three_alpha_code.clone())
            .or_insert_with(empty_hour_counts);
        departures.hour_counts[hour] += 1;
        if let Some(next_stop_three_alpha_code) = next_stop_three_alpha_code {
            departures.next_stop_three_alpha_code[hour].push(next_stop_three_alpha_code);
        }

        // If this is the first stop of the journey, also increment journey starts
        if trip_stop.is_first_stop {
            departures.hour_counts_journey_starts[hour] += 1;
        }

        // Ensure three_alpha_code is set
        departures.three_alpha_code = trip_stop.three_alpha_code.clone();
    } else {
        panic!("Trip stop without departure time at stop {:?}", trip_stop);
    }
}

fn empty_hour_counts() -> HourlyDepartures {
    // A little hacky way to create a Vec of Vecs
    let mut next_stop_three_alpha_code: Vec<Vec<ThreeAlphaCode>> = Vec::new();
    for _ in 0..24 {
        next_stop_three_alpha_code.push(Vec::new());
    }
    HourlyDepartures {
        three_alpha_code: ThreeAlphaCode("".to_string()), // A bit hacky also
        hour_counts: [0; 24],
        hour_counts_journey_starts: [0; 24],
        next_stop_three_alpha_code,
    }
}
