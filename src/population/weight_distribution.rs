use csv::ReaderBuilder;
use rand::Rng;
use std::error::Error;
use std::fs::File;

#[derive(Debug)]
pub struct WeightDistribution {
    percentiles: [f32; 10], // Stores percentiles (5th, 10th, 15th, 25th, 50th, 75th, 95th)
}

impl WeightDistribution {
    // Method to randomly generate a weight based on percentiles
    pub fn randomly_generate(&self) -> f32 {
        let mut rng = rand::thread_rng();
        let random_percentile: f32 = rng.gen_range(0.0..1.0); // Random float between 0 and 1

        // Assign weight based on the percentile thresholds
        match random_percentile {
            x if x < 0.05 => self.percentiles[0],  // 5th percentile
            x if x < 0.10 => self.percentiles[1],  // 10th percentile
            x if x < 0.15 => self.percentiles[2],  // 15th percentile
            x if x < 0.25 => self.percentiles[3],  // 25th percentile
            x if x < 0.50 => self.percentiles[4],  // Median (50th percentile)
            x if x < 0.75 => self.percentiles[5],  // 75th percentile
            _ => self.percentiles[6],              // 95th percentile
        }
    }

    pub fn read_weight_distribution(file_path: &str) -> Result<WeightDistribution, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().delimiter(b';').has_headers(true).from_reader(file);

        if let Some(result) = rdr.records().next() {
            let record = result?;
            
            // Parse percentiles from CSV record
            let percentiles = [
                record[3].parse()?,  // 5th
                record[4].parse()?,  // 10th
                record[5].parse()?,  // 15th
                record[6].parse()?,  // 25th
                record[7].parse()?,  // 50th (Median)
                record[8].parse()?,  // 75th
                record[9].parse()?,  // 85th
                record[10].parse()?, // 90th
                record[11].parse()?, // 95th
                record[12].parse()?, // 95th
            ];

            return Ok(WeightDistribution {
                percentiles,
            });
        }

        Err("No records found in the CSV file".into())
    }
}

