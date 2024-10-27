use crate::machine::{ Elevator, Direction };
use super::ElevatorControllerAlgorithm;

pub struct SimpleElevatorController {
    up_target: usize,    // Target floor for the upward-moving elevator
    down_target: usize,  // Target floor for the downward-moving elevator
    initialized: bool,   // Tracks whether the initial setup has been completed
}

impl SimpleElevatorController {
    pub fn new() -> Self {
        Self {
            up_target: 0,             // Initial target for the "up" elevator (bottom floor)
            down_target: usize::MAX,  // Placeholder to indicate no initial target for the "down" elevator
            initialized: false,
        }
    }

    fn reverse_direction(&mut self, current_floor: usize, max_floor: usize, going_up: bool) -> usize {
        // Reverse the target based on whether the elevator was going up or down
        if going_up && current_floor == max_floor {
            0 // Go to the bottom floor
        } else if !going_up && current_floor == 0 {
            max_floor // Go to the top floor
        } else {
            current_floor // Keep the same target if no reversal is needed
        }
    }
}

impl ElevatorControllerAlgorithm for SimpleElevatorController {
    fn update(
        &mut self,
        delta_time: f32,
        elevators: &mut Vec<Elevator>,
        calls: Vec<Option<Direction>>,
    ) {
        // Ensure we have exactly two elevators; otherwise, panic
        if elevators.len() != 2 {
            panic!("SimpleElevatorController requires exactly two elevators.");
        }

        if !self.initialized {
            // On the first update, set one elevator to the top floor and the other to the bottom
            let max_floor = elevators.len() - 1;
            elevators[0].set_target(0);         // Set the first elevator to start at the bottom
            elevators[1].set_target(max_floor); // Set the second elevator to start at the top
            self.initialized = true;
            self.up_target = max_floor;   // Set initial target for the up elevator to the top
            self.down_target = 0;         // Set initial target for the down elevator to the bottom
            println!("Initial setup complete: Elevator 1 set to top floor, Elevator 0 to bottom floor.");
            return;
        }

        let max_floor = elevators.len() - 1;

        // Split the elevators vector into two separate mutable references
        let (up_elevator_slice, down_elevator_slice) = elevators.split_at_mut(1);
        let up_elevator = &mut up_elevator_slice[0];
        let down_elevator = &mut down_elevator_slice[0];

        // For up elevator: Reverse direction if it reached its target floor
        if let Some(current_floor) = up_elevator.get_current_floor() {
            if current_floor == self.up_target {
                self.up_target = self.reverse_direction(current_floor, max_floor, true);
            }
            up_elevator.set_target(self.up_target);
        }

        // For down elevator: Reverse direction if it reached its target floor
        if let Some(current_floor) = down_elevator.get_current_floor() {
            if current_floor == self.down_target {
                self.down_target = self.reverse_direction(current_floor, max_floor, false);
            }
            down_elevator.set_target(self.down_target);
        }

        // Process calls to direct elevators to requested floors
        for (floor, call) in calls.iter().enumerate() {
            if let Some(direction) = call {
                match direction {
                    Direction::Up => up_elevator.set_target(floor),
                    Direction::Down => down_elevator.set_target(floor),
                }
                println!(
                    "Call received on floor {} to move {:?}. Elevator targets updated accordingly.",
                    floor, direction
                );
            }
        }

        println!(
            "Elevator 0 (Up) target set to floor {}. Elevator 1 (Down) target set to floor {}.",
            self.up_target,
            self.down_target
        );
    }
}
