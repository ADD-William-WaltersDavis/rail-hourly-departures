use fs_err::read_to_string;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{cmp::Eq, collections::HashMap, hash::Hash, str::FromStr};

use super::utils::progress_bar_for_count;
    
pub fn parse(raw_cif_text: String) -> Vec<Record> {
    println!("Parsing CIF file...");
    // Remove carriage returns and split the string into lines
    let cif = raw_cif_text.replace("\r", "");
    let mut cif_lines = cif.split("\n").collect::<Vec<&str>>();
    // Drop the last line if it is empty
    if cif_lines.last().unwrap().is_empty() {
        cif_lines.pop();
    }

    println!("Number of lines: {}", cif_lines.len());
    let progress = progress_bar_for_count(cif_lines.len());

    cif_lines
        .par_iter()
        .progress_with(progress)
        .filter_map(|line| {
            let record_identifier = RecordIdentifier::from_str(&line[0..2]).unwrap();
            match record_identifier {
                RecordIdentifier::BS => {
                    Some(Record::JourneyHeader(JourneyHeader::from_bs_str(line)))
                }
                // RecordIdentifier::QO => JourneyRecordStop::from_qo_str(line, &atco_mapping)
                //     .map(Record::JourneyRecordStop),
                // RecordIdentifier::QI => JourneyRecordStop::from_qi_str(line, &atco_mapping)
                //     .map(Record::JourneyRecordStop),
                // RecordIdentifier::QT => JourneyRecordStop::from_qt_str(line, &atco_mapping)
                //     .map(Record::JourneyRecordStop),
                // RecordIdentifier::QL => {
                //     StopName::from_ql_str(line, &atco_mapping).map(Record::StopName)
                // }
                _ => None,
            }
        })
        .collect()
}

pub fn read_file(file_path: &str) -> String {
    fs_err::read_to_string(file_path).expect("Something went wrong reading the file")
}

/// Loads in the ATCO to ATCO mapping from a TOML file.
/// This file was manually created to cover all the unaligned ATCO codes in the CIF data
/// where they do not match the NaPTAN data.
/// The mapping ensures the stops are correctly aligned to the same stop, for example:
/// 9100CNNBELL in the CIF files directly is mapped to 9100CNNB (Canonbury Rail Station)
fn read_mapping(config_path: &str) -> HashMap<Atco, Atco> {
    let file = read_to_string(format!("{}/atco_station_mappings.toml", config_path)).unwrap();
    let atco_mapping: HashMap<Atco, Atco> = toml::from_str(&file).unwrap();
    atco_mapping
}

#[derive(Debug)]
pub enum Record {
    JourneyHeader(JourneyHeader),
    JourneyRecordStop(JourneyRecordStop),
    StopName(StopName),
}

#[derive(Debug)]
pub enum RecordIdentifier {
    TI, // TIPLOC Insert Record
    TA, // TIPLOC Amend Record
    TD, // TIPLOC Delete Record
    BS, // Basic Schedule Record
    LO, // Location Origin
    LI, // Location Intermediate
    LT, // Location Terminate
    Other,
}

impl FromStr for RecordIdentifier {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TI" => Ok(RecordIdentifier::TI),
            "TA" => Ok(RecordIdentifier::TA),
            "TD" => Ok(RecordIdentifier::TD),
            "BS" => Ok(RecordIdentifier::BS),
            "LO" => Ok(RecordIdentifier::LO),
            "LI" => Ok(RecordIdentifier::LI),
            "LT" => Ok(RecordIdentifier::LT),
            _ => Ok(RecordIdentifier::Other),
        }
    }
}

/// A value for time past midnight in seconds.
/// For example 8am is 28800 seconds past midnight.
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash,
)]
pub struct SecondsPastMidnight(pub usize);

// One record per journey. A journey header may be immediately
// followed by optional sets of date running records and journey note
// records and should then be followed by a set of journey records
// (origin, intermediate, destination) giving a set of records that
// completely define dates, times, places, operator and vehicle type of
// the journey. The entire set of records relating to a single journey
// may be immediately followed by one or more journey repetition
// records.
/// Denoted by "BS" in the CIF file
#[derive(Debug, Clone)]
pub struct JourneyHeader {
    pub status: Status,
    pub _uid: String,
    pub date_runs_from: Date,
    pub date_runs_to: Date,
    pub operating_days: OperatingDays,
    pub train_status: char,
    pub category: TrainCategory,
}

impl JourneyHeader {
    fn from_bs_str(bs_string: &str) -> Self {
        // Parse the BS string and extract the relevant fields
        JourneyHeader {
            status: Status::from_str(&bs_string[2..3]).unwrap(),
            _uid: bs_string[3..9].trim().to_string(),
            date_runs_from: Date::from_str(&bs_string[9..15]).unwrap(),
            date_runs_to: Date::from_str(&bs_string[15..21]).unwrap(),
            operating_days: OperatingDays::from_cif_str(&bs_string[21..28]),
            train_status: bs_string.chars().nth(29).unwrap(),
            category: TrainCategory::from_str(&bs_string[30..32]).unwrap(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Date {
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl FromStr for Date {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let year = s[0..2].parse::<u8>().map_err(|_| ())?;
        let month = s[2..4].parse::<u8>().map_err(|_| ())?;
        let day = s[4..6].parse::<u8>().map_err(|_| ())?;
        Ok(Date { day, month, year })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrainCategory {
    Passenger,
    Other,
}

impl FromStr for TrainCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OL" => Ok(TrainCategory::Passenger),
            "OU" => Ok(TrainCategory::Passenger),
            "OO" => Ok(TrainCategory::Passenger),
            "OS" => Ok(TrainCategory::Passenger),
            "OW" => Ok(TrainCategory::Passenger),
            _ => Ok(TrainCategory::Other),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    New,
    Delete,
    Revise,
}

impl FromStr for Status {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "N" => Ok(Status::New),
            "D" => Ok(Status::Delete),
            "R" => Ok(Status::Revise),
            _ => Err(()),
        }
    }
}

impl Status {
    pub fn is_operating(&self) -> bool {
        match self {
            Status::New | Status::Revise => true,
            Status::Delete => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OperatingDays(pub Vec<Day>);
impl OperatingDays {
    fn from_cif_str(s: &str) -> Self {
        // Example input ["1111100"]
        let mut days = Vec::new();
        for (i, c) in s.chars().enumerate() {
            if c == '1' {
                match i {
                    0 => days.push(Day::Monday),
                    1 => days.push(Day::Tuesday),
                    2 => days.push(Day::Wednesday),
                    3 => days.push(Day::Thursday),
                    4 => days.push(Day::Friday),
                    5 => days.push(Day::Saturday),
                    6 => days.push(Day::Sunday),
                    _ => {}
                }
            }
        }
        OperatingDays(days)
    }
    pub fn contains(&self, day: &Day) -> bool {
        self.0.contains(day)
    }
}

#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl FromStr for Day {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Monday" => Ok(Day::Monday),
            "Tuesday" => Ok(Day::Tuesday),
            "Wednesday" => Ok(Day::Wednesday),
            "Thursday" => Ok(Day::Thursday),
            "Friday" => Ok(Day::Friday),
            "Saturday" => Ok(Day::Saturday),
            "Sunday" => Ok(Day::Sunday),
            _ => Err(()),
        }
    }
}

pub trait TimeConversion {
    fn from_24hr_str(s: &str) -> Self;
}

impl TimeConversion for SecondsPastMidnight {
    fn from_24hr_str(s: &str) -> Self {
        let hours = s[0..2].parse::<usize>().unwrap();
        let minutes = s[2..].parse::<usize>().unwrap();
        SecondsPastMidnight((hours * 3600) + (minutes * 60))
    }
}

#[derive(Clone, Debug)]
pub struct JourneyRecordStop {
    pub atco_code: Atco,
    pub activity_flag: ActivityFlag,
    pub _arrival_time: Option<SecondsPastMidnight>,
    pub departure_time: Option<SecondsPastMidnight>,
    pub is_first_stop: bool,
}

impl JourneyRecordStop {
    /// Denoted by "QO" in the CIF file
    fn from_qo_str(s: &str, atco_mapping: &HashMap<Atco, Atco>) -> Option<Self> {
        let atco_code = Atco::from_mapped_str(&s[2..14], atco_mapping)?;
        Some(JourneyRecordStop {
            atco_code,
            activity_flag: ActivityFlag::PickUpOnly, // As origin stop
            _arrival_time: None,
            departure_time: Some(SecondsPastMidnight::from_24hr_str(&s[14..18])),
            is_first_stop: true,
        })
    }
    /// Denoted by "QI" in the CIF file
    fn from_qi_str(s: &str, atco_mapping: &HashMap<Atco, Atco>) -> Option<Self> {
        let atco_code = Atco::from_mapped_str(&s[2..14], atco_mapping)?;
        Some(JourneyRecordStop {
            atco_code,
            _arrival_time: Some(SecondsPastMidnight::from_24hr_str(&s[14..18])),
            departure_time: Some(SecondsPastMidnight::from_24hr_str(&s[18..22])),
            activity_flag: ActivityFlag::from_str(&s[22..23]).unwrap(),
            is_first_stop: false,
        })
    }
    /// Denoted by "QT" in the CIF file
    fn from_qt_str(s: &str, atco_mapping: &HashMap<Atco, Atco>) -> Option<Self> {
        let atco_code = Atco::from_mapped_str(&s[2..14], atco_mapping)?;
        Some(JourneyRecordStop {
            atco_code,
            activity_flag: ActivityFlag::SetDownOnly, // As final stop
            _arrival_time: Some(SecondsPastMidnight::from_24hr_str(&s[14..18])),
            departure_time: None,
            is_first_stop: false,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize)]
pub struct Atco(pub String);
impl FromStr for Atco {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Atco(s.trim().to_string()))
    }
}

impl Atco {
    fn starts_with_999(&self) -> bool {
        self.0.starts_with("999")
    }
    // Converts a string to an ATCO code, using a mapping if available where the mapping
    // is of missing ATCO codes to their correct ATCO codes from the underlying data.
    // TODO: Remove this fix once NaPTAN data covers all these stops.
    fn from_mapped_str(s: &str, atco_mapping: &HashMap<Atco, Atco>) -> Option<Atco> {
        let atco = Atco::from_str(s).unwrap();
        // If the ATCO is to be dropped or starts with 999, return None.
        // These have been identified as non-existent stops or 999 denotes that
        // passengers cannot board or alight at this stop.
        if atco.starts_with_999() {
            return None;
        }
        Some(atco_mapping.get(&atco).cloned().unwrap_or(atco))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityFlag {
    Both,
    PickUpOnly,
    SetDownOnly,
    Neither,
}

impl FromStr for ActivityFlag {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "B" => Ok(ActivityFlag::Both),
            "P" => Ok(ActivityFlag::PickUpOnly),
            "S" => Ok(ActivityFlag::SetDownOnly),
            "N" => Ok(ActivityFlag::Neither),
            _ => Err(()),
        }
    }
}

/// Denoted by "QL" in the CIF file
#[derive(Debug)]
pub struct StopName {
    pub _status: Status,
    pub atco_code: Atco,
    pub name: String,
}

impl StopName {
    fn from_ql_str(ql_string: &str, atco_mapping: &HashMap<Atco, Atco>) -> Option<Self> {
        let atco_code = Atco::from_str(&ql_string[3..15]).unwrap();
        // Drop the record if the ATCO code is already in the mapping as we will use the
        // mapped stop instead of this one.
        if atco_mapping.get(&atco_code).is_some() || atco_code.starts_with_999() {
            return None;
        }
        // Parse the QL string and extract the relevant fields
        Some(StopName {
            _status: Status::from_str(&ql_string[2..3]).unwrap(),
            atco_code,
            name: ql_string[15..63].trim().replace(",", "").to_string(),
        })
    }
}
