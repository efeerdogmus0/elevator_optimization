// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use rand::Rng;
use super::boardable::Boardable;
use super::human::{ Human, Gender, HumanGroup };

pub enum EntityType {
    Human,
    HumanGroup,
    // Additional types can be added here
}

impl EntityType {
    // Method to get the probability for each entity type and validate the total sum
    fn probability(&self) -> f32 {
        // Define individual probabilities
        let human_prob = 0.7;
        let human_group_prob = 0.3;

        // List of all probabilities
        let total_probability: f32 = human_prob + human_group_prob;
        assert!(
            (total_probability - 1.0).abs() < f32::EPSILON,
            "Probabilities do not add up to 1.0"
        );

        match self {
            EntityType::Human => human_prob,
            EntityType::HumanGroup => human_group_prob,
        }
    }

    // Method to return a list of all entity types with probabilities
    fn all_types() -> Vec<(EntityType, f32)> {
        vec![
            (EntityType::Human, EntityType::Human.probability()),
            (EntityType::HumanGroup, EntityType::HumanGroup.probability()),
        ]
    }
}

// Struct for the PopulationGenerator
pub struct PopulationGenerator {
    entities_per_time: Vec<(u32, u32)>, // (time, count) pairs for entity generation count
    floor_count: usize,
}

impl PopulationGenerator {
    pub fn new(
        entities_per_time: Vec<(u32, u32)>,
        floor_count: usize,
    ) -> Self {
        Self {
            entities_per_time,
            floor_count,
        }
    }

    // Generate entities based on time and probability
    pub fn generate(
        &self, 
        time: u32,
        current_floor: usize,
    ) -> Vec<Box<dyn Boardable>> {

        let mut rng = rand::thread_rng();
        let mut generated_entities: Vec<Box<dyn Boardable>> = Vec::new();

        // Get the number of entities to generate based on time
        let entity_count = self.entities_per_time.iter()
            .find(|&&(t, _)| t == time)
            .map(|&(_, count)| count)
            .unwrap_or(0);

        for _ in 0..entity_count {
            // Select an entity type based on probability
            let rand_val: f32 = rng.gen();
            let mut cumulative_probability = 0.0;

            for (entity_type, probability) in EntityType::all_types() {
                cumulative_probability += probability;

                if rand_val <= cumulative_probability {
                    let entity = match entity_type {
                        EntityType::Human => self.create_human(self.floor_count, current_floor),
                        EntityType::HumanGroup => self.create_human_group(self.floor_count, current_floor),
                    };
                    generated_entities.push(entity);
                    break;
                }
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

    fn generate_gender() -> Gender{
        let mut rng = rand::thread_rng();
        if rng.gen_range(0..2) == 0 { Gender::Male } else { Gender::Female }
    }

    fn generate_age() -> u8 {
        let mut rng = rand::thread_rng();
        rng.gen_range(18..80)
    }

    fn create_human(&self, floor_count: usize, current_floor: usize) -> Box<dyn Boardable> {
        Box::new(Human::new(
                Self::generate_age(),
                Self::generate_gender(),
                Self::generate_destination(floor_count, current_floor),
            )
        )
    }

    // Function to create a HumanGroup
    fn create_human_group(&self, floor_count: usize, current_floor: usize) -> Box<dyn Boardable> {
        let mut rng = rand::thread_rng();
        let destination_floor = Self::generate_destination(floor_count, current_floor);

        let mut members: Vec<Human> = Vec::new();
        for _ in 0..rng.gen_range(2..5) {
            members.push(Human::new(
                    Self::generate_age(),
                    Self::generate_gender(),
                    Self::generate_destination(floor_count, current_floor),
                )
            )
        }

        Box::new(HumanGroup::new(members))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_weight() {
        let pop_gen = PopulationGenerator::new(vec![(1, 1)], 10);
        let human = pop_gen.create_human(10, 0);
        
        assert!(human.get_weight() > 50.0 && human.get_weight() < 100.0);
    }

    #[test]
    fn test_human_destination() {
        let pop_gen = PopulationGenerator::new(vec![(1, 1)], 10);
        let human = pop_gen.create_human(10, 0);
        
        assert!(human.get_destination() < 10);
    }

    #[test]
    fn test_generate_human_group_weight() {
        let pop_gen = PopulationGenerator::new(vec![(1, 1)], 10);
        let human_group = pop_gen.create_human_group(10, 0);

        assert!(human_group.get_weight() > 100.0);
    }

    #[test]
    fn test_generate_destination() {
        let floor_count = 10;
        let current_floor = 5;
        let destination = PopulationGenerator::generate_destination(floor_count, current_floor);

        // Ensure destination is within bounds and not the same as the current floor
        assert!(destination < floor_count);
        assert_ne!(destination, current_floor);
    }

    #[test]
    fn test_generate_gender() {
        // Check if gender generation is balanced
        let mut male_count = 0;
        let mut female_count = 0;
        
        for _ in 0..1000 {
            match PopulationGenerator::generate_gender() {
                Gender::Male => male_count += 1,
                Gender::Female => female_count += 1,
            }
        }

        println!("male: {}, female: {}", male_count, female_count);
        assert!(male_count > 350 && female_count > 350); // Expected to be somewhat balanced
    }

    #[test]
    fn test_population_generation() {
        // Test population generation based on time and probability
        let pop_gen = PopulationGenerator::new(vec![(1, 10)], 10);
        let entities = pop_gen.generate(1, 0);

        // Ensure correct number of entities generated
        assert_eq!(entities.len(), 10);

        // Check that generated entities are valid Boardable types
        for entity in entities {
            assert!(entity.get_area() > 0.0);
            assert!(entity.get_weight() > 0.0);
            assert!(entity.get_destination() < 10);
        }
    }
}
