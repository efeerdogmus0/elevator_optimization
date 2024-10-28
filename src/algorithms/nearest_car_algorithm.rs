use crate::machine::{Elevator, Direction, Queue};
use super::ElevatorControllerAlgorithm;

pub struct NearestCarDispatchController {
    is_initialized: bool,
    directions: Vec<Option<Direction>>, // Tracks each elevator’s current direction
}

impl NearestCarDispatchController {
    fn new() -> Self {
        NearestCarDispatchController {
            is_initialized: false,
            directions: Vec::new(),
        }
    }

    // Calculate the "distance" between an elevator's current height and the requested floor
    fn calculate_distance(elevator: &Elevator, target_floor: usize) -> f32 {
        let target_height = elevator.get_floor_height(target_floor);
        (elevator.get_current_height() - target_height).abs()
    }

    // Updates the direction for a specific elevator based on its target and current floor
    fn update_direction(&mut self, elevator_idx: usize, elevator: &Elevator) {
        if let Some(current_floor) = elevator.get_current_floor() {

            let target_floor = elevator.get_target_floor();

            self.directions[elevator_idx] = if target_floor > current_floor {
                Some(Direction::Up)
            } else if target_floor < current_floor {
                Some(Direction::Down)
            } else {
                None // Elevator has reached its target and is idle
            };
        }
    }

    // Unload passengers at the elevator's current floor
    fn unload_elevator(&self, elevator: &mut Elevator) {
        if elevator.get_entity_count() > 0 {
            elevator.unload();
        }
    }

    // Load passengers from the queue to the elevator based on the elevator’s direction and capacity
    fn load_elevator(&mut self, elevator_idx: usize, elevator: &mut Elevator, queue: &mut Queue) {
        if !elevator.can_board() {
            return;
        }

        if let Some(floor) = elevator.get_current_floor() {
            if queue.is_floor_empty(floor) {
                return;
            }

            println!(
                "Elevator {} is at floor {} and loading, queue len: {}",
                elevator_idx,
                floor,
                queue.get_floor_queue(floor).len(),
            );

            let current_direction = self.directions[elevator_idx].clone();
            let mut idx = 0;
            while idx < queue.get_floor_queue(floor).len() {
                let max_floor = queue.get_floor_count()-1;
                let entity = queue.get(floor, idx);

                // Check direction compatibility for mid-floor destinations
                if floor != 0 && floor != max_floor {
                    if (entity.get_destination() > floor && current_direction == Some(Direction::Down)) ||
                       (entity.get_destination() < floor && current_direction == Some(Direction::Up)) {
                        idx += 1;
                        continue;
                    }
                }

                // If the entity is compatible, try to load it onto the elevator
                if elevator.can_fit(entity) {
                    let entity = queue.remove(floor, idx).unwrap();
                    elevator.load(entity);
                } else {
                    idx += 1;
                }
            }
        }
    }

    // Find best elevator for external call based on proximity and direction compatibility
    fn find_best_elevator_for_call(
        &self,
        elevators: &mut Vec<Elevator>,
        target_floor: usize,
        direction: &Direction,
    ) -> Option<usize> {
        let mut best_elevator_idx = None;
        let mut min_distance = f32::MAX;

        for (idx, elevator) in elevators.iter_mut().enumerate() {
            let current_direction = self.directions[idx].clone();

            let is_compatible_direction = match direction {
                Direction::Up => current_direction == Some(Direction::Up) || elevator.can_board(),
                Direction::Down => current_direction == Some(Direction::Down) || elevator.can_board(),
                Direction::Both => {
                    current_direction == Some(Direction::Up) || current_direction == Some(Direction::Down) || elevator.can_board()
                }
                Direction::None => continue, // Skip floors with Direction::None
            };

            if is_compatible_direction {
                let distance = Self::calculate_distance(elevator, target_floor);
                if distance < min_distance {
                    min_distance = distance;
                    best_elevator_idx = Some(idx);
                }
            }
        }

        best_elevator_idx
    }
}

impl ElevatorControllerAlgorithm for NearestCarDispatchController {
    fn init(
        &mut self,
        elevator_count: usize, 
        num_elevators: usize
    ) {
        self.directions = vec![None; num_elevators];
    }

    fn update(
        &mut self,
        _delta_time: f32,
        elevators: &mut Vec<Elevator>,
        queue: &mut Queue,
        calls: Vec<Direction>,
    ) {
        if !self.is_initialized {
            panic!("NearestCarDispatchController is not initialized.");
        }

        if elevators.len() != self.directions.len() {
            panic!("Mismatch between the number of elevators and directions tracked.");
        }

        // Handle external calls
        for (floor, direction) in calls.iter().enumerate() {
            if *direction == Direction::None {
                continue;
            }

            if let Some(best_elevator_idx) = self.find_best_elevator_for_call(elevators, floor, direction) {
                // Set target for the best elevator based on the call
                elevators[best_elevator_idx].set_target(floor);
                self.update_direction(best_elevator_idx, &elevators[best_elevator_idx]);

                println!(
                    "Dispatching elevator {} to floor {} for {:?} direction",
                    best_elevator_idx, floor, direction
                );
            } else {
                // println!("No available elevator for floor {} with direction {:?}", floor, direction);
            }
        }

        // Process loading and unloading for each elevator
        for (idx, elevator) in elevators.iter_mut().enumerate() {
            // Unload passengers if the elevator has reached its target
            self.unload_elevator(elevator);

            // Update direction after unloading if needed
            self.update_direction(idx, elevator);

            // Load new passengers based on direction and current floor queue
            self.load_elevator(idx, elevator, queue);
        }
    }
}
