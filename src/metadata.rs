use std::{fs::File, io::BufWriter, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub size: u32,
    pub date_of_recipe: chrono::NaiveDate,
    pub time_of_recipe: chrono::NaiveTime,
    pub number_of_steps: u16,
    pub steps: Vec<String>,
    pub units: f32,
    pub granularity: u16,
}

impl Metadata {
    pub fn store(&self, path: PathBuf) {
        let file = File::create(path).expect("Could not create file.");
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self).expect("Could not write json.");
    }
}
