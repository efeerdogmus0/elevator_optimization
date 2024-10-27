// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use crate::machine::elevator::{ self, Elevator, ElevatorParameters };
use super::elevator_system_parameters::ElevatorSystemParameters;
use crate::population::{ Boardable, PopulationGenerator };

use std::time::Instant;
use std::error::Error;


pub struct ElevatorSystem {
    floor_heights: Vec<f32>,
    queue: Vec<Vec<Box<dyn Boardable>>>, 
    pop_gen: PopulationGenerator,
    elevators: Vec<Elevator>,
    last_update: Instant,
    time_multiplier: f32,
    gravity: f32,
}


impl ElevatorSystem {
    pub fn from_file(
        file_path: &str,        
    ) -> Result<Self, Box<dyn Error>> {
        let parameters = ElevatorSystemParameters::from_file(file_path)?;
        Ok(Self::new(parameters))
    }

    pub fn new(
        parameters: ElevatorSystemParameters,
    ) -> Self {
        let mut elevators = Vec::new();
        for file in &parameters.elevators {
            elevators.push(
                Elevator::new(
                    parameters.floors.clone(),
                    parameters.gravity,
                    ElevatorParameters::from_file(file).unwrap()
                )
            );
        }
        let floor_count = parameters.floors.len();
        Self {
            floor_heights: parameters.floors,
            queue: Vec::with_capacity(floor_count),
            pop_gen: PopulationGenerator::new(vec![(0,0)], floor_count),
            elevators,
            last_update: Instant::now(),
            time_multiplier: parameters.time_multiplier,
            gravity: parameters.gravity,
        }
    }

    pub fn get_used_energy(&self) -> f32 {
        let mut total = 0.0;
        for elevator in &self.elevators {
            total += elevator.get_used_energy();
        }
        total
    }

    fn get_delta_time(&mut self) -> f32 {
        // calculate delta time
        let now = Instant::now();
        let mut delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        delta_time *= self.time_multiplier;
        
        delta_time
    }

    pub fn load_elevators(&mut self) {
        for idx in 0..self.elevators.len() {
            self.load_elevator(idx);
        }
    }

    pub fn load_elevator(&mut self, elevator_idx: usize) {
        let elevator = &mut self.elevators[elevator_idx];
        if !elevator.can_board() {
            return;
        }

        match elevator.get_current_floor() {
            Some(floor) => {
                let mut idx = 0;
                while idx < self.queue[floor].len() {
                    if elevator.can_fit(&self.queue[floor][idx]) {
                        let entity = self.queue[floor].remove(idx);
                        elevator.load(entity);
                    } else { idx += 1; }
                }
            },
            _ => {},
        };
    }

    pub fn update_population(&mut self) {
        for floor in 0..self.floor_heights.len() {
            let entities = self.pop_gen.generate(0, floor);
            self.queue[floor].extend(entities);
        }
    }

    pub fn update(&mut self) {
        // give birth to new homo sapiens
        self.update_population();

        let delta_time = self.get_delta_time();
        for elevator in &mut self.elevators {
            elevator.update(delta_time);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::population::{ Boardable, Human, Gender };

    #[test]
    fn test_elevator_system_initialization() {
        let parameters = ElevatorSystemParameters {
            floors: vec![0.0, 3.0, 6.0, 9.0], // Four floors with 3 meters between each
            elevators: vec!["param/elevator_test_parameters.yaml".into()], // Mock file paths
            time_multiplier: 1.0,
            gravity: 9.81,
        };
        
        let system = ElevatorSystem::new(parameters);

        assert_eq!(system.floor_heights.len(), 4);
        assert_eq!(system.elevators.len(), 1); // One elevator initialized
    }

    #[test]
    fn test_energy_calculation() {
        let parameters = ElevatorSystemParameters {
            floors: vec![0.0, 3.0, 6.0],
            elevators: vec!["param/elevator_test_parameters.yaml".into()],
            time_multiplier: 1.0,
            gravity: 9.81,
        };

        let mut system = ElevatorSystem::new(parameters);

        // Mock energy consumption for testing
        // system.elevators[0].use_energy(10.0); // Assume 10 units of energy used

        assert_eq!(system.get_used_energy(), 10.0);
    }

    #[test]
    fn test_update_population() {
        let parameters = ElevatorSystemParameters {
            floors: vec![0.0, 3.0, 6.0],
            elevators: vec!["param/elevator_test_parameters.yaml".into()],
            time_multiplier: 1.0,
            gravity: 9.81,
        };
        let mut system = ElevatorSystem::new(parameters);
        system.update_population();

        // Check that population is updated and not empty on each floor
        for queue in &system.queue {
            assert!(!queue.is_empty());
        }
    }

    #[test]
    fn test_load_elevator() {
        let parameters = ElevatorSystemParameters {
            floors: vec![0.0, 3.0, 6.0],
            elevators: vec!["param/elevator_test_parameters.yaml".into()],
            time_multiplier: 1.0,
            gravity: 9.81,
        };
        let mut system = ElevatorSystem::new(parameters);

        // Mock floor population and elevator loading behavior
        let mut human = Box::new(Human::new(30, Gender::Male, 2)); // Human destined for floor 2
        system.queue[0].push(human);

        system.load_elevator(0);
        assert!(system.queue[0].is_empty(), "Queue should be empty after loading");
        assert!(system.elevators[0].get_entity_count() > 0, "Elevator should contain boarded entities");
    }

    #[test]
    fn test_get_delta_time() {
        let parameters = ElevatorSystemParameters {
            floors: vec![0.0, 3.0, 6.0],
            elevators: vec!["param/elevator_test_parameters.yaml".into()],
            time_multiplier: 2.0, // Speed up time
            gravity: 9.81,
        };

        let mut system = ElevatorSystem::new(parameters);
        std::thread::sleep(Duration::from_millis(500));
        
        let delta_time = system.get_delta_time();
        assert!(delta_time >= 1.0 && delta_time < 1.1); // Roughly 1 second with time_multiplier
    }
    
    #[test]
    fn test_update() {
        let parameters = ElevatorSystemParameters {
            floors: vec![0.0, 3.0, 6.0],
            elevators: vec!["param/elevator_test_parameters.yaml".into()],
            time_multiplier: 1.0,
            gravity: 9.81,
        };
        let mut system = ElevatorSystem::new(parameters);

        // Mock initial conditions and population
        system.update_population();

        // Run update
        system.update();

        for elevator in &system.elevators {
            assert!(elevator.get_used_energy() > 0.0, "Elevator should consume energy during update");
        }
    }
}
