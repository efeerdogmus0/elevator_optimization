// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use crate::machine::elevator::{ Elevator, ElevatorParameters };
use super::elevator_system_parameters::ElevatorSystemParameters;
use crate::population::{ Boardable, PopulationGenerator };
use crate::algorithms::ElevatorControllerAlgorithm;

use std::error::Error;

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
}


pub struct ElevatorSystem {
    floor_heights: Vec<f32>,
    controller: Box<dyn ElevatorControllerAlgorithm>,
    queue: Vec<Vec<Box<dyn Boardable>>>, 
    pop_gen: PopulationGenerator,
    elevators: Vec<Elevator>,
    all_wait_times: Vec<f32>,
    time_of_day: u32,
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
        controller: Box<dyn ElevatorControllerAlgorithm>,
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

        let mut queue = Vec::with_capacity(floor_count);
        for _ in 0..floor_count {
            queue.push(Vec::new());
        }

        Self {
            floor_heights: parameters.floors,
            controller,
            queue,
            pop_gen: PopulationGenerator::new(floor_count),
            elevators,
            all_wait_times: Vec::new(),
            time_of_day: parameters.time_of_day,
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
                        self.all_wait_times.push(entity.get_wait_time());
                        elevator.load(entity);
                    } else { idx += 1; }
                }
            },
            _ => {},
        };
    }

    fn generate_population(&mut self, delta_time: f32) {
        for floor in 0..self.floor_heights.len() {
            let entities = self.pop_gen.generate(self.time_of_day, delta_time, floor);

            self.queue[floor].extend(entities);
            // for entity in entities {
            //     self.queue[floor].push(entity);
            // }
        }
    }

    pub fn get_call_buttons(&self) -> Vec<Option<Direction>> {
        let mut call_buttons = Vec::new();
        for (floor_idx, floor_queue) in self.queue.iter().enumerate() {
            if floor_queue.is_empty() {
                call_buttons.push(None);
            } 
            for entity in floor_queue {
                if entity.get_destination() > floor_idx {
                    call_buttons.push(Some(Direction::Up));
                    break;
                } 
                else if entity.get_destination() < floor_idx {
                    call_buttons.push(Some(Direction::Down));
                    break;
                }
                else {
                    panic!("
                        1. The Industrial Revolution and its consequences have been 
                        a disaster for the human race. They have greatly increased 
                        the life-expectancy of those of us who live in “advanced” 
                        countries, but they have destabilized society, have made life 
                        unfulfilling, have subjected human beings to indignities, have 
                        led to widespread psychological suffering (in the Third World 
                        to physical suffering as well) and have inflicted severe damage 
                        on the natural world. The continued development of technology 
                        will worsen the situation. It will certainly subject human beings 
                        to greater indignities and inflict greater damage on the natural 
                        world, it will probably lead to greater social disruption and 
                        psychological suffering, and it may lead to increased physical 
                        suffering even in “advanced” countries.

                        2. The industrial-technological system may survive or it may break
                        down. If it survives, it MAY eventually achieve a low level of
                        physical and psychological suffering, but only after passing through
                        a long and very painful period of adjustment and only at the cost of
                        permanently reducing human beings and many other living organisms to
                        engineered products and mere cogs in the social machine. Furthermore,
                        if the system survives, the consequences will be inevitable: There is
                        no way of reforming or modifying the system so as to prevent it from
                        depriving people of dignity and autonomy.

                        https://www.washingtonpost.com/wp-srv/national/longterm/unabomber/manifesto.text.htm
                    ");
                }
            }
        }
        call_buttons
    }

    fn update_queue_wait_time(&mut self, delta_time: f32) {
        for floor_queue in &mut self.queue {
            for entity in floor_queue {
                entity.increase_wait_time(delta_time);
            }
        }
    }
    
    pub fn update(&mut self) {
        let delta_time = self.get_delta_time();

        // give birth to new homo sapiens
        self.generate_population(delta_time);
        self.update_queue_wait_time(delta_time);

        let called_buttons = self.get_call_buttons();
        self.controller.update(
            delta_time,
            &mut self.elevators,
            called_buttons,
        );

        for idx in 0..self.elevators.len() {
            self.elevators[idx].unload();
            self.load_elevator(idx);

            self.elevators[idx].update(delta_time);
        }
    }
}