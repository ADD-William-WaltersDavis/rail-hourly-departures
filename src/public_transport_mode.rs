use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PublicTransportMode {
    Bus,
    Coach,
    Ferry,
    LightRail,
    Metro,
    NationalRail,
    Tram,
    Tube,
}

impl FromStr for PublicTransportMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "bus" => Ok(PublicTransportMode::Bus),
            "coach" => Ok(PublicTransportMode::Coach),
            "ferry" => Ok(PublicTransportMode::Ferry),
            "light rail" => Ok(PublicTransportMode::LightRail),
            "metro" => Ok(PublicTransportMode::Metro),
            "national rail" => Ok(PublicTransportMode::NationalRail),
            "tram" => Ok(PublicTransportMode::Tram),
            "tube" => Ok(PublicTransportMode::Tube),
            // For CIF timetable data which have a character limit on the mode names
            // we map them to the full names.
            "lightrai" => Ok(PublicTransportMode::LightRail),
            "national" => Ok(PublicTransportMode::NationalRail),
            "subway" => Ok(PublicTransportMode::Tube),
            "rail" => Ok(PublicTransportMode::NationalRail),
            _ => Err(format!("Invalid public transport mode: {}", s)),
        }
    }
}