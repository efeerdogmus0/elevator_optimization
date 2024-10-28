// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

mod state_space;
mod neural;
mod neural_wrapper;
mod train_network;

pub use neural_wrapper::NeuralControlAlgorithm;
pub use train_network::train_network;