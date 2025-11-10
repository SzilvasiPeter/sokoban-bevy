use quick_xml::de::from_str;
use serde::Deserialize;
use std::fs::File;

#[derive(Deserialize)]
struct SokobanLevels {
    #[serde(rename = "LevelCollection")]
    level_collection: LevelCollection,
}

#[derive(Deserialize)]
struct LevelCollection {
    #[serde(rename = "Level")]
    levels: Vec<Level>,
}

#[derive(Deserialize)]
struct Level {
    #[serde(rename = "L")]
    lines: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = std::fs::read_to_string("levels/microban.slc")?;
    let data: SokobanLevels = from_str(&xml)?;
    let levels: Vec<_> = data
        .level_collection
        .levels
        .into_iter()
        .map(|l| l.lines)
        .collect();
    serde_json::to_writer_pretty(
        File::create("levels/microban.json")?,
        &serde_json::json!({ "levels": levels }),
    )?;
    Ok(())
}
