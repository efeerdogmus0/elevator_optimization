// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use crate::machine::{ Elevator, Direction };

pub trait ElevatorControllerAlgorithm {
    /// Updates the state of the elevator system.
    /// 
    /// This function will be called each cycle to manage elevator requests, 
    /// determine elevator movements, and make other adjustments as needed.
    /// 
    /// * `delta_time` - The time passed since the last update, allowing 
    ///   for time-based calculations.
    fn update(
        &mut self,
        delta_time: f32,
        elevators: &mut Vec<Elevator>,
        calls: Vec<Option<Direction>>,
    );
}
