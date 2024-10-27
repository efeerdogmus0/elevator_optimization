use elevator_lib::*;
use elevator_lib::algorithms::SimpleElevatorController;
use elevator_lib::machine::ElevatorSystem;


use std::time::{ Duration, Instant };
use std::thread;


fn main() {
    println!("    
        elevator_optimization  Copyright (C) 2024  Tuna Gül
        This program comes with ABSOLUTELY NO WARRANTY;
        This is free software, and you are welcome to redistribute it
        under certain conditions;
    ");
    let controller = Box::new(SimpleElevatorController::new());
    let mut system = ElevatorSystem::from_file(controller, "param/elevator_system_test_parameters.yaml")
        .expect("Failed to create elevator system");

    let start = Instant::now();
    loop {
        system.update();
        // if start.elapsed().as_secs() > 60 {
        //     break;
        // }
        // thread::sleep(Duration::from_millis(1000));
    }
    let energy = system.get_used_energy();
    println!("Total energy used: {:.2} J", energy);
}