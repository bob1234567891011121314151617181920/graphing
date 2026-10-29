use serde::{Deserialize, Serialize};
use std::fs;

pub const DATA_PATH: &str = "assets/data.json";

//what inside the json file looks like
#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default)]
    pub price: f32,
}

pub fn load_items(path: &str) -> Vec<Item> {
    let json = fs::read_to_string(path).unwrap_or_else(|e| panic!("can't read {path}: {e}"));
    serde_json::from_str(&json).unwrap_or_else(|e| panic!("can't parse {path}: {e}"))
}

pub fn save_items(path: &str, items: &[Item]) {
    //later will use a better way to notift the user that the save was successful
    println!("Saving graph to {path}");
    let json = serde_json::to_string_pretty(items).unwrap();
    fs::write(path, json).unwrap_or_else(|e| panic!("can't write {path}: {e}"));
}
