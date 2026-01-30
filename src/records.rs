use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{cmp::Eq, hash::Hash, str::FromStr};

use super::utils::progress_bar_for_count;

/// Parse in the raw CIF rail timetable data
/// See: https://wiki.openraildata.com/index.php/CIF_File_Format for details on the format
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
                RecordIdentifier::TI => Some(Record::Stop(Stop::from_ti_str(line)?)),
                RecordIdentifier::LO => Some(Record::JourneyRecordStop(
                    JourneyRecordStop::from_lo_str(line)?,
                )),
                RecordIdentifier::LI => Some(Record::JourneyRecordStop(
                    JourneyRecordStop::from_li_str(line)?,
                )),
                RecordIdentifier::LT => Some(Record::JourneyRecordStop(
                    JourneyRecordStop::from_lt_str(line)?,
                )),
                _ => None,
            }
        })
        .collect()
}

pub fn read_file(file_path: &str) -> String {
    fs_err::read_to_string(file_path).expect("Something went wrong reading the file")
}

#[derive(Debug)]
pub enum Record {
    JourneyHeader(JourneyHeader),
    JourneyRecordStop(JourneyRecordStop),
    Stop(Stop),
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
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
    pub _train_status: char,
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
            _train_status: bs_string.chars().nth(29).unwrap(),
            category: TrainCategory::from_str(&bs_string[30..32]).unwrap(),
        }
    }
}

/// YYMMDD format date
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Date(pub usize);

impl FromStr for Date {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Date(s.parse::<usize>().unwrap()))
    }
}

pub fn parse_date(s: &str) -> Result<Date, String> {
    s.parse::<usize>()
        .map(Date)
        .map_err(|e| format!("Invalid date format: {}", e))
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
            "OL" => Ok(TrainCategory::Passenger), // London Underground/Metro Service
            // "OU" => Ok(TrainCategory::Passenger), // Unadvertised Passenger Train
            "OO" => Ok(TrainCategory::Passenger), // Ordinary Passenger Train
            // "OS" => Ok(TrainCategory::Passenger), // Staff Train
            "OW" => Ok(TrainCategory::Passenger), // Mixed
            "XC" => Ok(TrainCategory::Passenger), // Channel Tunnel
            "XI" => Ok(TrainCategory::Passenger), // International
            "XX" => Ok(TrainCategory::Passenger), // Express Passenger
            "XZ" => Ok(TrainCategory::Passenger), // Sleeper (Domestic)
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
    pub tiploc: Tiploc,
    pub activity_flag: ActivityFlag,
    pub _arrival_time: Option<SecondsPastMidnight>,
    pub departure_time: Option<SecondsPastMidnight>,
    pub is_first_stop: bool,
}

impl JourneyRecordStop {
    /// Denoted by "LO" in the CIF file
    fn from_lo_str(s: &str) -> Option<Self> {
        Some(JourneyRecordStop {
            tiploc: Tiploc::from_str(&s[2..9]).unwrap(),
            activity_flag: ActivityFlag::PickUpOnly, // As origin stop
            _arrival_time: None,
            departure_time: Some(SecondsPastMidnight::from_24hr_str(&s[10..14])),
            is_first_stop: true,
        })
    }
    /// Denoted by "LI" in the CIF file
    fn from_li_str(s: &str) -> Option<Self> {
        if !&s[20..24].trim().is_empty() {
            return None;
        }
        Some(JourneyRecordStop {
            tiploc: Tiploc::from_str(&s[2..9]).unwrap(),
            _arrival_time: Some(SecondsPastMidnight::from_24hr_str(&s[10..14])),
            departure_time: Some(SecondsPastMidnight::from_24hr_str(&s[15..19])),
            activity_flag: ActivityFlag::from_str(s[42..54].trim()).unwrap(),
            is_first_stop: false,
        })
    }
    /// Denoted by "LT" in the CIF file
    fn from_lt_str(s: &str) -> Option<Self> {
        Some(JourneyRecordStop {
            tiploc: Tiploc::from_str(&s[2..9]).unwrap(),
            activity_flag: ActivityFlag::SetDownOnly, // As final stop
            _arrival_time: Some(SecondsPastMidnight::from_24hr_str(&s[10..14])),
            departure_time: None,
            is_first_stop: false,
        })
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
        // TODO: Handle all the other cases properly
        if s.is_empty() {
            return Ok(ActivityFlag::Neither);
        }
        if s.len() == 1 {
            match s {
                "T" => Ok(ActivityFlag::Both),
                "R" => Ok(ActivityFlag::Both), // "Request Stop" treated as Both
                "D" => Ok(ActivityFlag::SetDownOnly),
                "U" => Ok(ActivityFlag::PickUpOnly),
                _ => Ok(ActivityFlag::Neither),
            }
        } else {
            let first_two_chars = &s[0..2];
            match first_two_chars {
                "T " => Ok(ActivityFlag::Both),
                "R " => Ok(ActivityFlag::Both), // "Request Stop" treated as Both
                "D " => Ok(ActivityFlag::SetDownOnly),
                "U " => Ok(ActivityFlag::PickUpOnly),
                _ => Ok(ActivityFlag::Neither),
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ThreeAlphaCode(pub String);

impl FromStr for ThreeAlphaCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ThreeAlphaCode(s.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Tiploc(pub String);

impl FromStr for Tiploc {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err(())
        } else {
            Ok(Tiploc(s.trim().to_string()))
        }
    }
}
/// Denoted by "QL" in the CIF file
#[derive(Debug)]
pub struct Stop {
    pub tiploc: Tiploc,
    pub _nlc: String, // National Location Code
    pub _tps_description: String,
    pub stanox: String,
    pub three_alpha_code: Option<ThreeAlphaCode>,
    pub _nlc_description: String,
}

impl Stop {
    fn from_ti_str(ti_string: &str) -> Option<Self> {
        // Parse the QL string and extract the relevant fields
        let three_alpha_code_str = ti_string[53..56].trim();
        let three_alpha_code = if three_alpha_code_str.is_empty() {
            None
        } else {
            Some(ThreeAlphaCode::from_str(three_alpha_code_str).unwrap())
        };
        Some(Stop {
            tiploc: Tiploc::from_str(&ti_string[2..9]).unwrap(),
            _nlc: ti_string[11..17].trim().to_string(),
            _tps_description: ti_string[18..44].trim().to_string(),
            stanox: ti_string[44..49].trim().to_string(),
            three_alpha_code,
            _nlc_description: ti_string[56..72].trim().to_string(),
        })
    }
}
