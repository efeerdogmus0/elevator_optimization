// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use crate::machine::pid_controller::PIDController;
use crate::machine::motor::ElevatorMotor;
use crate::population::Boardable;
use super::elevator_parameters::ElevatorParameters;
use crate::util::LinePlotter;

use std::error::Error;

pub struct Elevator {
    floors: Vec<f32>, // floor heights, taken from elevator controller
    current_height: f32,
    height_pid: PIDController,
    // weigth and forces 
    elevator_mass: f32,
    elevator_counter_mass: f32,
    max_load: f32,
    current_load: f32,
    motor: ElevatorMotor,
    // simulation-related
    gravity: f32,
    entities: Vec<Box<dyn Boardable>>,
    target_idx: usize,
    area: f32,
    current_area: f32,
    line_plotter: Option<LinePlotter>,
}


impl Elevator {
    pub fn from_file(
        floors: Vec<f32>,
        gravity: f32,
        file_path: &str,        
    ) -> Result<Self, Box<dyn Error>> {
        let parameters = ElevatorParameters::from_file(file_path)?;
        Ok(
            Self::new(
                floors,
                gravity,
                parameters,
            )
        )
    }

    pub fn new(
        floors: Vec<f32>,
        gravity: f32,
        parameters: ElevatorParameters,
    ) -> Self {
        let height_pid = PIDController::new(parameters.pid_parameters);
        let mut motor = ElevatorMotor::new(parameters.motor_parameters).unwrap();

        // line plotterı yarat
        let line_plotter = if parameters.enable_debug_plotting {
            let plot_rv = LinePlotter::new(parameters.plot_path);
            match plot_rv {
                Ok(line_plotter) => {
                    println!("Plotter created");
                    Some(line_plotter)
                },
                Err(e) => {
                    println!("Error creating plotter: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let max_accel = Self::calc_max_accel(
            gravity, 
            parameters.elevator_mass, 
            parameters.elevator_counter_mass, 
            parameters.max_load,
            motor.max_force,
        );

        motor.set_acceleration(max_accel);

        Self {
            floors,
            current_height: 0.0,
            height_pid,
            elevator_mass: parameters.elevator_mass,
            elevator_counter_mass: parameters.elevator_counter_mass,
            max_load: parameters.max_load,
            current_load: 0.0,
            entities: Vec::new(),
            target_idx: 0,
            area: parameters.area,
            current_area: parameters.area,
            motor,
            gravity,
            line_plotter,
        }
    }

    pub fn get_used_energy(&self) -> f32{
        self.motor.get_total_energy_used()
    }

    pub fn get_total_mass(&self) -> f32 {
        self.elevator_mass + self.current_load + self.elevator_counter_mass
    }

    fn calculate_target_speed(&mut self, delta_time: f32) -> f32 {
        // calculate target speed
        let target_speed = self.height_pid.update(self.current_height, delta_time);

        // limits are applied in the motor
        // if target_speed > self.max_speed {
        //     target_speed = self.max_speed;
        // } else if target_speed < -self.max_speed {
        //     target_speed = -self.max_speed;
        // }

        target_speed
    }

    fn calculate_motor_force(&self, target_accel: f32) -> f32 {
        let m = self.get_total_mass();
        let e = self.elevator_mass + self.current_load;

        // required force by the elevator motor
        let req_force = (e-self.elevator_counter_mass)*self.gravity - m*target_accel;
        req_force
    }

    fn plot(&mut self, delta_time: f32) {
        if let Some(line_plotter) = &mut self.line_plotter {
            line_plotter.add_point(self.current_height, delta_time);
            line_plotter.update().unwrap();
        }
    }

    pub fn can_fit(&self, entity: &Box<dyn Boardable>) -> bool {
        if self.current_load + entity.get_weight() > self.max_load {
            println!("Weight limit exceeded");
            return false;
        }
        if self.current_area < entity.get_area() {
            println!("Area limit exceeded");
            return false;
        }

        return true;
    }

    pub fn load(&mut self, entity: Box<dyn Boardable>) {
        // this is also checked in the elevator 
        // system so nothing should go wrong
        if !self.can_fit(&entity) {
            panic!("Entity cannot fit in the elevator");
        }

        self.current_load += entity.get_weight();
        self.area -= entity.get_area();
        self.entities.push(entity);
    }

    pub fn unload(&mut self) {
        let mut idx = 0;
        while idx < self.entities.len() {
            let entity = &self.entities[idx];
            if entity.get_destination() == self.target_idx {
                self.current_load -= entity.get_weight();
                self.area += entity.get_area();
                self.entities.remove(idx);
            }
            else { idx += 1;}
        }
    }

    pub fn can_board(&self) -> bool {
        if !self.height_pid.has_reached_target(self.current_height) {
            return false;
        }
        if (self.motor.get_current_speed() - 0.).abs() > 0.1 {
            return false;
        }
        true
    }

    pub fn get_current_height(&self) -> f32 {
        self.current_height
    }

    pub fn get_current_floor(&self) -> Option<usize> {
        match self.can_board() {
            true => Some(self.target_idx),
            false => None,
        }
    }

    pub fn set_target(&mut self, floor_idx: usize) {
        self.height_pid.set_target(self.floors[floor_idx]);
        self.target_idx = floor_idx;
    }

    pub fn get_target_floor(&self) -> usize {
        self.target_idx
    }

    pub fn is_moving_towards(&self, idx: usize) -> bool {
        let call_h = self.floors[idx];
        let target_h = self.floors[self.target_idx];
        let current_h = self.get_current_height();

        // yukarı gidiyorsak 
        if current_h < call_h && current_h < target_h {
            return true;
        }
        // aşağı gidiyosak
        if current_h > call_h && current_h > target_h {
            return true;
        }

        false
    }

    pub fn get_current_speed(&self) -> f32 {
        self.motor.get_current_speed()
    }

    pub fn get_target_floors(&self) -> Vec<usize> {
        self.entities.iter()
            .map(|entity| entity.get_destination())
            .collect()
    }

    pub fn debug_string(&self) -> String {
        format!("height {}, target {}, reached? {} entity count: {}",
            self.get_current_height(),
            self.floors[self.target_idx],
            self.can_board(),
            self.entities.len(),
        )
    }

    pub fn debug_print(&self) {
        println!("{}", self.debug_string());
    }

    fn calc_max_accel(g: f32, e: f32, c: f32, l: f32, motor_force: f32) -> f32 {
        // max acceleration by the elevator motor
        (e-c)*g-motor_force/(e+c+l)
    }

    pub fn update(&mut self, delta_time: f32) {
        self.current_height += self.motor.get_current_speed() * delta_time;
        self.motor.update_energy_used(delta_time);

        let target_speed: f32 = self.calculate_target_speed(delta_time);
        self.plot(delta_time);
        self.motor.set_target_speed(target_speed);
        self.motor.update(delta_time);
    }
}