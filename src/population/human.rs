// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

use super::transportable::Transportable;
use crate::machine::Elevator;

#[derive(Clone)]
pub enum Gender {
    Male,
    Female,
}


#[derive(Clone)]
pub struct Human {
    weight: f32,
    age: u8,
    area: f32,
    gender: Gender,
    destination_floor: usize,
    queue_wait: f32,
    transport_wait: f32,
    is_boarded: bool,
}

impl Transportable for Human {
    fn clone_box(&self) -> Box<dyn Transportable> {
        Box::new(self.clone())
    }

    fn get_area(&self) -> f32 {
        self.area 
    }

    fn get_weight(&self) -> f32 {
        self.weight
    }

    fn calculate_boarding_time(&self, elevator: &Elevator) -> f32 {
        // Calculation logic based on elevator properties, e.g., door width
        // Placeholder value for boarding time
        2.0 
    }

    fn get_destination(&self) -> usize {
        self.destination_floor
    }

    fn increase_wait_time(&mut self, increment: f32) {
        self.queue_wait += increment;
    }

    fn get_queue_wait(&self) -> f32 {
        self.queue_wait
    }

    fn get_transport_wait(&self) -> f32 {
        self.queue_wait
    }

    fn board(&mut self) {
        self.is_boarded = true;
    }
}


impl Human {
    pub fn new(
        age: u8, 
        gender: Gender,
        weight: f32,
        area: f32,
        destination_floor: usize,
    ) -> Self {
        Self {
            weight,
            age,
            gender,
            area,
            destination_floor,
            queue_wait: 0.,
            transport_wait: 0.,
            is_boarded: false,
        }
    }
}


#[derive(Clone)]
pub struct HumanGroup {
    members: Vec<Human>,
}

impl HumanGroup {
    pub fn new(members: Vec<Human>) -> Self {
        Self {
            members,
        }
    }
}

impl Transportable for HumanGroup {
    fn clone_box(&self) -> Box<dyn Transportable> {
        Box::new(self.clone())
    }

    fn get_area(&self) -> f32 {
        self.members.iter().map(|h| h.area).sum()
    }

    fn get_weight(&self) -> f32 {
        self.members.iter().map(|h| h.weight).sum()
    }

    fn calculate_boarding_time(&self, elevator: &Elevator) -> f32 {
        self.members.len() as f32 * 2.0 
    }

    fn get_destination(&self) -> usize {
        self.members[0].destination_floor
    }

    fn increase_wait_time(&mut self, increment: f32) {
        self.members.iter_mut().for_each(|h| h.queue_wait += increment); 
    }

    fn get_queue_wait(&self) -> f32 {
        self.members.iter().map(|h| h.queue_wait).sum::<f32>() / self.members.len() as f32
    }

    fn get_transport_wait(&self) -> f32 {
        self.members.iter().map(|h| h.queue_wait).sum::<f32>() / self.members.len() as f32
    }

    fn board(&mut self) {
        self.members.iter_mut().for_each(|h| h.is_boarded = true);
    }
}