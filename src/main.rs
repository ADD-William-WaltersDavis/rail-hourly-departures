mod criteria;
mod hour_grouping;
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
    #[clap(long, value_parser = records::parse_date)]
    operating_week: records::Date,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let selected_date = records::Date(260113);

    let raw_cif_text = records::read_file(&format!(
        "{}/{}.CIF",
        &args.input_file_dir, "CIF_ALL_FULL_DAILY_toc-full"
    ));

    let record_lines = records::parse(raw_cif_text);
    println!("Records len: {:?}", record_lines.len());

    let gb_station_three_alpha_codes: Vec<records::ThreeAlphaCode> = utils::read_json_file(
        "config/gb_station_three_alpha_codes.json".to_string(),
    )?;
    let lookup = stops::create_lookup(&record_lines, &gb_station_three_alpha_codes);

    let hourly_departures =
        hour_grouping::group(record_lines, &lookup, args.operating_day, selected_date);
    let criteria_results = criteria::evaluate_criteria(&hourly_departures);
    utils::write_json_file(
        "rail_hourly_departures".to_string(),
        &args.output_directory,
        &criteria_results,
    )?;
    Ok(())
}
