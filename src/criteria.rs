use serde::Serialize;
use std::collections::HashMap;

use super::hour_grouping::HourlyDepartures;
use super::records::Atco;

#[derive(Serialize)]
pub struct CriteriaResults {
    pub hour_counts: [u32; 24],
    pub hour_counts_journey_starts: [u32; 24],
    pub all_7_7: bool,
    pub all_6_10: bool,
    pub avg_7_7: bool,
    pub avg_6_10: bool,
}

pub fn evaluate_criteria(
    departures: &HashMap<Atco, HourlyDepartures>,
) -> HashMap<Atco, CriteriaResults> {
    let mut results: HashMap<Atco, CriteriaResults> = HashMap::new();

    for (atco, hourly_departure) in departures.iter() {
        let criteria_result = CriteriaResults {
            hour_counts: hourly_departure.hour_counts,
            hour_counts_journey_starts: hourly_departure.hour_counts_journey_starts,
            all_7_7: does_station_meet_all_7_7(hourly_departure),
            all_6_10: does_station_meet_all_6_10(hourly_departure),
            avg_7_7: does_station_meet_avg_7_7(hourly_departure),
            avg_6_10: does_station_meet_avg_6_10(hourly_departure),
        };
        results.insert(atco.clone(), criteria_result);
    }

    results
}

fn does_station_meet_all_7_7(departures: &HourlyDepartures) -> bool {
    // Each station which has more than four departures per hour (or more than 2 at the start of
    // their route) for every hour between 7am and 6:59pm
    for hour in 7..19 {
        if departures.hour_counts[hour] < 4 && departures.hour_counts_journey_starts[hour] < 2 {
            return false;
        }
    }
    true
}

fn does_station_meet_all_6_10(departures: &HourlyDepartures) -> bool {
    // Each station which has more than four departures per hour (or more than 2 at the start of
    // their route) for every hour between 6am and 9:59pm
    for hour in 6..22 {
        if departures.hour_counts[hour] < 4 && departures.hour_counts_journey_starts[hour] < 2 {
            return false;
        }
    }
    true
}

fn does_station_meet_avg_7_7(departures: &HourlyDepartures) -> bool {
    // Each station which has 64 departures or more (i.e. an average of four per hour) between 7am and 6:59pm
    let total: u32 = departures.hour_counts[7..19].iter().sum();
    total >= 48
}

fn does_station_meet_avg_6_10(departures: &HourlyDepartures) -> bool {
    // Each station which has 48 departures or more (i.e. an average of four per hour) between 6am and 9:59pm
    let total: u32 = departures.hour_counts[6..22].iter().sum();
    total >= 64
}
