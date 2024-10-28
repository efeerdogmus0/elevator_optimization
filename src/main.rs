use clap::{Parser, ArgGroup};
use elevator_lib::algorithms::{ 
    SimpleElevatorController, 
    // NearestCarDispatchController,
    NeuralControlAlgorithm,
    train_network,
    ElevatorControllerAlgorithm,
};

use elevator_lib::machine::ElevatorSystem;

#[derive(Parser)]
#[command(name = "elevator_optimization")]
#[command(about = "An elevator optimization simulator", long_about = None)]
#[command(group(ArgGroup::new("mode").required(true).args(&["train", "algorithm"])))]
struct Args {
    /// Specifies the algorithm to use: "simple", "nearest", or "neural"
    #[arg(long, value_parser = ["simple", "neural"])]
    algorithm: Option<String>,

    /// Train the neural network
    #[arg(long)]
    train: bool,

    /// Number of iterations for the simulation
    #[arg(long, default_value = "1")]
    iterations: usize,
}

fn main() {
    println!("    
        elevator_optimization  Copyright (C) 2024  Tuna Gül
        This program comes with ABSOLUTELY NO WARRANTY;
        This is free software, and you are welcome to redistribute it
        under certain conditions;
    ");

    let args = Args::parse();

    // If --train is specified, run the training function and exit
    if args.train {
        println!("Training the neural network...");
        let trained = train_network(
            100,
            30,
            0.1,
            0.1,
            10,
            0.2,
            10,
        );
        trained.save_to_file("trained_network.json").expect("Failed to save trained network");
        return;
    }

    // Select the controller based on the --algorithm argument
    let controller = match args.algorithm.as_deref() {
        Some("simple") => Box::new(SimpleElevatorController::new()) as Box<dyn ElevatorControllerAlgorithm>,
        // Some("nearest") => Box::new(NearestCarDispatchController::new(2)) as Box<dyn ElevatorControllerAlgorithm>,
        Some("neural") => Box::new(NeuralControlAlgorithm::new()) as Box<dyn ElevatorControllerAlgorithm>,
        _ => {
            eprintln!("Error: Invalid algorithm specified.");
            return;
        }
    };

    // Initialize the elevator system
    let mut system = ElevatorSystem::from_file(controller, "param/system_parameters.yaml")
        .expect("Failed to create elevator system");

    // Run the simulation for the specified number of iterations
    for _ in 0..args.iterations {
        while system.update() {}
    }

    // Output final results
    println!("Total energy cost: {:.2}", system.final_kwh() * 2.7);
    println!("Total wait time: {:.2}", system.get_total_wait_time());
}
