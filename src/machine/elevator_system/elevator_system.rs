// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use crate::machine::elevator::{ Elevator, ElevatorParameters };
use super::elevator_system_parameters::ElevatorSystemParameters;
use crate::population::{ Transportable, PopulationGenerator };
use crate::algorithms::ElevatorControllerAlgorithm;

use pyo3::prelude::*;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Both,
    None,
}

pub struct Queue {
    entities: Vec<Vec<Box<dyn Transportable>>>,
    wait_times: Vec<f32>,
}

impl Queue {
    pub fn new(floor_count: usize) -> Self {
        let mut entities = Vec::with_capacity(floor_count);
        for _ in 0..floor_count {
            entities.push(Vec::new());
        }
        Self {
            entities,
            wait_times: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        for floor in 0..self.len() {
            if !self.is_floor_empty(floor) {
                return false;
            }
        }
        true
    }

    pub fn is_floor_empty(&self, floor: usize) -> bool {
        self.entities[floor].is_empty()
    }

    pub fn get(&mut self, floor: usize, entity_idx: usize) -> &Box<dyn Transportable> {
        &self.entities[floor][entity_idx]
    }

    pub fn get_mut(&mut self, floor: usize, entity_idx: usize) -> &mut Box<dyn Transportable> {
        &mut self.entities[floor][entity_idx]
    }

    pub fn remove(&mut self, floor: usize, entity_idx: usize) -> Option<Box<dyn Transportable>> {
        if self.entities[floor].is_empty() {
            return None;
        }
        let entity = self.entities[floor].remove(entity_idx);
        self.wait_times.push(entity.get_queue_wait());
        Some(entity)
    }

    pub fn add(&mut self, floor: usize, entity: Box<dyn Transportable>) {
        self.entities[floor].push(entity);
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn get_floor_queue(&self, floor: usize) -> &Vec<Box<dyn Transportable>> {
        &self.entities[floor]
    }

    pub fn get_floor_queue_mut(&mut self, floor: usize) -> &mut Vec<Box<dyn Transportable>> {
        &mut self.entities[floor]
    }

    pub fn get_floor_count(&self) -> usize {
        self.entities.len()
    }

    pub fn update_wait_time(&mut self, delta_time: f32) {
        for floor_queue in &mut self.entities {
            for entity in floor_queue {
                entity.increase_wait_time(delta_time);
            }
        }
    }
}


#[pyclass]
pub struct ElevatorSystem {
    floor_heights: Vec<f32>,
    controller: Box<dyn ElevatorControllerAlgorithm>,
    queue: Queue,
    pop_gen: PopulationGenerator,
    elevators: Vec<Elevator>,
    time_of_day: u32,
    time_step: f32,
    hour_length: f32,
    total_time_passed: f32,
    max_human_count: u32,
}


#[pymethods]
impl ElevatorSystem {
    pub fn get_used_energy(&self) -> f32 {
        let mut total: f32 = 0.0;
        for elevator in &self.elevators {
            total += elevator.get_used_energy();
        }
        total
    }

    pub fn final_kwh(&self) -> f32 {
        // normalde burda hour_len kullanıcaktım ama 
        // onu sadece popülasyona özel kılmak istedim
        let hours = self.get_total_time() / 3600.;
        self.get_used_energy() * hours
    }

    pub fn get_total_wait_time(&self) -> f32 {
        let mut total: f32 = 0.0;
        for wait_time in &self.queue.wait_times {
            total += wait_time;
        }

        if total > self.queue.wait_times.len() as f32 * self.get_total_time() {
            for _ in 0..10 {
                println!("There has been a mistake in the calculation of total wait time");
                println!("Please contact the supreme leader tunapro1234 for further instructions");
            }
        }

        total
    }

    pub fn get_total_time(&self) -> f32 {
        self.total_time_passed
    }

    fn update_time_of_day(&mut self) {
        self.time_of_day += ((self.total_time_passed / self.hour_length) % 24.) as u32;
    }

    pub fn is_finished(&self) -> bool {
        if self.pop_gen.get_human_generated() < self.max_human_count {
            return false;
        }

        for elevator in &self.elevators {
            if elevator.get_entity_count() > 0 {
                return false;
            }
        }

        if !self.queue.is_empty() {
            return false;
        }

        true
    }

    pub fn update(&mut self) -> bool {
        let delta_time: f32 = self.get_delta_time();
        self.total_time_passed += delta_time;
        self.update_time_of_day();

        if self.is_finished() {
            return false;
        }

        // give birth to new homo sapiens
        if self.pop_gen.get_human_generated() < self.max_human_count {
            self.generate_population(delta_time);
        } 
        self.queue.update_wait_time(delta_time);

        let called_buttons = self.get_call_buttons();
        self.controller.update(
            delta_time,
            &mut self.elevators,
            &mut self.queue,
            called_buttons,
        );

        for idx in 0..self.elevators.len() {
            // self.elevators[idx].unload();
            // self.load_elevator(idx);

            self.elevators[idx].update(delta_time);

            // println!("### Elevator {}", idx);
            // self.elevators[idx].debug_print();
        }
        true
    }

    pub fn get_elevators(&self) -> Vec<Elevator> {
        self.elevators.clone()
    }
}

impl ElevatorSystem {
    pub fn from_file(
        controller: Box<dyn ElevatorControllerAlgorithm>,
        file_path: &str,        
    ) -> Result<Self, Box<dyn Error>> {
        let parameters = ElevatorSystemParameters::from_file(file_path)?;
        Ok(Self::new(controller, parameters))
    }

    pub fn new(
        mut controller: Box<dyn ElevatorControllerAlgorithm>,
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

        controller.init(elevators.len(), floor_count);

        Self {
            floor_heights: parameters.floors,
            controller,
            queue: Queue::new(floor_count),
            pop_gen: PopulationGenerator::new(floor_count),
            elevators,
            time_of_day: parameters.time_of_day,
            time_step: parameters.time_step,
            hour_length: parameters.hour_length,
            total_time_passed: 0.0,
            max_human_count: parameters.max_human_count,
        }
    }

    fn get_delta_time(&mut self) -> f32 {
        // calculate delta time
        // let now = Instant::now();
        // let mut delta_time = now.duration_since(self.last_update).as_secs_f32();
        // self.last_update = now;
        // delta_time *= self.time_multiplier;
        
        self.time_step
    }

    fn generate_population(&mut self, delta_time: f32) {
        let generated = self.pop_gen.generate(
            self.time_of_day, 
            delta_time, 
            self.hour_length,
        );

        for (floor, entity) in generated {
            self.queue.add(floor, entity);
        }
    }

    pub fn get_call_buttons(&self) -> Vec<Direction> {
        let mut call_buttons = vec![Direction::None; self.queue.len()];

        for floor_idx in 0..self.queue.len() {
            let floor_queue = self.queue.get_floor_queue(floor_idx);

            if floor_queue.is_empty() {
                call_buttons[floor_idx] = Direction::None;
                continue;
            } 
            
            for entity in floor_queue {
                // tuş yukarıyaysa
                if entity.get_destination() > floor_idx {
                    if call_buttons[floor_idx] == Direction::Down {
                        call_buttons[floor_idx] = Direction::Both;
                    } else {
                        call_buttons[floor_idx] = Direction::Up;
                    }
                    break;
                } 

                // tuş aşağıyaysa
                if entity.get_destination() < floor_idx {
                    if call_buttons[floor_idx] == Direction::Up {
                        call_buttons[floor_idx] = Direction::Both;
                    } else {
                        call_buttons[floor_idx] = Direction::Down;
                    }
                    break;
                }
            }
        }

        call_buttons
    }
}