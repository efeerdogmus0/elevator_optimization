// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use super::boardable::Boardable;
use super::human::{Human, Gender, HumanGroup};
use super::weight_distribution::WeightDistribution;
use super::commute_by_age::CommuteByAge;

use rand::Rng;
use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;


pub enum EntityType {
    Human,
    HumanGroup,
}

pub struct PopulationGenerator {
    avg_passenger_by_time: Vec<(u32, f32)>, // (time, avg passenger count)
    floor_count: usize,
    avg_female_weight: WeightDistribution,
    avg_male_weight: WeightDistribution,
    commute_by_age: CommuteByAge, // Field for commute probabilities by age
}

impl PopulationGenerator {
    pub fn new(
        floor_count: usize,
    ) -> Self {
        let avg_passenger_by_time = Self::parse_avg_passenger_by_time("data/avg_passengers_by_time.csv")
            .expect("average passenger by time file not found");
        let commute_by_age = CommuteByAge::new_from_file("data/commuting_activity_by_age.csv")
            .expect("commuting activity by age file not found");
        let avg_female_weight = WeightDistribution::read_weight_distribution("data/avg_female_weight.csv")
            .expect("average female weight distribution file not found");
        let avg_male_weight = WeightDistribution::read_weight_distribution("data/avg_male_weight.csv")
            .expect("average male weight distribution file not found");

        Self {
            avg_passenger_by_time,
            floor_count,
            avg_female_weight,
            avg_male_weight,
            commute_by_age,
        }
    }

    fn parse_avg_passenger_by_time(file_path: &str) -> Result<Vec<(u32, f32)>, Box<dyn Error>> {
        let mut avg_passenger_by_time = Vec::new();
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.records() {
            let record = result?;
            let hour: u32 = record[0].parse()?;
            let avg_passenger: f32 = record[1].parse()?;

            avg_passenger_by_time.push((hour, avg_passenger));
        }

        Ok(avg_passenger_by_time)
    }

    pub fn generate(
        &self, 
        time: u32,
        delta_time: f32,
        current_floor: usize,
    ) -> Vec<Box<dyn Boardable>> {
        let mut rng = rand::thread_rng();
        let mut generated_entities: Vec<Box<dyn Boardable>> = Vec::new();

        // Determine the average number of passengers based on time
        let avg_passengers = self.avg_passenger_by_time
            .iter()
            .find(|&&(t, _)| t == time)
            .map(|&(_, count)| count)
            .unwrap_or(0.0);

        let num_passengers = (avg_passengers * delta_time).round() as u32;

        // Generate each passenger
        for _ in 0..num_passengers {
            // Randomly choose between Human and HumanGroup
            let entity_type = if rng.gen::<f32>() < 0.7 {
                EntityType::Human
            } else {
                EntityType::HumanGroup
            };

            // Generate Human or HumanGroup based on selection, skipping if None
            let entity = match entity_type {
                EntityType::Human => self.create_human(current_floor, time),
                EntityType::HumanGroup => self.create_human_group(current_floor, time),
            };

            if let Some(e) = entity {
                generated_entities.push(e);
            }
        }

        generated_entities
    }

    fn generate_destination(floor_count: usize, current_floor: usize) -> usize {
        let mut rng = rand::thread_rng();
        let mut destination_floor = rng.gen_range(0..floor_count);
        while current_floor == destination_floor {
            destination_floor = rng.gen_range(0..floor_count);
        }
        destination_floor
    }

    fn generate_gender() -> Gender {
        if rand::thread_rng().gen_bool(0.5) { Gender::Male } else { Gender::Female }
    }

    fn generate_age() -> u8 {
        let mut rng = rand::thread_rng();
        rng.gen_range(21..80)
    }

    fn will_use_elevator(&self, age: u8, time: u32) -> bool {
        let probability = self.commute_by_age.get_probability(age, time);
        rand::thread_rng().gen::<f32>() < probability
    }

    fn create_human(&self, current_floor: usize, time: u32) -> Option<Box<dyn Boardable>> {
        let gender = Self::generate_gender();
        let age = Self::generate_age();

        // Assign weight based on gender
        let weight = match gender {
            Gender::Male => self.avg_male_weight.randomly_generate(age as u32),
            Gender::Female => self.avg_female_weight.randomly_generate(age as u32),
        }.unwrap();

        // Determine if this person uses the elevator based on age
        let uses_elevator = self.will_use_elevator(age, time);

        if uses_elevator {
            Some(Box::new(Human::new(
                age,
                gender,
                weight,
                0.5,
                Self::generate_destination(self.floor_count, current_floor),
            )))
        } else {
            None // Return None if the person doesn't want to commute
        }
    }

    fn create_human_group(&self, current_floor: usize, time: u32) -> Option<Box<dyn Boardable>> {
        let destination_floor = Self::generate_destination(self.floor_count, current_floor);
        let mut members: Vec<Human> = Vec::new();
        let mut rng = rand::thread_rng();
        let group_size = rng.gen_range(2..5);

        for _ in 0..group_size {
            let gender = Self::generate_gender();
            let age = Self::generate_age();

            let weight = match gender {
                Gender::Male => self.avg_male_weight.randomly_generate(age as u32),
                Gender::Female => self.avg_female_weight.randomly_generate(age as u32),
            }.unwrap();

            // Check if each group member wants to use the elevator
            if self.will_use_elevator(age, time) {
                members.push(Human::new(age, gender, weight, 10., destination_floor));
            }
        }

        // Only create HumanGroup if there are members who want to commute
        if members.is_empty() {
            None
        } else {
            Some(Box::new(HumanGroup::new(members)))
        }
    }
}
