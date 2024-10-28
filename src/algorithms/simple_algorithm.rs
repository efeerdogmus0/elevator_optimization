// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use crate::machine::{ Elevator, Direction, Queue };
use super::ElevatorControllerAlgorithm;
use crate::population::Boardable;

pub struct SimpleElevatorController {
    up_elevator_idx: usize,
    up_current_target: usize,
    initialized: bool,

    // max floor normalde süper gerekli değil ama 
    // her seferinde elevators.len() - 1 yazmaya üşendim
    max_floor: usize,
    // daha production ready bir kod için belki kaldırılabilir
}

impl SimpleElevatorController {
    pub fn new() -> Self {
        Self {
            // yukarı giden asansör
            up_elevator_idx: 0,
            // yukarı giden asansörün anlık hedef katı
            up_current_target: 0,
            initialized: false,
            max_floor: 0,
        }
    }

    fn init_print(&self, elevators: &Vec<Elevator>) {
        let el = &elevators[self.get_down_idx()];
        println!("%{} Simple Elevator Algorithm initializing...", 
            (el.get_current_height()/el.get_max_height()*100.0) as u32);
    }

    fn prepare_elevators(&mut self, elevators: &mut Vec<Elevator>) {
        self.init_print(elevators);

        self.max_floor = elevators[0].get_floor_count() - 1;
        self.up_current_target = 0;
        self.send_target(elevators);

        if let Some(floor) = elevators[self.get_down_idx()].get_current_floor() {
            if floor == self.max_floor {
                self.initialized = true;
                println!("Initial setup complete.");
                println!("Up Elevator index: {}", self.get_up_idx());
                println!("Down Elevator index: {}", self.get_down_idx());

                self.up_current_target = 1;
                self.send_target(elevators);
            }
        }
    }

    fn send_target(&mut self, elevators: &mut Vec<Elevator>) {
        elevators[self.get_up_idx()].set_target(self.up_current_target);
        elevators[self.get_down_idx()].set_target(self.max_floor - self.up_current_target);
    }

    fn switch_elevators(&mut self) {
        self.up_elevator_idx = 1 - self.up_elevator_idx;
        self.up_current_target = 0;
    }

    pub fn get_up_idx(&self) -> usize {
        self.up_elevator_idx
    }

    pub fn get_down_idx(&self) -> usize {
        1 - self.up_elevator_idx
    }

    fn are_both_targets_reached(&self, elevators: &Vec<Elevator>) -> bool {
        if elevators[self.get_up_idx()].get_current_floor() == Some(self.up_current_target) {
            if elevators[self.get_down_idx()].get_current_floor() == Some(self.max_floor - self.up_current_target) {
                return true;
            }
        }
        false
    }

    // fn unload_elevators(&self, elevators: &mut Vec<Elevator>) {
    //     if elevators[0].get_entity_count() > 0 {
    //         elevators[0].unload();
    //     }
    //     if elevators[1].get_entity_count() > 0 {
    //         elevators[1].unload();
    //     }
    // }

    // fn load_elevators(&mut self, elevators: &mut Vec<Elevator>, queue: &mut Queue) {
    //     self.load_elevator(&mut elevators, self.get_up_idx(), queue);
    //     self.load_elevator(&mut elevators, self.get_down_idx(), queue);
    // }

    fn load_elevator(&mut self, is_up: bool, elevator: &mut Elevator, queue: &mut Queue) {
        if !elevator.can_board() {
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

                    if floor != 0 && floor != self.max_floor {
                        // yukarı gidiyosa ve aşağı giden asansöre binmeye çalışıyosa boşver
                        if entity.get_destination() > floor && !is_up {
                            idx += 1;
                            continue;
                        }

                        // aşağı gidiyosa ve yukarı giden asansöre binmeye çalışıyosa boşver
                        if entity.get_destination() < floor && is_up {
                            idx += 1;
                            continue;
                        }
                    }

                    // doğru yöndeyse bindir
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


impl ElevatorControllerAlgorithm for SimpleElevatorController {
    fn init(
        &mut self,
        _elevator_count: usize, 
        _num_elevators: usize
    ) { }

    fn update(
        &mut self,
        _delta_time: f32,
        elevators: &mut Vec<Elevator>,
        queue: &mut Queue,
        _calls: Vec<Direction>,
    ) {
        // Ensure we have exactly two elevators; otherwise, panic
        if elevators.len() != 2 {
            panic!("SimpleElevatorController requires exactly two elevators.");
        }

        // if not initialized, initialize elevators
        if !self.initialized {
            self.prepare_elevators(elevators);
            return;
        }
        
        if self.are_both_targets_reached(elevators) {
            println!("Up Elevator at floor: {}", self.up_current_target);
            println!("Down Elevator at floor: {}", self.max_floor - self.up_current_target);

            elevators[self.get_up_idx()].unload();
            elevators[self.get_down_idx()].unload();
            self.load_elevator(true, &mut elevators[self.get_up_idx()], queue);
            self.load_elevator(false, &mut elevators[self.get_down_idx()], queue);

            if self.up_current_target == self.max_floor {
                self.switch_elevators();
                println!("Switching elevators");
                println!("elevator[0] entitiy count: {}", elevators[0].get_entity_count());
                println!("elevator[1] entitiy count: {}", elevators[1].get_entity_count());
            }
            self.up_current_target += 1;
            self.send_target(elevators);
        }
    }
}
