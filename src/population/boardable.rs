// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

// Define the trait for entities that can board an elevator

use crate::machine::Elevator;

pub trait Boardable: Send + Sync {
    fn clone_box(&self) -> Box<dyn Boardable>;

    fn get_area(&self) -> f32; // Area occupied by the entity
    fn get_weight(&self) -> f32; // Weight of the entity
    fn calculate_boarding_time(&self, elevator: &Elevator) -> f32; // Time to board the elevator
    fn get_wait_time(&self) -> f32; // Time spent waiting for the elevator
    fn increase_wait_time(&mut self, increment: f32);
    fn get_destination(&self) -> usize;
}

impl Clone for Box<dyn Boardable> {
    fn clone(&self) -> Box<dyn Boardable> {
        self.clone_box()
    }
}
