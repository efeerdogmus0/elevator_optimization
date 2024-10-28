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
    accumulated_entity_probability: f32,
    total_entity_generated: u32,
    total_human_generated: u32,
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
            accumulated_entity_probability: 0.,
            total_entity_generated: 0,
            total_human_generated: 0,
        }
    }

    pub fn get_human_generated(&self) -> u32 {
        self.total_human_generated
    }

    pub fn get_entity_generated(&self) -> u32 {
        self.total_entity_generated
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
        &mut self, 
        time: u32,
        delta_time: f32,
        hour_length: f32,
    ) -> Vec<(usize, Box<dyn Boardable>)> {
        // this returns a list of (floor and the entity)

        let mut rng = rand::thread_rng();
        let mut generated_entities: Vec<(usize, Box<dyn Boardable>)> = Vec::new();

        // Determine the average number of passengers based on time
        let avg_passengers = self.avg_passenger_by_time
            .iter()
            .find(|&&(t, _)| t == time)
            .map(|&(_, count)| count)
            .unwrap_or(0.0);

        self.accumulated_entity_probability += avg_passengers * delta_time / hour_length;

        // eğer oluşan insan sayısı 1in altındaysa hiçbir şey yapma
        if self.accumulated_entity_probability < 1. {
            return generated_entities;
        }

        // Generate each passenger
        for _ in 0..self.accumulated_entity_probability as u32 {
            // Randomly choose a floor for the passenger
            let current_floor = rng.gen_range(0..self.floor_count);

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
                generated_entities.push((current_floor, e));
                self.accumulated_entity_probability -= 1.;
                self.total_entity_generated += 1;
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

    fn create_human_group(&mut self, current_floor: usize, time: u32) -> Option<Box<dyn Boardable>> {
        let destination_floor = Self::generate_destination(self.floor_count, current_floor);
        let mut members: Vec<Human> = Vec::new();
        let mut rng = rand::thread_rng();
        let group_size = rng.gen_range(2..3);

        // grup üye yaşları genelde birbirine yakın olur
        let age = Self::generate_age();
        for _ in 0..group_size {
            let gender = Self::generate_gender();

            let weight = match gender {
                Gender::Male => self.avg_male_weight.randomly_generate(age as u32),
                Gender::Female => self.avg_female_weight.randomly_generate(age as u32),
            }.unwrap();

            if self.will_use_elevator(age, time) {
                self.total_human_generated += 1;
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
