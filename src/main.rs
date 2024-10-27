// elevator_optimization
// Copyright (C) 2024   Tuna Gül

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.


mod util;
mod machine;
mod algorithms;
mod population;

use machine::ElevatorSystem;
use algorithms::SimpleElevatorController;

use std::time::Instant;

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
        if start.elapsed().as_secs() > 60 {
            break;
        }
    }
    let energy = system.get_used_energy();
    println!("Total energy used: {:.2} J", energy);
}
