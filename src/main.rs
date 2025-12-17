mod hour_grouping;
mod records;
mod stops;

use anyhow::Result;
use clap::Parser;
use connectivity::write_json_file;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    input_dir_path: String,
    #[clap(long, default_value = "tuesday")]
    operating_day: records::Day,
    #[clap(long)]
    output_directory: String,
}

// #[rustfmt::skip]
fn main() -> Result<()> {
    let args = Args::parse();

    let mut raw_cif_text = String::new();
    // for file in ["timetables_2025_Q4_Rail", "timetables_2025_Q4_SubwayMetro", "timetables_2025_Q4_TramStreetcarLightRail"] {
    let file_path = format!("{}/{}.cif", args.input_dir_path, "timetables_2025_Q4_Rail");
    raw_cif_text.push_str(&records::read_file(&file_path));
    // }

    let record_lines = records::parse(raw_cif_text, "./config");
    println!("Records len: {:?}", record_lines.len());
    
    let lookup = stops::create_lookup(&record_lines);
    write_json_file("atco_stopname_lookup".to_string(), &args.output_directory, &lookup)?;

    let hourly_departures = hour_grouping::group(record_lines, args.operating_day);
    write_json_file(
        "rail_hourly_departures".to_string(),
        &args.output_directory,
        &hourly_departures,
    )?;

    Ok(())
}
