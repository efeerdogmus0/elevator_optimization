// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

#[cfg(test)]
// Import the outer module's functions
mod elevator_tests {
    use super::super::*;

    #[test]
    fn create() {
        let mut _elevator = Elevator::from_file(
            vec![0., 3., 6., 9., 12.],
            9.81, 
            "param/elevator_test_parameters.yaml"
        ).unwrap();
    }

    #[test]
    fn for_tuning() {
        let mut elevator = Elevator::from_file(
            vec![0., 3., 6., 9., 12.],
            9.81, 
            "param/elevator_test_parameters.yaml"
        ).unwrap();

        let target_idx = 3;
        elevator.set_target(target_idx);

        let mut elapsed = 0.;
        let delta_time = 0.1;
        loop {
            elevator.update(delta_time);

            elapsed += delta_time;
            // time limit
            if elapsed >= 3. {
                break;
            }
        }

        let result = elevator.get_current_floor().unwrap() == target_idx;
        assert!(result);
    }

    #[test]
    fn goto_floor() {
        let mut elevator = Elevator::from_file(
            vec![0., 3., 6., 9., 12.],
            9.81, 
            "param/elevator_test_parameters.yaml"
        ).unwrap();
        let target_idx = 3;

        elevator.set_target(target_idx);

        let mut elapsed = 0.;
        let delta_time = 0.1;
        loop {
            elevator.update(delta_time);
            if elevator.can_board() {
                break;
            }

            // time limit
            elapsed += delta_time;
            if elapsed >= 6. {
                panic!("Timeout");
            }
        }
        let result = elevator.get_current_floor().unwrap() == target_idx;
        assert!(result);
    }
}


#[cfg(test)]
mod elevator_parameter_tests {
    use super::super::*;

    #[test]
    fn read() {
        let parameters = ElevatorParameters::from_file("param/test/elevator_test_parameters.yaml").unwrap();
        println!("{:?}", parameters);
    }
}

