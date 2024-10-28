// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use pyo3::ffi::printfunc;

use crate::machine::{ Elevator, Direction, Queue };
use crate::algorithms::ElevatorControllerAlgorithm;
use crate::population::Boardable;

use super::neural::NeuralNetwork;
use super::state_space::{ SystemState, ElevatorState };

#[derive(Clone)]
pub struct NeuralControlAlgorithm {
    is_initialized: bool,
    network: NeuralNetwork,
    floor_count: usize,
    elevator_count: usize,
}

impl NeuralControlAlgorithm  {
    pub fn new() -> Self {
        let network = NeuralNetwork::new(0, 0, 0);
        Self::new_with_nn(network)
    }

    pub fn new_with_nn(network: NeuralNetwork) -> Self {
        NeuralControlAlgorithm {
            is_initialized: true,
            network,
            floor_count: 0,
            elevator_count: 0,
        }
    }

    pub fn get_network(&self) -> &NeuralNetwork {
        &self.network
    }

    fn send_targets(&self, network_output: Vec<f32>, elevators: &mut Vec<Elevator>) {
        if network_output.len() != elevators.len() {
            panic!("NeuralControlAlgorithm: Invalid network output size.");
        }

        for idx in 0..elevators.len() {
            let target = self.neural_output_to_floor(network_output[idx], elevators[idx].get_floor_count());
            elevators[idx].set_target(target);
        }
    }

    fn neural_output_to_floor(&self, neural_output: f32, floor_count: usize) -> usize {
        (neural_output * floor_count as f32) as usize
    }

    fn load_elevator(&mut self, elevator: &mut Elevator, queue: &mut Queue) {
        if !elevator.can_board(){
            return;
        }

        match elevator.get_current_floor() {
            Some(floor) => {
                if queue.is_floor_empty(floor) {
                    return;
                }

                println!("Elevator is at floor {} and loading, queue len: {}", 
                    floor,
                    queue.get_floor_queue(floor).len(),
                );

                let mut idx = 0;
                while idx < queue.get_floor_queue(floor).len() {
                    let entity = queue.get(floor, idx);

                    let can_fit = elevator.can_fit(entity);
                    if can_fit {
                        let entity = queue.remove(floor, idx).unwrap();
                        elevator.load(entity);
                    } else { idx += 1; }
                }
            },
            _ => {},
        };
    }
}


impl ElevatorControllerAlgorithm for NeuralControlAlgorithm {
    fn init(
        &mut self,
        elevator_count: usize,
        floor_count: usize,
    ) {
        // klipperımı özledim
        let input_shape = SystemState::var_count(elevator_count, floor_count);
        let hidden_shape = (input_shape as f32 * 1.5) as usize;
        
        self.network = NeuralNetwork::new(input_shape, hidden_shape, elevator_count);
        self.elevator_count = elevator_count;
        self.floor_count = floor_count;
    }

    fn update(
        &mut self,
        _delta_time: f32,
        elevators: &mut Vec<Elevator>,
        queue: &mut Queue,
        calls: Vec<Direction>,
    ) {
        if !self.is_initialized {
            panic!("NeuralControlAlgorithm: Not initialized.");
        } 
        
        let system_state = SystemState::new(&elevators, calls);

        let network_output = self.network.forward(&system_state.to_vec());

        self.send_targets(network_output, elevators);

        for elevator in elevators.iter_mut() {
            elevator.unload();
            self.load_elevator(elevator, queue);
        }
    }
}
