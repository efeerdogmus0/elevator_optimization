// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

pub struct CommuteByAge {
    probabilities: HashMap<String, [f32; 3]>, // [Morning, Midday, Evening] probabilities
}

impl CommuteByAge {
    pub fn new_from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut probabilities = HashMap::new();
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.records() {
            let record = result?;
            let age_group = record[0].to_string();
            let morning: f32 = record[1].parse()?;
            let midday: f32 = record[2].parse()?;
            let evening: f32 = record[3].parse()?;

            probabilities.insert(age_group, [morning, midday, evening]);
        }

        Ok(CommuteByAge { probabilities })
    }

    pub fn get_probability(&self, age: u8, time: u32) -> f32 {
        let age_group = match age {
            15..=24 => "15-24",
            25..=54 => "25-54",
            _ => "55+",
        };

        let time_of_day = match time {
            6..=9 => 0,      // Morning
            11..=14 => 1,    // Midday
            16..=19 => 2,    // Evening
            _ => return 0.0, // Outside commuting times
        };

        self.probabilities
            .get(age_group)
            .map(|times| times[time_of_day])
            .unwrap_or(0.0)
    }
}
