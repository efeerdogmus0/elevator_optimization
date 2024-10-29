// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use std::error::Error;
use serde::Deserialize;

use crate::machine::pid_controller::PIDParameters;
use crate::machine::motor::MotorParameters;

#[derive(Debug, Deserialize)]
pub struct ElevatorParameters {
    pub pid_parameters: PIDParameters,
    pub motor_parameters: MotorParameters,
    pub max_speed: f32,
    pub max_accel: f32,
    pub max_load: f32,
    pub elevator_mass: f32,
    pub elevator_counter_mass: f32,
    pub area: f32,

    #[serde(default = "default_enable_debug_plotting")]
    pub enable_debug_plotting: bool,
    #[serde(default = "default_plot_path")]
    pub plot_path: String,
}

fn default_enable_debug_plotting() -> bool { false }
fn default_plot_path() -> String { "data/debug/elevator_plot.png".to_string() }

impl ElevatorParameters {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = std::fs::File::open(file_path)?;
        let result = serde_yaml::from_reader(file)?;
        Ok(result)
    }
}
