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
        // let now = Instant::now();
        // let mut delta_time = now.duration_since(self.last_update).as_secs_f32();
        // self.last_update = now;
        // delta_time *= self.time_multiplier;
        
        0.1
    }

    fn _get_delta_time(&mut self) -> f32 {
        // calculate delta time
        let now = Instant::now();
        let mut delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        delta_time *= self.time_multiplier;
        
        delta_time
    }

    fn unload_elevator(&mut self, elevator_idx: usize) {
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

    fn load_elevator(&mut self, elevator_idx: usize) {
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

    fn update_population(&mut self, delta_time: f32) {
        for floor in 0..self.floor_heights.len() {
            let entities = self.pop_gen.generate(0, delta_time, floor);
            self.queue[floor].extend(entities);
        }
    }

    pub fn get_elevator(&self, idx: usize) -> &Elevator {
        &self.elevators[idx]
    }

    pub fn set_elevator_target(&mut self, elevator_idx: usize, floor_idx: usize) {
        self.elevators[elevator_idx].set_target(floor_idx);
    }

    pub fn update(&mut self) {
        let delta_time = self.get_delta_time();

        // give birth to new homo sapiens
        self.update_population(delta_time);

        for idx in 0..self.elevators.len() {
            self.unload_elevator(idx);
            self.load_elevator(idx);

            self.elevators[idx].update(delta_time);
        }
    }
}