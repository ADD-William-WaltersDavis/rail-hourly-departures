mod records;

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
    for file in ["timetables_2025_Q4_Rail"] {
        let file_path = format!("{}/{}.cif", args.input_dir_path, file);
        raw_cif_text.push_str(&records::read_file(&file_path));
    }

    let record_lines = records::parse(raw_cif_text, "./config");
    println!("Records len: {:?}", record_lines.len());

    // write_json_file("pt_graph_walk".to_string(), &args.output_directory, &graph_ouput.pt_graph_walk)?;


    Ok(())
}
