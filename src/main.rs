use elevator_lib::algorithms::{ 
    SimpleElevatorController, 
    NearestCarDispatchController,
};
use elevator_lib::machine::ElevatorSystem;


fn main() {
    println!("    
        elevator_optimization  Copyright (C) 2024  Tuna Gül
        This program comes with ABSOLUTELY NO WARRANTY;
        This is free software, and you are welcome to redistribute it
        under certain conditions;
    ");

    let controller = Box::new(SimpleElevatorController::new());
    // let controller = Box::new(NearestCarDispatchController::new(2));

    let mut system = ElevatorSystem::from_file(controller, "param/system_parameters.yaml")
        .expect("Failed to create elevator system");

    // system.update() bitince false veriyor
    while system.update() {

    }

    println!("Total energy cost: {:.2}", system.final_kwh() * 2.7);
    println!("Total wait time: {:.2}", system.get_total_wait_time());
}