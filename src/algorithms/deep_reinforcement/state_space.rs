use crate::machine::{ Direction, Elevator, Queue };


pub struct SystemState {
    elevator_states: Vec<ElevatorState>,
    upwards_calls: Vec<bool>,
    downwards_calls: Vec<bool>,
}


impl SystemState {
    pub fn new(
        elevators: &Vec<Elevator>,
        calls: Vec<Direction>,
    ) -> Self {
        let (upwards_calls, downwards_calls) = SystemState::parse_calls(calls);
        let mut elevator_states = Vec::new();
        for elevator in elevators {
            elevator_states.push(ElevatorState::new(elevator));
        }

        SystemState {
            elevator_states: Vec::new(),
            upwards_calls,
            downwards_calls,
        }

    }

    pub fn to_vec(&self) -> Vec<f32> {
        let mut state_vec = Vec::new();

        for elevator_state in self.elevator_states.iter() {
            state_vec.extend(elevator_state.to_vec());
        }

        for idx in 0..self.upwards_calls.len() {
            state_vec.push(if self.upwards_calls[idx] { 1.0 } else { 0.0 });
            state_vec.push(if self.downwards_calls[idx] { 1.0 } else { 0.0 });
        }

        state_vec
    }

    pub fn var_count(elevator_count: usize, floor_count: usize) -> usize {
        // elevator states = elevator_vars * elevator_count
        // upwards_calls = floor_count
        // downwards_calls = floor_count

        ElevatorState::var_count(floor_count) * elevator_count + 2 * floor_count
    }

    pub fn parse_calls(calls: Vec<Direction>) -> (Vec<bool>, Vec<bool>) {
        let mut upwards_calls = Vec::new();
        let mut downwards_calls = Vec::new();

        for call in calls {
            match call {
                Direction::Up => {
                    upwards_calls.push(true);
                    downwards_calls.push(false);
                }
                Direction::Down => {
                    upwards_calls.push(false);
                    downwards_calls.push(true);
                }
                Direction::Both => {
                    upwards_calls.push(true);
                    downwards_calls.push(true);
                }
                Direction::None => {
                    upwards_calls.push(false);
                    downwards_calls.push(false);
                }
            }
        }
        (upwards_calls, downwards_calls)
    }
}


pub struct ElevatorState {
    current_floor: i32,
    target_floor: usize,
    speed: f32,

    current_load: f32,
    area: f32,

    inner_target_floors: Vec<bool>,
}


impl ElevatorState {
    pub fn new(
        elevator: &Elevator,
    ) -> Self {
        let current_floor = match elevator.get_current_floor() {
            Some(floor) => floor as i32,
            None => -1 as i32,
        };
            

        Self {
            current_floor,
            speed: elevator.get_current_speed(),
            target_floor: elevator.get_target_floor(),
            current_load: elevator.get_current_load(),
            area: elevator.get_available_area(),
            inner_target_floors: Self::calc_inner_target_floors(elevator),
        }
    }

    pub fn to_vec(&self) -> Vec<f32> {
        let mut state_vec = Vec::new();
        state_vec.push(self.current_floor as f32);
        state_vec.push(self.speed as f32);
        state_vec.push(self.target_floor as f32);
        state_vec.push(self.current_load as f32);
        state_vec.push(self.area as f32);
        for target in self.inner_target_floors.iter() {
            if target.clone() {
                state_vec.push(1.0);
            } else {
                state_vec.push(0.0);
            }
        }

        state_vec
    }

    pub fn var_count(floor_count: usize) -> usize {
        // current_floor: i32,
        // target_floor: usize,
        // speed: f32,
        // current_load: f32,
        // area: f32,
        // inner_target_floors: Vec<bool>,

        2 + 1 + 1 + 1 + floor_count
    }

    pub fn calc_inner_target_floors(elevator: &Elevator) -> Vec<bool> {
        let mut inner_target_floors = Vec::new();
        for _ in 0..elevator.get_floor_count() {
            inner_target_floors.push(false);
        }

        for target_floor in elevator.get_insider_targets() {
            inner_target_floors[target_floor] = true;
        }

        inner_target_floors
    }

}

