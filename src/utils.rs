use anyhow::Result;
use fs_err::File;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Serialize, de::DeserializeOwned};
use std::io::{BufReader, BufWriter, Write};

/// Creates a progress bar for monitoring function progress.
pub fn progress_bar_for_count(count: usize) -> ProgressBar {
    ProgressBar::new(count as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap())
}

pub fn write_json_file<T: Serialize>(
    file_name: String,
    output_directory: &str,
    data: T,
) -> Result<()> {
    let path = format!("{output_directory}/{file_name}.json");
    println!("Writing to {path}");
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &data)?;
    writer.flush()?;
    Ok(())
}

pub fn read_json_file<T: DeserializeOwned>(path: String) -> Result<T> {
    if !path.ends_with(".json") && !path.ends_with(".geojson") {
        return Err(anyhow::anyhow!(
            "read_json_file needs {} to end with .json or .geojson",
            path
        ));
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}
