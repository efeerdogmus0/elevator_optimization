use rand::seq::index::IndexVec;

use crate::machine::ElevatorSystem;
use crate::algorithms::NeuralControlAlgorithm;

use super::neural::NeuralNetwork;


pub fn reward(
    system: &ElevatorSystem,
) -> f32{
    -system.get_total_wait_time() -system.final_kwh() * 2.7
}

fn run_generation(
    networks: &mut Vec<NeuralControlAlgorithm>,
    population_size: usize,
    run_per_network: usize,
) -> Vec<f32> {
    let mut scores = vec![0.0; population_size];
    for i in 0..population_size {
        for _ in 0..run_per_network {
            let controller = Box::new(networks[i].clone());
            let mut system = ElevatorSystem::from_file(controller, "param/system_parameters.yaml")
                .expect("Failed to create elevator system");

            while system.update() {

            }

            scores[i] += reward(&system);
        }
        scores[i] /= run_per_network as f32;
    }
    scores
}

fn sort_by_score(
    networks: &mut Vec<NeuralControlAlgorithm>,
    scores: &mut Vec<f32>,
    population_size: usize,
) {
    for i in 0..population_size {
        for j in i+1..population_size {
            if scores[i] < scores[j] {
                scores.swap(i, j);
                networks.swap(i, j);
            }
        }
    }
}

fn next_generation(
    networks: Vec<NeuralControlAlgorithm>,
    population_size: usize,
    mutation_prob: f32,
    mutation_amount: f32,
    evolutionary_pressure: f32,
) -> Vec<NeuralControlAlgorithm> {
    let count_to_keep = ((networks.len() as f32) * (1.0 - evolutionary_pressure)).ceil() as usize;
    let child_count = (population_size as f32 / count_to_keep as f32).ceil() as usize;
    let mut new_gen = Vec::new();

    for i in 0..count_to_keep {
        for j in 0..child_count {
            if new_gen.len() >= population_size {
                break;
            }

            // önce kendisini kopyala sonra mutasyon yap
            if j == 0 {
                new_gen.push(networks[i].clone());
                continue;
            }

            let new_network = NeuralControlAlgorithm::new_with_nn(
                networks[i]
                .get_network()
                .mutate(mutation_prob, mutation_amount)
            );
            new_gen.push(new_network);

        }

        if new_gen.len() >= population_size {
            break;
        }
    }
    new_gen
}

pub fn train(
    generation_count: usize,
    population_size: usize,
    mutation_prob: f32,
    mutation_amount: f32,
    run_per_network: usize,
    evolutionary_pressure: f32,
) -> NeuralNetwork {
    if evolutionary_pressure < 0.0 || evolutionary_pressure > 1.0 {
        panic!("Invalid evolutionary pressure value.");
    }

    let mut networks = Vec::new();
    for _ in 0..population_size {
        networks.push(NeuralControlAlgorithm::new());
    }

    for i in 0..generation_count {
        let mut scores = run_generation(&mut networks, population_size, run_per_network);
        sort_by_score(&mut networks, &mut scores, population_size);
        println!("Generation {} - Best score: {:.2}", generation_count, scores[0]);

        if i == generation_count - 1 { break; }
        networks = next_generation(networks, population_size, mutation_prob, mutation_amount, evolutionary_pressure);
    }  

    networks[0].get_network().clone()
}
