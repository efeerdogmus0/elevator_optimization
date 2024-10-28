use crate::machine::{ Elevator, ElevatorSystem, Direction };
use crate::algorithms::SimpleElevatorController;
use pyo3::prelude::*;


#[pyfunction]
pub fn new_simple_system() -> ElevatorSystem {
    println!("    
        elevator_optimization  Copyright (C) 2024  Tuna Gül
        This program comes with ABSOLUTELY NO WARRANTY;
        This is free software, and you are welcome to redistribute it
        under certain conditions;");

    let controller = Box::new(SimpleElevatorController::new());
    let system = ElevatorSystem::from_file(controller, "param/system_parameters.yaml")
        .expect("Failed to create elevator system");

    system
}

#[pyfunction]
pub fn get_calls(system: &ElevatorSystem) -> Vec<Option<i32>> {
    let mut result: Vec<Option<i32>> = Vec::new();

    for call in system.get_call_buttons() {
        result.push(match call {
            Direction::Up => Some(1),
            Direction::Down => Some(-1),
            Direction::Both => Some(0),
            Direction::None => None,
        });
    }

    result
}


#[pymodule]
fn elevator_lib(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new_simple_system, m)?)?;
    m.add_class::<Elevator>()?;
    m.add_class::<ElevatorSystem>()?;
    Ok(())
}
