use connectivity::{progress_bar_for_count, PublicTransportMode, RouteDirection, SecondsPastMidnight};
use fs_err::read_to_string;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Eq,
    hash::Hash,
    str::FromStr,
    collections::HashMap,
};

/// Parse a CIF file from raw text format into a vector of Records where 
/// each record corresponds to a line in the CIF file.
/// 
/// Each line in the raw data starts with a two-character identifier.
/// Can get more info on the CIF format here:
/// https://admin.opendatani.gov.uk/dataset/6d9677cf-8d03-4851-985c-16f73f7dd5fb/resource/6c9a6067-55e5-48c7-bcdd-164d50f2ce30/download/atco-cif-spec1.pdf
/// 
/// This function reads the file, splits it into lines and parses each line
/// into a Record structure based on its identifier.
/// 
/// Identifiers can be:
/// QS - Journey Header
/// QO - Journey Origin Stop
/// QI - Journey Intermediate Stop
/// QT - Journey Destination
/// QL - Stop Name
/// QB - Stop Location
/// QR - Journey Repetition Record (not used in basemap data)
/// 
/// An example of a timetable record in CIF format:
/// QSNNXMT3     20241007202410131111111  YEL       Metro           O
/// QO9400ZZTWSSH10000   T1  
/// QI9400ZZTWCHT200020002P   T1  
/// QI9400ZZTWTYD200040004P   T1  
/// QI9400ZZTWSIM200060006B   T1  
/// QI9400ZZTWBDE200080008P   T1  
/// QI9400ZZTWJRW200100010P   T1  
/// QI9400ZZTWHBN200150015S   T1  
/// QT9400ZZTWPLW20040   T1
/// 
/// An example of a stop name/location record in CIF format:
/// QLN9400ZZTWSSH1South Shields (Tyne and Wear Metro Station)      B        
/// QBN9400ZZTWSSH1436344  567224                          
/// 
pub fn parse(raw_cif_text: String, config_path: &str) -> Vec<Record> {
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
    let atco_mapping = read_mapping(config_path);

    cif_lines
        .par_iter()
        .progress_with(progress)
        .filter_map(|line| {
            let record_identifier = RecordIdentifier::from_str(&line[0..2]).unwrap();
            match record_identifier {
                RecordIdentifier::QS => {
                    Some(Record::JourneyHeader(JourneyHeader::from_qs_str(line)))
                }
                RecordIdentifier::QO => JourneyRecordStop::from_qo_str(line, &atco_mapping).map(Record::JourneyRecordStop),
                RecordIdentifier::QI => JourneyRecordStop::from_qi_str(line, &atco_mapping).map(Record::JourneyRecordStop),
                RecordIdentifier::QT => JourneyRecordStop::from_qt_str(line, &atco_mapping).map(Record::JourneyRecordStop),
                RecordIdentifier::QL => StopName::from_ql_str(line, &atco_mapping).map(Record::StopName),
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
    QS, // Journey Header
    QO, // Journey Origin Stop
    QI, // Journey Intermediate Stop
    QT, // Journey Destination Stop
    QL, // Stop Name
    Other, // For any other record denotion which we don't use
}

impl FromStr for RecordIdentifier {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "QS" => Ok(RecordIdentifier::QS),
            "QO" => Ok(RecordIdentifier::QO),
            "QI" => Ok(RecordIdentifier::QI),
            "QT" => Ok(RecordIdentifier::QT),
            "QL" => Ok(RecordIdentifier::QL),
            _ => Ok(RecordIdentifier::Other),
        }
    }
}

// One record per journey. A journey header may be immediately
// followed by optional sets of date running records and journey note
// records and should then be followed by a set of journey records
// (origin, intermediate, destination) giving a set of records that
// completely define dates, times, places, operator and vehicle type of
// the journey. The entire set of records relating to a single journey
// may be immediately followed by one or more journey repetition
// records.
/// Denoted by "QS" in the CIF file
#[derive(Debug, Clone)]
pub struct JourneyHeader {
    pub status: Status,
    pub _operator: String,
    pub _unique_journey_identifier: String,
    pub operating_days: OperatingDays,
    pub _route_number: String,
    pub _vehicle_type: PublicTransportMode,
    pub _route_direction: RouteDirection,
}

impl JourneyHeader {
    fn from_qs_str(qs_string: &str) -> Self {
        // Parse the QS string and extract the relevant fields
        JourneyHeader {
            status: Status::from_str(&qs_string[2..3]).unwrap(),
            _operator: qs_string[3..7].trim().to_string(),
            _unique_journey_identifier: qs_string[7..13].trim().to_string(),
            operating_days: OperatingDays::from_cif_str(&qs_string[29..36]),
            _route_number: qs_string[38..42].trim().to_string(),
            _vehicle_type: PublicTransportMode::from_str(&qs_string[48..56]).unwrap(),
            _route_direction: RouteDirection::from_str(&qs_string[64..65]).unwrap(),
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