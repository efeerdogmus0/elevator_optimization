use pyo3::ffi::printfunc;

use crate::machine::{ Elevator, Direction };
use super::ElevatorControllerAlgorithm;

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

    fn init(&mut self, elevators: &mut Vec<Elevator>) {
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
}


impl ElevatorControllerAlgorithm for SimpleElevatorController {
    fn update(
        &mut self,
        _delta_time: f32,
        elevators: &mut Vec<Elevator>,
        _calls: Vec<Direction>,
    ) {
        // Ensure we have exactly two elevators; otherwise, panic
        if elevators.len() != 2 {
            panic!("SimpleElevatorController requires exactly two elevators.");
        }

        // if not initialized, initialize elevators
        if !self.initialized {
            self.init(elevators);
            return;
        }
        
        if self.are_both_targets_reached(elevators) {
            println!("Up Elevator at floor: {}", self.up_current_target);
            println!("Down Elevator at floor: {}", self.max_floor - self.up_current_target);

            if self.up_current_target == self.max_floor {
                self.switch_elevators();
                println!("Switching elevators");
            }
            self.up_current_target += 1;
            self.send_target(elevators);
        }
    }
}
