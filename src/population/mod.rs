// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

mod boardable;
mod human;

mod population_generator;

pub use population_generator::PopulationGenerator;
pub use human::{ Human, HumanGroup, Gender};
pub use boardable::Boardable;