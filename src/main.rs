mod criteria;
mod hour_grouping;
mod public_transport_mode;
mod records;
mod stops;
mod utils;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(long)]
    input_file_dir: String,
    #[clap(long, default_value = "tuesday")]
    operating_day: records::Day,
    #[clap(long)]
    output_directory: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut raw_cif_text = String::new();
    for file in ["Bus", "Rail", "SubwayMetro", "TramStreetcarLightRail"] {
        raw_cif_text.push_str(&records::read_file(&format!("{}/{}.cif", &args.input_file_dir, file)));
    }

    let record_lines = records::parse(raw_cif_text, "./config");
    println!("Records len: {:?}", record_lines.len());

    let lookup = stops::create_lookup(&record_lines);
    utils::write_json_file(
        "atco_stopname_lookup".to_string(),
        &args.output_directory,
        &lookup,
    )?;

    let hourly_departures = hour_grouping::group(record_lines, &lookup, args.operating_day);
    let criteria_results = criteria::evaluate_criteria(&hourly_departures);
    utils::write_json_file(
        "rail_hourly_departures".to_string(),
        &args.output_directory,
        &criteria_results,
    )?;
    Ok(())
}
