// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

// Define the trait for entities that can board an elevator

use crate::machine::Elevator;

pub trait Transportable: Send + Sync {
    fn clone_box(&self) -> Box<dyn Transportable>;

    fn get_area(&self) -> f32;
    fn get_weight(&self) -> f32;
    fn calculate_boarding_time(&self, elevator: &Elevator) -> f32;
    fn get_destination(&self) -> usize;
    fn increase_wait_time(&mut self, increment: f32);
    fn get_queue_wait(&self) -> f32;
    fn get_transport_wait(&self) -> f32;
    fn board(&mut self);
}

impl Clone for Box<dyn Transportable> {
    fn clone(&self) -> Box<dyn Transportable> {
        self.clone_box()
    }
}
