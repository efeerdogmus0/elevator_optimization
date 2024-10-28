// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2024 Tuna Gül

#[cfg(test)]
use super::*;

#[test]
fn read() {
    let parameters = MotorParameters::from_file("param/motor_test_parameters.yaml").unwrap();
    println!("{:?}", parameters);
}