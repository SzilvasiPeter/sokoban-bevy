use quick_xml::de::from_str;
use serde::Deserialize;

use std::fs;

#[derive(Debug, Deserialize)]
pub struct LevelCollection {
    #[serde(rename = "Level")]
    pub levels: Vec<Level>,
}

#[derive(Debug, Deserialize)]
pub struct Level {
    #[serde(rename = "L")]
    pub lines: Vec<String>,
}

pub fn load_levels(path: &'static str) -> Result<Vec<Level>, Box<dyn std::error::Error>> {
    let xml = fs::read_to_string(path)?;

    let start = xml.find("<LevelCollection").unwrap();
    let end = xml.rfind("</LevelCollection>").unwrap() + "</LevelCollection>".len();
    let level_xml = &xml[start..end];

    let collection: LevelCollection = from_str(level_xml)?;

    Ok(collection.levels)
}
