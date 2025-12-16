use connectivity::{progress_bar_for_count, SecondsPastMidnight};
use indicatif::ProgressIterator;
use std::collections::HashMap;

use super::records::{ActivityFlag, Atco, Day, Record, JourneyHeader};

#[derive(Clone, Debug)]
pub struct TripStop {
    pub atco_code: Atco,
    pub activity_flag: ActivityFlag,
    pub departure_time: Option<SecondsPastMidnight>,
}

pub fn group(records: Vec<Record>, day: Day) -> HashMap<Atco, [u32; 24]> {
    let mut hour_counts: HashMap<Atco, [u32; 24]> = HashMap::new();

    let mut current_trip_header: Option<JourneyHeader> = None;
    let mut current_trip_stops: Vec<TripStop> = Vec::new();

    let progress = progress_bar_for_count(records.len());
    for record in records.iter().progress_with(progress) {
        match record {
            // When we find a JourneyHeader, we need to push the previous trip (current trip) if it's valid
            // and then start a new trip by setting the current trip to the new header
            Record::JourneyHeader(header) => {
                push_previous_trip_if_acceptable(
                    &mut hour_counts,
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
                });

            }
            _ => {
                // Ignore other records
                continue;
            }
        }

    }
    

    hour_counts

}

fn push_previous_trip_if_acceptable(
    hour_counts: &mut HashMap<Atco, [u32; 24]>,
    current_trip_header: &Option<JourneyHeader>,
    current_trip_stops: &[TripStop],
    operating_day: &Day,
) {
    if current_trip_stops.len() > 1 
        && current_trip_header.as_ref().unwrap().operating_days.contains(operating_day) 
        && current_trip_header.as_ref().unwrap().status.is_operating()
    {
        for stop in current_trip_stops {
            match stop.activity_flag {
                ActivityFlag::PickUpOnly | ActivityFlag::Both => {
                    add_departure_hour_count(hour_counts, stop);
                }
                _ => {}
            }
        }
    }
}

fn add_departure_hour_count(
    hour_counts: &mut HashMap<Atco, [u32; 24]>,
    trip_stop: &TripStop,
) {
    if let Some(departure_time) = trip_stop.departure_time {
        
        let hour = (departure_time.0 as f64 / 3600.0).floor() as usize;
        let counts = hour_counts.entry(trip_stop.atco_code.clone()).or_insert_with(empty_hour_counts);
        counts[hour as usize] += 1;
    } else {
        panic!("Trip stop without departure time at stop {:?}", trip_stop);
    }
}

fn empty_hour_counts() -> [u32; 24] {
    [0; 24]
}