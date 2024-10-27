// src/socket/socket_test.rs

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Elevator {
    id: usize,
    current_floor: usize,
    current_height: f32, // in meters
    moving: bool,
    passengers: Vec<Passenger>,
    capacity: f32, // in kg
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Passenger {
    destination_floor: usize,
    group_size: usize,
    weight: f32,      // in kg
    is_disabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Floor {
    waiting: Vec<Passenger>,
    stairs: Vec<Passenger>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SimulationState {
    elevators: Vec<Elevator>,
    floors: HashMap<usize, Floor>,
    simulation_time: String,
    floor_count: usize,
    elevator_count: usize,
    floor_heights: Vec<f32>,
    elevator_door_width: f32,
    elevator_width: f32,
    elevator_depth: f32,
    time_multiplier: f32,
}

impl SimulationState {
    fn moving_simulation(&mut self) {
        // Implement actual simulation logic here
        for elevator in self.elevators.iter_mut() {
            elevator.moving = true; // Placeholder
        }
    }

    fn stop_simulation(&mut self) {
        // Implement actual stopping logic here
        for elevator in self.elevators.iter_mut() {
            elevator.moving = false; // Placeholder
        }
    }

    fn toggle_doors(&mut self, doors_open: bool) {
        // Implement door toggling logic here
        if doors_open {
            // Placeholder: Unload passengers
            for elevator in self.elevators.iter_mut() {
                elevator.passengers.clear();
            }
        } else {
            // Placeholder: Load passengers
            // Implement actual loading logic
        }
    }
}

#[derive(Deserialize, Debug)]
struct Request {
    action: String,
    #[serde(flatten)]
    params: HashMap<String, serde_json::Value>,
}

fn handle_client(mut stream: TcpStream, simulation: Arc<Mutex<SimulationState>>) {
    let mut buffer = [0; 4096];
    loop {
        // Read data from the client
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                // Connection was closed
                break;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        };

        // Deserialize the request
        let request_str = match std::str::from_utf8(&buffer[..bytes_read]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {}", e);
                let error_response = json!({
                    "success": false,
                    "message": "Invalid JSON format."
                });
                let _ = stream.write_all(error_response.to_string().as_bytes());
                continue;
            }
        };

        let request: Request = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                let error_response = json!({
                    "success": false,
                    "message": "Invalid JSON format."
                });
                let _ = stream.write_all(error_response.to_string().as_bytes());
                continue;
            }
        };

        // Handle the request based on the action
        let response = match request.action.as_str() {
            "get_state" => {
                let sim = simulation.lock().unwrap();
                serde_json::to_value(&*sim).unwrap()
            }
            "start_simulation" => {
                {
                    let mut sim = simulation.lock().unwrap();
                    sim.moving_simulation();
                } // Mutable borrow ends here

                json!({
                    "success": true,
                    "message": "Simulation started."
                })
            }
            "stop_simulation" => {
                {
                    let mut sim = simulation.lock().unwrap();
                    sim.stop_simulation();
                } // Mutable borrow ends here

                json!({
                    "success": true,
                    "message": "Simulation stopped."
                })
            }
            "update_settings" => {
                if let Some(settings) = request.params.get("settings") {
                    {
                        let mut sim = simulation.lock().unwrap();
                        // Update settings based on received parameters
                        if let Some(floor_count) = settings.get("floor_count").and_then(|f| f.as_u64()) {
                            sim.floor_count = floor_count as usize;
                            // Adjust floor_heights if necessary
                            let floor_heights_len = sim.floor_heights.len();
                            let target_len = sim.floor_count - 1;
                            if floor_heights_len < target_len {
                                let extension = vec![3.0; target_len - floor_heights_len];
                                sim.floor_heights.extend(extension);
                            }
                        }
                        if let Some(elevator_count) = settings.get("elevator_count").and_then(|e| e.as_u64()) {
                            sim.elevator_count = elevator_count as usize;
                            // Add or remove elevators as needed
                            let new_elevator = Elevator {
                                id: sim.elevators.len() + 1,
                                current_floor: 1,
                                current_height: 0.0,
                                moving: false,
                                passengers: vec![],
                                capacity: settings
                                    .get("elevator_capacity")
                                    .and_then(|c| c.as_f64())
                                    .unwrap_or(1000.0) as f32,
                            };
                            
                            while sim.elevators.len() < sim.elevator_count {
                                sim.elevators.push(new_elevator.clone());
                            }
                            while sim.elevators.len() > sim.elevator_count {
                                sim.elevators.pop();
                            }
                        }
                        if let Some(capacity) = settings.get("elevator_capacity").and_then(|c| c.as_f64()) {
                            for elevator in sim.elevators.iter_mut() {
                                elevator.capacity = capacity as f32;
                            }
                        }
                        if let Some(door_width) = settings
                            .get("elevator_door_width")
                            .and_then(|d| d.as_f64())
                        {
                            sim.elevator_door_width = door_width as f32;
                        }
                        if let Some(width) = settings.get("elevator_width").and_then(|w| w.as_f64()) {
                            sim.elevator_width = width as f32;
                        }
                        if let Some(depth) = settings.get("elevator_depth").and_then(|d| d.as_f64()) {
                            sim.elevator_depth = depth as f32;
                        }
                        if let Some(floor_heights) = settings.get("floor_heights").and_then(|fh| fh.as_array()) {
                            sim.floor_heights = floor_heights
                                .iter()
                                .filter_map(|h| h.as_f64())
                                .map(|h| h as f32)
                                .collect();
                        }
                    } // Mutable borrow ends here
                    json!({
                        "success": true,
                        "message": "Settings updated."
                    })
                } else {
                    json!({
                        "success": false,
                        "message": "No settings provided."
                    })
                }
            }
            "toggle_doors" => {
                if let Some(doors_open) = request.params.get("doors_open").and_then(|d| d.as_bool()) {
                    {
                        let mut sim = simulation.lock().unwrap();
                        sim.toggle_doors(doors_open);
                    } // Mutable borrow ends here
                    let state = if doors_open { "opened" } else { "closed" };
                    json!({
                        "success": true,
                        "message": format!("Doors have been {}.", state)
                    })
                } else {
                    json!({
                        "success": false,
                        "message": "No door state provided."
                    })
                }
            }
            "update_speed" => {
                if let Some(time_multiplier) =
                    request.params.get("time_multiplier").and_then(|tm| tm.as_f64())
                {
                    {
                        let mut sim = simulation.lock().unwrap();
                        sim.time_multiplier = time_multiplier as f32;
                    } // Mutable borrow ends here
                    json!({
                        "success": true,
                        "message": format!("Time multiplier set to {}.", time_multiplier)
                    })
                } else {
                    json!({
                        "success": false,
                        "message": "No time multiplier provided."
                    })
                }
            }
            "skip_to_time" => {
                if let Some(new_time) = request.params.get("new_time").and_then(|nt| nt.as_str()) {
                    {
                        let mut sim = simulation.lock().unwrap();
                        sim.simulation_time = new_time.to_string();
                    } // Mutable borrow ends here
                    json!({
                        "success": true,
                        "message": format!("Simulation time skipped to {}.", new_time)
                    })
                } else {
                    json!({
                        "success": false,
                        "message": "No new time provided."
                    })
                }
            }
            "set_manual_target" => {
                let elevator_id = request.params.get("elevator_id").and_then(|eid| eid.as_u64());
                let target_floor = request.params.get("target_floor").and_then(|tf| tf.as_u64());

                if let (Some(elevator_id), Some(target_floor)) = (elevator_id, target_floor) {
                    let mut response_msg = String::new();
                    let success;
                    {
                        let mut sim = simulation.lock().unwrap();
                        if let Some(elevator) = sim.elevators.iter_mut().find(|e| e.id == elevator_id as usize) {
                            elevator.current_floor = target_floor as usize;
                            // To prevent panic if target_floor is 1 or below, handle indexing carefully
                            let floor_heights = {
                                let sim = simulation.lock().unwrap();
                                sim.floor_heights.clone()
                            };
                            elevator.current_height = if target_floor >= 2 && ((target_floor as u64) - 2) < floor_heights.len() as u64 {
                                (target_floor as f32 - 1.0) * floor_heights[target_floor as usize - 2]
                            } else {
                                0.0
                            };
                            elevator.moving = true; // Simulate movement
                            success = true;
                            response_msg = format!("Elevator {} set to floor {}.", elevator_id, target_floor);
                        } else {
                            success = false;
                            response_msg = format!("Elevator {} not found.", elevator_id);
                        }
                    } // Mutable borrow ends here
                    if success {
                        json!({
                            "success": true,
                            "message": response_msg
                        })
                    } else {
                        json!({
                            "success": false,
                            "message": response_msg
                        })
                    }
                } else {
                    json!({
                        "success": false,
                        "message": "Missing elevator_id or target_floor."
                    })
                }
            }
            "get_elevators" => {
                let sim = simulation.lock().unwrap();
                serde_json::to_value(&sim.elevators).unwrap()
            }
            _ => {
                json!({
                    "success": false,
                    "message": "Unknown action."
                })
            }
        };

        // Serialize and send the response
        let response_str = match serde_json::to_string(&response) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialize response: {}", e);
                json!({
                    "success": false,
                    "message": "Internal server error."
                })
                .to_string()
            }
        };

        if let Err(e) = stream.write_all(response_str.as_bytes()) {
            eprintln!("Failed to send response to client: {}", e);
            break;
        }
    }
}
fn start_server(address: &str, running: Arc<AtomicBool>) {
    let listener = TcpListener::bind(address).expect("Failed to bind to address");
    while running.load(Ordering::SeqCst) {
        if let Ok((stream, _)) = listener.accept() {
            let simulation = Arc::new(Mutex::new(SimulationState {
                elevators: vec![],
                floors: HashMap::new(),
                simulation_time: "00:00:00".to_string(),
                floor_count: 0,
                elevator_count: 0,
                floor_heights: vec![],
                elevator_door_width: 0.0,
                elevator_width: 0.0,
                elevator_depth: 0.0,
                time_multiplier: 1.0,
            }));
            thread::spawn(move || {
                handle_client(stream, simulation);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::time::Duration;

    // Helper function to start the server in a separate thread for testing
    fn spawn_test_server(address: &str) -> (thread::JoinHandle<()>, Arc<AtomicBool>) {
        let running = Arc::new(AtomicBool::new(true));
        let server_running = Arc::clone(&running);

        let address = address.to_string();
        let handle = thread::spawn(move || {
            start_server(&address, server_running);
        });

        // Give the server a moment to start
        thread::sleep(Duration::from_millis(100));

        (handle, running)
    }

    // Helper function to send a JSON request and receive a response
    fn send_request(address: &str, request: serde_json::Value) -> serde_json::Value {
        let mut stream = TcpStream::connect(address).expect("Failed to connect to server");
        let request_str = serde_json::to_string(&request).unwrap();
        stream
            .write_all(request_str.as_bytes())
            .expect("Failed to send request");

        let mut buffer = [0; 4096];
        let bytes_read = stream
            .read(&mut buffer)
            .expect("Failed to read response");

        let response_str = std::str::from_utf8(&buffer[..bytes_read]).expect("Invalid UTF-8 in response");
        serde_json::from_str(response_str).expect("Failed to parse JSON response")
    }

    #[test]
    fn test_get_state() {
        let address = "127.0.0.1:9001";
        let (server_handle, running) = spawn_test_server(address);

        let request = json!({
            "action": "get_state"
        });

        let response = send_request(address, request);
        assert!(response.get("success").is_none()); // get_state returns the state, not a success message

        // Optionally, check specific fields in the response
        assert!(response.get("elevators").is_some());
        assert!(response.get("floors").is_some());

        // Stop the server
        running.store(false, Ordering::SeqCst);
        // Connect again to unblock the server
        TcpStream::connect(address).expect("Failed to connect to stop the server");
        server_handle.join().unwrap();
    }

    #[test]
    fn test_start_and_stop_simulation() {
        let address = "127.0.0.1:9002";
        let (server_handle, running) = spawn_test_server(address);

        // Start Simulation
        let start_request = json!({
            "action": "start_simulation"
        });

        let start_response = send_request(address, start_request);
        assert_eq!(start_response.get("success").unwrap(), &json!(true));
        assert_eq!(
            start_response.get("message").unwrap(),
            &json!("Simulation started.")
        );

        // Stop Simulation
        let stop_request = json!({
            "action": "stop_simulation"
        });

        let stop_response = send_request(address, stop_request);
        assert_eq!(stop_response.get("success").unwrap(), &json!(true));
        assert_eq!(
            stop_response.get("message").unwrap(),
            &json!("Simulation stopped.")
        );

        // Stop the server
        running.store(false, Ordering::SeqCst);
        // Connect again to unblock the server
        TcpStream::connect(address).expect("Failed to connect to stop the server");
        server_handle.join().unwrap();
    }

    #[test]
    fn test_update_settings() {
        let address = "127.0.0.1:9003";
        let (server_handle, running) = spawn_test_server(address);

        let update_request = json!({
            "action": "update_settings",
            "settings": {
                "floor_count": 15,
                "elevator_count": 3,
                "elevator_capacity": 1200.0,
                "elevator_door_width": 1.2,
                "floor_heights": [3.0, 3.5, 4.0, 3.0, 3.5, 4.0, 3.0, 3.5, 4.0, 3.0, 3.5, 4.0, 3.0, 3.5]
            }
        });

        let update_response = send_request(address, update_request);
        assert_eq!(update_response.get("success").unwrap(), &json!(true));
        assert_eq!(
            update_response.get("message").unwrap(),
            &json!("Settings updated.")
        );

        // Verify the updated settings by getting the state
        let get_state_request = json!({
            "action": "get_state"
        });

        let state_response = send_request(address, get_state_request);
        assert_eq!(state_response.get("floor_count").unwrap(), &json!(15));
        assert_eq!(state_response.get("elevator_count").unwrap(), &json!(3));
        // Additional assertions can be added as needed

        // Stop the server
        running.store(false, Ordering::SeqCst);
        // Connect again to unblock the server
        TcpStream::connect(address).expect("Failed to connect to stop the server");
        server_handle.join().unwrap();
    }

    #[test]
    fn test_toggle_doors() {
        let address = "127.0.0.1:9004";
        let (server_handle, running) = spawn_test_server(address);

        // Open Doors
        let open_doors_request = json!({
            "action": "toggle_doors",
            "doors_open": true
        });

        let open_doors_response = send_request(address, open_doors_request);
        assert_eq!(open_doors_response.get("success").unwrap(), &json!(true));
        assert_eq!(
            open_doors_response.get("message").unwrap(),
            &json!("Doors have been opened.")
        );

        // Close Doors
        let close_doors_request = json!({
            "action": "toggle_doors",
            "doors_open": false
        });

        let close_doors_response = send_request(address, close_doors_request);
        assert_eq!(close_doors_response.get("success").unwrap(), &json!(true));
        assert_eq!(
            close_doors_response.get("message").unwrap(),
            &json!("Doors have been closed.")
        );

        // Stop the server
        running.store(false, Ordering::SeqCst);
        // Connect again to unblock the server
        TcpStream::connect(address).expect("Failed to connect to stop the server");
        server_handle.join().unwrap();
    }

    #[test]
    fn test_set_manual_target() {
        let address = "127.0.0.1:9005";
        let (server_handle, running) = spawn_test_server(address);

        let manual_target_request = json!({
            "action": "set_manual_target",
            "elevator_id": 1,
            "target_floor": 5
        });

        let manual_target_response = send_request(address, manual_target_request);
        assert_eq!(manual_target_response.get("success").unwrap(), &json!(true));
        assert_eq!(
            manual_target_response.get("message").unwrap(),
            &json!("Elevator 1 set to floor 5.")
        );

        // Optionally, verify the elevator's state
        let get_elevators_request = json!({
            "action": "get_elevators"
        });

        let elevators_response = send_request(address, get_elevators_request);
        assert!(elevators_response.is_array());
        let elevators = elevators_response.as_array().unwrap();
        let elevator1 = elevators.iter().find(|e| e.get("id").unwrap() == &json!(1)).unwrap();
        assert_eq!(elevator1.get("current_floor").unwrap(), &json!(5));
        assert_eq!(elevator1.get("moving").unwrap(), &json!(true));

        // Stop the server
        running.store(false, Ordering::SeqCst);
        // Connect again to unblock the server
        TcpStream::connect(address).expect("Failed to connect to stop the server");
        server_handle.join().unwrap();
    }

    #[test]
    fn test_unknown_action() {
        let address = "127.0.0.1:9006";
        let (server_handle, running) = spawn_test_server(address);

        let unknown_request = json!({
            "action": "unknown_action"
        });

        let unknown_response = send_request(address, unknown_request);
        assert_eq!(unknown_response.get("success").unwrap(), &json!(false));
        assert_eq!(
            unknown_response.get("message").unwrap(),
            &json!("Unknown action.")
        );

        // Stop the server
        running.store(false, Ordering::SeqCst);
        // Connect again to unblock the server
        TcpStream::connect(address).expect("Failed to connect to stop the server");
        server_handle.join().unwrap();
    }
}