// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use csv::ReaderBuilder;
use rand::Rng;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug)]
pub struct WeightDistribution {
    distributions: HashMap<(u32, u32), [f32; 9]>, // Stores percentiles by age range (min_age, max_age)
}

impl WeightDistribution {
    /// Generates a random weight based on the specified age and corresponding percentiles.
    pub fn randomly_generate(&self, age: u32) -> Result<f32, &str> {
        // Find the correct age range for the provided age
        let percentiles = self
            .distributions
            .iter()
            .find(|&((min_age, max_age), _)| age >= *min_age && age <= *max_age)
            .map(|(_, percentiles)| percentiles);

        let percentiles = match percentiles {
            Some(p) => p,
            None => return Err("No matching age range found"),
        };

        let mut rng = rand::thread_rng();
        let random_percentile: f32 = rng.gen_range(0.0..1.0); // Random float between 0 and 1

        // Select weight based on the percentile thresholds
        let weight = match random_percentile {
            x if x < 0.05 => percentiles[0],  // 5th percentile
            x if x < 0.10 => percentiles[1],  // 10th percentile
            x if x < 0.15 => percentiles[2],  // 15th percentile
            x if x < 0.25 => percentiles[3],  // 25th percentile
            x if x < 0.50 => percentiles[4],  // Median (50th percentile)
            x if x < 0.75 => percentiles[5],  // 75th percentile
            x if x < 0.85 => percentiles[6],  // 85th percentile
            x if x < 0.90 => percentiles[7],  // 90th percentile
            _ => percentiles[8],              // 95th percentile
        };

        Ok(weight)
    }

    /// Reads the weight distribution data from a CSV file and populates the distributions map.
    pub fn read_weight_distribution(file_path: &str) -> Result<WeightDistribution, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().delimiter(b';').has_headers(true).from_reader(file);

        let mut distributions = HashMap::new();

        for result in rdr.records() {
            let record = result?;

            // Extract the age range as a tuple (min_age, max_age)
            let min_age: u32 = record[0].parse()?;
            let max_age: u32 = record[1].parse()?;
            let age_range = (min_age, max_age);

            // Parse the percentiles from the CSV record
            let percentiles = [
                record[4].parse()?,  // 5th percentile
                record[5].parse()?,  // 10th percentile
                record[6].parse()?,  // 15th percentile
                record[7].parse()?,  // 25th percentile
                record[8].parse()?,  // 50th percentile (Median)
                record[9].parse()?,  // 75th percentile
                record[10].parse()?, // 85th percentile
                record[11].parse()?, // 90th percentile
                record[12].parse()?, // 95th percentile
            ];

            // Insert the age range and its percentiles into the distributions map
            distributions.insert(age_range, percentiles);
        }

        Ok(WeightDistribution { distributions })
    }
}
