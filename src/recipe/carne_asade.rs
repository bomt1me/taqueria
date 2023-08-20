use crate::{command::Command, event::Event, metadata};

use std::cmp::Ordering;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::thread;
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Read},
};

use super::{ParseRecipe, Recipe, RecipeParsed};

const DELIMITER: &str = ",";
const FILENAME: &str = "recipe.json";
const METADATA_FILENAME: &str = "metadata.json";
const GUACAMOLE_START_SLICE: &str = ",\"guacamole\":[[";
const GUACAMOLE_END_SLICE: &str = "]]}";
const CARNE_ASADA_MAGIC_NUMBER: &str = "CARNE1.0";
const MAX_THREADS: u32 = 8;
const MAX_BYTES: u32 = 102_400;
const DTYPE: u32 = 2;

const STEPS_BY_NAME: [&str; 20] = [
    "Unknown",
    "Generic Bipolar",
    "X",
    "Y",
    "Z",
    "I",
    "II",
    "III",
    "VR",
    "VL",
    "VF",
    "V1",
    "V2",
    "V3",
    "V4",
    "V5",
    "V6",
    "ES",
    "AS",
    "AI",
];

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Header {
    size: u32,
    date_of_recipe: chrono::NaiveDate,
    time_of_recipe: chrono::NaiveTime,
    number_of_steps: u16,
    steps: [usize; 12],
    unit_conversion: [i32; 12],
    granularity: u16,
}

impl Header {
    #[must_use]
    pub fn parse_date_of_recipe(buffer: &[u8; 512]) -> chrono::NaiveDate {
        let day: u32 = u32::from(u16::from_le_bytes(
            buffer[128..130].try_into().expect("Could not parse day."),
        ));
        let month: u32 = u32::from(u16::from_le_bytes(
            buffer[130..132].try_into().expect("Could not parse month."),
        ));
        let year: i32 = i32::from(u16::from_le_bytes(
            buffer[132..134].try_into().expect("Could not parse year."),
        ));
        chrono::NaiveDate::from_ymd_opt(year, month, day).expect("Could not parse date.")
    }

    #[must_use]
    pub fn parse_time_of_recipe(buffer: &[u8; 512]) -> chrono::NaiveTime {
        let hour: u32 = u32::from(u16::from_le_bytes(
            buffer[140..142].try_into().expect("Could not parse hour."),
        ));
        let min: u32 = u32::from(u16::from_le_bytes(
            buffer[142..144].try_into().expect("Could not parse min."),
        ));
        let sec: u32 = u32::from(u16::from_le_bytes(
            buffer[144..146].try_into().expect("Could not parse sec."),
        ));
        chrono::NaiveTime::from_hms_opt(hour, min, sec).expect("Could not parse time.")
    }

    #[must_use]
    pub fn parse_steps(buffer: &[u8; 512]) -> [usize; 12] {
        let mut steps: [usize; 12] = [0; 12];
        for i in 0..steps.len() {
            let val = usize::from(u16::from_le_bytes(
                buffer[(148 + i * 2)..(148 + (i * 2) + 2)]
                    .try_into()
                    .expect("Could not create step."),
            ));
            if val < STEPS_BY_NAME.len() {
                steps[i] = val;
            } else {
                steps[i] = 0;
            }
        }
        steps
    }

    #[must_use]
    pub fn parse_unit_conversion(buffer: &[u8; 512]) -> [i32; 12] {
        let mut conversions = [-9_i32; 12];
        for i in 0..conversions.len() {
            conversions[i] = i32::from(i16::from_le_bytes(
                buffer[(196 + (i * 2))..(196 + (i * 2) + 2)]
                    .try_into()
                    .expect("Could not create conversion."),
            ));
        }
        conversions
    }

    #[must_use]
    pub fn parse(buffer: &[u8; 512]) -> Self {
        Self {
            size: u32::from_le_bytes(buffer[4..8].try_into().expect("Could not create size.")),
            date_of_recipe: Self::parse_date_of_recipe(buffer),
            time_of_recipe: Self::parse_time_of_recipe(buffer),
            number_of_steps: (u16::from(buffer[146])) + (u16::from(buffer[147]) << 8),
            steps: Self::parse_steps(buffer),
            unit_conversion: Self::parse_unit_conversion(buffer),
            granularity: u16::from(buffer[262]) + (u16::from(buffer[263]) << 8),
        }
    }
}

pub struct CarneAsada {}

impl CarneAsada {
    #[must_use]
    pub fn calculate_chunks(
        total_bytes: u32,
        max_bytes: u32,
        max_threads: u32,
        step_count: u32,
        sample_size: u32,
    ) -> Vec<Vec<(u32, u32)>> {
        let mut chunks = Vec::<Vec<(u32, u32)>>::new();

        if (step_count * sample_size) < 1 {
            return chunks;
        }

        if max_threads < 1 {
            return chunks;
        }

        let adjusted_max_bytes: u32 =
            (max_bytes / (step_count * sample_size)) * (step_count * sample_size);
        let mut total_iterations: u32 = total_bytes / (adjusted_max_bytes * max_threads);
        if total_bytes % (adjusted_max_bytes * max_threads) > 0 {
            total_iterations += 1;
        }

        let mut counter = 0_u32;
        for _ in 0..total_iterations {
            let mut thread_chunks = Vec::<(u32, u32)>::new();
            for _ in 0..max_threads {
                if counter + adjusted_max_bytes > total_bytes {
                    thread_chunks.push((counter, total_bytes));
                    counter += total_bytes - counter;
                    break;
                }

                thread_chunks.push((counter, counter + adjusted_max_bytes));
                counter += adjusted_max_bytes;
            }
            chunks.push(thread_chunks);
        }
        chunks
    }
}

impl Recipe for CarneAsada {
    fn parse(&self, command: &Command<ParseRecipe>) -> Option<Event<RecipeParsed>> {
        let dir = std::sync::Arc::new(
            std::path::Path::new(&command.payload.basepath)
                .join(command.payload.identifier.to_string()),
        );
        fs::create_dir(dir.as_ref()).expect("Could not create directory.");

        let file: File = File::open(&command.payload.filepath).expect("Could not open file.");
        let mut reader: BufReader<File> = BufReader::new(file);
        let mut buffer: [u8; 10] = [0u8; 10];
        reader
            .read_exact(&mut buffer)
            .expect("Could not read magic header buffer");
        if std::str::from_utf8(&buffer[..8]).expect("Invalid magic number.")
            != CARNE_ASADA_MAGIC_NUMBER
        {
            return None;
        }
        let mut header_buffer: [u8; 512] = [0u8; 512];
        reader
            .read_exact(&mut header_buffer)
            .expect("Could not read header buffer");
        let header_data: Header = Header::parse(&header_buffer);
        let metadata = metadata::Metadata {
            size: header_data.size,
            date_of_recipe: header_data.date_of_recipe,
            time_of_recipe: header_data.time_of_recipe,
            number_of_steps: header_data.number_of_steps,
            steps: header_data
                .steps
                .to_vec()
                .iter()
                .filter_map(|i| {
                    if i > &0 {
                        Some(String::from(STEPS_BY_NAME[*i]))
                    } else {
                        None
                    }
                })
                .collect(),
            units: 1.0,
            granularity: header_data.granularity,
        };
        metadata.store(dir.join(METADATA_FILENAME));

        let step_count = u32::from(header_data.number_of_steps);
        let total_bytes = header_data.size * step_count * 2;
        let chunks = Self::calculate_chunks(total_bytes, MAX_BYTES, MAX_THREADS, step_count, DTYPE);

        let mut generated_files = Vec::<(usize, u32, String)>::new();
        for chunk in chunks {
            let mut threads = Vec::new();
            for thread in chunk {
                let dir = dir.clone();
                let filepath = command.payload.filepath.clone();
                threads.push(thread::spawn(move || {
                    CarneAsadaGaucamole::parse_guacamole(
                        &dir,
                        &filepath,
                        thread.0,
                        thread.1,
                        522,
                        header_data.number_of_steps,
                        &header_data.unit_conversion,
                    )
                }));
            }
            for t in threads {
                generated_files.append(&mut t.join().expect("Could not join thread."));
            }
        }

        generated_files.sort_by(|a, b| match a.0.cmp(&b.0) {
            Ordering::Equal => a.1.cmp(&b.1),
            other => other,
        });

        CarneAsadeFile::merge(&dir, &generated_files);

        Some(Event {
            event_type: 0,
            payload: RecipeParsed {
                output: dir.join(FILENAME),
            },
        })
    }

    fn identifier(&self) -> String {
        String::from("carne_asada")
    }
}

pub struct CarneAsadeFile {}

impl CarneAsadeFile {
    #[must_use]
    pub fn read_chunk(filepath: &String, onset: u32, offset: u32, start: u32) -> Vec<u8> {
        let file: File = File::open(filepath).expect("Could not open file.");
        let mut reader: BufReader<File> = BufReader::new(file);
        let mut buffer = vec![
            0;
            usize::try_from(offset - onset)
                .expect("Could not create buffer from onset-offset.")
        ];
        reader
            .seek(SeekFrom::Start((start + onset).into()))
            .expect("Could not seek.");
        reader.read_exact(&mut buffer).expect("Could not read.");
        buffer
    }

    #[must_use]
    pub fn write_chunk(
        dir: &Path,
        onset: u32,
        offset: u32,
        step: usize,
        guac: &Vec<f64>,
    ) -> (usize, u32, String) {
        let filename = format!("{onset}_{offset}_{step}.json");
        let file = File::create(dir.join(&filename)).expect("Could not create file.");
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &guac).expect("Could not write json.");
        (step, onset, filename)
    }

    pub fn merge(dir: &Path, files: &[(usize, u32, String)]) {
        let mut custom = String::from(GUACAMOLE_START_SLICE).as_bytes().to_vec();
        let mut header = fs::read(dir.join(METADATA_FILENAME)).expect("Could not reader metadata.");
        header.remove(header.len() - 1);
        header.append(&mut custom);
        std::fs::write(dir.join(FILENAME), &header).expect("Could not copy header.");
        let mut recipe_file = std::fs::OpenOptions::new()
            .append(true)
            .open(dir.join(FILENAME))
            .expect("Could not open in append mode.");
        let mut current_step: usize = usize::MAX;
        for f in files {
            if current_step == usize::MAX {
                current_step = f.0;
            }

            if f.0 != current_step {
                recipe_file
                    .write_all(String::from("],[").as_bytes())
                    .expect("Could not write new step.");
                current_step = f.0;
            }

            if f.1 > 0 {
                recipe_file
                    .write_all(DELIMITER.as_bytes())
                    .expect("Could not write delimiter.");
            }

            let r = fs::read(dir.join(&f.2)).expect("Could not read guac.");
            recipe_file
                .write_all(&r[1..r.len() - 1])
                .expect("Could not write guac.");
        }
        recipe_file
            .write_all(String::from(GUACAMOLE_END_SLICE).as_bytes())
            .expect("Could not write recipe end slice.");
    }
}

pub struct CarneAsadaGaucamole {}

impl CarneAsadaGaucamole {
    #[must_use]
    pub fn parse_guacamole(
        dir: &std::path::Path,
        filepath: &String,
        onset: u32,
        offset: u32,
        start: u32,
        number_of_steps: u16,
        unit_conversion: &[i32; 12],
    ) -> Vec<(usize, u32, String)> {
        let buffer = CarneAsadeFile::read_chunk(filepath, onset, offset, start);
        let mut guac = vec![
            vec![
                0.0;
                ((offset - onset) / u32::from(number_of_steps) / DTYPE)
                    .try_into()
                    .expect("")
            ];
            usize::from(number_of_steps)
        ];
        for (idx, sample) in buffer.chunks(2).enumerate() {
            let step: usize = idx % usize::from(number_of_steps);
            let val = u16::from_le_bytes(sample[0..2].try_into().expect(""));
            let step_idx: usize = idx / usize::from(number_of_steps);
            if val > 32767_u16 {
                guac[step][step_idx] = (f64::from(val) - 65536_f64)
                    * f64::from(unit_conversion[step])
                    * 10_f64.powi(-6);
            } else {
                guac[step][step_idx] =
                    f64::from(val) * f64::from(unit_conversion[step]) * 10_f64.powi(-6);
            }
        }
        let mut generated_files = Vec::<(usize, u32, String)>::new();
        for (i, g) in guac.iter().enumerate() {
            generated_files.push(CarneAsadeFile::write_chunk(dir, onset, offset, i, g));
        }
        generated_files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_given_empty_then_no_epics() {
        let chunks = CarneAsada::calculate_chunks(0, 0, 0, 0, 0);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_given_size_less_than_max_then_one_epic() {
        let total_bytes = 10000;
        let max_bytes = 1024;
        let steps = 3;
        let sample_size = 2;
        let max_threads = 3;
        let chunks =
            CarneAsada::calculate_chunks(total_bytes, max_bytes, max_threads, steps, sample_size);
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].len(), 3);
        assert_eq!(chunks[0][2], (2040, 3060));
    }
}
