use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use super::hour_grouping::HourlyDepartures;
use super::records::ThreeAlphaCode;

#[derive(Debug, Serialize)]
pub struct CriteriaResults {
    pub three_alpha_code: ThreeAlphaCode,
    pub hour_counts: [u32; 24],
    pub hour_counts_journey_starts: [u32; 24],
    pub all_7_7: bool,
    pub all_6_10: bool,
    pub avg_7_7: bool,
    pub avg_6_10: bool,
    pub flagged_for_review: bool,
    pub next_stop_three_alpha_code: Option<Vec<Vec<ThreeAlphaCode>>>,
}

pub fn evaluate_criteria(
    departures: &HashMap<ThreeAlphaCode, HourlyDepartures>,
) -> HashMap<ThreeAlphaCode, CriteriaResults> {
    let mut results: HashMap<ThreeAlphaCode, CriteriaResults> = HashMap::new();

    for (three_alpha_code, hourly_departure) in departures.iter() {
        let mut flagged_for_review = false;
        let mut criteria_result = CriteriaResults {
            three_alpha_code: hourly_departure.three_alpha_code.clone(),
            hour_counts: hourly_departure.hour_counts,
            hour_counts_journey_starts: hourly_departure.hour_counts_journey_starts,
            all_7_7: all_meet_criteria(7..19, hourly_departure, &mut flagged_for_review),
            all_6_10: all_meet_criteria(6..22, hourly_departure, &mut flagged_for_review),
            avg_7_7: avg_meet_criteria(7..19, hourly_departure, &mut flagged_for_review),
            avg_6_10: avg_meet_criteria(6..22, hourly_departure, &mut flagged_for_review),
            flagged_for_review: false,
            next_stop_three_alpha_code: None,
        };
        criteria_result.flagged_for_review = flagged_for_review;
        if flagged_for_review {
            criteria_result.next_stop_three_alpha_code =
                Some(hourly_departure.next_stop_three_alpha_code.clone());
        }
        results.insert(three_alpha_code.clone(), criteria_result);
    }

    results
}

fn all_meet_criteria(
    range: Range<usize>,
    departures: &HourlyDepartures,
    flagged_for_review: &mut bool,
) -> bool {
    // Each station which has more than four departures per hour (or more than 2 at the start of
    // their route) for every hour within the range
    let mut criteria_met = true;

    // First check all hours have 4+ departures
    for hour in range.clone() {
        if departures.hour_counts[hour] < 4 && departures.hour_counts_journey_starts[hour] < 2 {
            criteria_met = false;
        }
    }

    // If not met, check if all hours have 2+ departures to the same next stop
    if !criteria_met {
        criteria_met = all_hours_have_2_same_next_stop(range, departures, flagged_for_review);
    }
    criteria_met
}

fn all_hours_have_2_same_next_stop(
    range: Range<usize>,
    departures: &HourlyDepartures,
    flagged_for_review: &mut bool,
) -> bool {
    // Get sum for each next station at each hour in range
    let mut next_station_counts: Vec<HashMap<ThreeAlphaCode, u32>> =
        Vec::with_capacity(range.len());
    let mut unique_stations: HashSet<ThreeAlphaCode> = HashSet::new();

    for hour in range {
        let mut hour_map: HashMap<ThreeAlphaCode, u32> = HashMap::new();
        for three_alpha_code in departures.next_stop_three_alpha_code[hour].iter() {
            *hour_map.entry(three_alpha_code.clone()).or_insert(0) += 1;
            unique_stations.insert(three_alpha_code.clone());
        }
        next_station_counts.push(hour_map);
    }

    // Check each unique destination station to see if it occurs every hour with 2+ departures
    let mut meets_criteria = false;
    for station in unique_stations.iter() {
        let mut station_meets_criteria = true;
        for hour_map in next_station_counts.iter() {
            if let Some(count) = hour_map.get(station) {
                if *count >= 2 {
                    continue;
                } else {
                    station_meets_criteria = false;
                    break;
                }
            } else {
                station_meets_criteria = false;
                break;
            }
        }
        if station_meets_criteria {
            meets_criteria = true;
            break;
        }
    }

    // If not meeting criteria, but there are three stations at each hour, flag for review
    // This could be caused by a two services inthe same direction leaving within the hour,
    // but having different next stops
    if !meets_criteria {
        let mut all_hours_have_three_unique = true;
        for hour_map in next_station_counts.iter() {
            if hour_map.len() < 3 {
                all_hours_have_three_unique = false;
                break;
            }
        }
        if all_hours_have_three_unique {
            *flagged_for_review = true;
        }
    }
    meets_criteria
}

fn avg_meet_criteria(
    range: Range<usize>,
    departures: &HourlyDepartures,
    flagged_for_review: &mut bool,
) -> bool {
    // Each station which has an average of 4 departures or more (i.e. an average of four per hour) or
    // an average of 2+ at the start of their route across the hours in the range
    let total: u32 = range
        .clone()
        .map(|hour| {
            u32::max(
                departures.hour_counts[hour],
                departures.hour_counts_journey_starts[hour] * 2,
            )
        })
        .sum();
    let mut criteria_met = total >= (range.len() * 4) as u32;

    // If not met, check if average of 2+ departures to the same next stop
    if !criteria_met {
        criteria_met = avg_hours_have_2_same_next_stop(range, departures, flagged_for_review);
    }
    criteria_met
}

fn avg_hours_have_2_same_next_stop(
    range: Range<usize>,
    departures: &HourlyDepartures,
    flagged_for_review: &mut bool,
) -> bool {
    // Get sum for each next station at each hour in range
    let mut next_station_counts: HashMap<ThreeAlphaCode, u32> = HashMap::new();

    for hour in range.clone() {
        for three_alpha_code in departures.next_stop_three_alpha_code[hour].iter() {
            *next_station_counts
                .entry(three_alpha_code.clone())
                .or_insert(0) += 1;
        }
    }

    // Check if any next station has an average of 2+ departures across the hours in range
    let mut meets_criteria = false;
    for (_station, count) in next_station_counts.iter() {
        if *count >= (range.len() * 2) as u32 {
            meets_criteria = true;
            break;
        }
    }

    // If not meeting criteria, but there are at least three next stations and an average of 3+ departures, flag for review
    if !meets_criteria {
        let unique_stations_count = next_station_counts.len();
        let total_departures: u32 = next_station_counts.values().sum();
        if unique_stations_count >= 3 && total_departures >= (range.len() * 3) as u32 {
            *flagged_for_review = true;
        }
    }
    meets_criteria
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avg_meet_criteria() {
        let departures = HourlyDepartures {
            three_alpha_code: ThreeAlphaCode("TEST".to_string()),
            hour_counts: [
                0, 0, 0, 0, 0, 3, 3, 4, 5, 3, 5, 3, 5, 3, 5, 3, 5, 4, 5, 3, 4, 4, 4, 1,
            ],
            hour_counts_journey_starts: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            next_stop_three_alpha_code: vec![Vec::new(); 24],
        };
        let mut flagged_for_review = false;
        let result = avg_meet_criteria(7..19, &departures, &mut flagged_for_review);
        assert!(result);
    }

    #[test]
    fn test_all_meet_criteria() {
        let departures = HourlyDepartures {
            three_alpha_code: ThreeAlphaCode("9100KMPSTNH".to_string()),
            hour_counts: [
                0, 0, 0, 0, 0, 1, 3, 3, 3, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 3, 2, 0,
            ],
            hour_counts_journey_starts: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            next_stop_three_alpha_code: vec![Vec::new(); 24],
        };
        let mut flagged_for_review = false;
        let result = all_meet_criteria(7..19, &departures, &mut flagged_for_review);
        assert!(result);
    }
}
