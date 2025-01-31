use super::rand_block_update::RANDOM_UPDATE_INTERVAL;
use crate::voxel::{world::WorldGenType, Block, World, EMPTY_BLOCK};

/*
 * Test simulations for testing purposes
 * Each simulation takes in the number of iterations to run and returns
 * the average amount of time it takes in minutes
 * */

fn simulate_wheat_growth(iterations: i32) -> f32 {
    eprintln!("WHEAT GROWTH SIMULATION");
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat);
        let mut total_time = 0.0;
        for x in 0..9 {
            for z in 0..9 {
                world.set_block(x, 0, z, Block::new_id(77));
                world.set_block(x, -1, z, Block::new_id(43));
            }
        }
        let mut done = false;
        while !done {
            world.rand_block_update(RANDOM_UPDATE_INTERVAL, None, 0);
            total_time += RANDOM_UPDATE_INTERVAL;
            done = true;
            for x in 0..9 {
                for z in 0..9 {
                    let block = world.get_block(x, 0, z);
                    if block.id != 53 {
                        done = false;
                    }
                }
            }
        }
        let minutes = total_time / 60.0;
        total += minutes;
        eprintln!(
            "({} / {iterations}) took {total_time} s ({minutes} min) to grow all wheat",
            i + 1
        );
    }
    total / iterations as f32
}

fn simulate_slow_wheat_growth(iterations: i32) -> f32 {
    eprintln!("SLOW WHEAT GROWTH SIMULATION");
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat);
        let mut total_time = 0.0;
        for x in 0..9 {
            for z in 0..9 {
                world.set_block(x, 0, z, Block::new_id(50));
                world.set_block(x, -1, z, Block::new_id(45));
            }
        }
        let mut done = false;
        while !done {
            world.rand_block_update(RANDOM_UPDATE_INTERVAL, None, 0);
            total_time += RANDOM_UPDATE_INTERVAL;
            done = true;
            for x in 0..9 {
                for z in 0..9 {
                    let block = world.get_block(x, 0, z);
                    if block.id != 53 {
                        done = false;
                    }
                }
            }
        }
        let minutes = total_time / 60.0;
        total += minutes;
        eprintln!(
            "({} / {iterations}) took {total_time} s ({minutes} min) to grow all wheat",
            i + 1
        );
    }
    total / iterations as f32
}

fn simulate_sugarcane_growth(iterations: i32) -> f32 {
    eprintln!("SUGAR CANE GROWTH SIMULATION");
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat);
        let mut total_time = 0.0;
        for x in 0..16 {
            world.set_block(x, 1, 0, Block::new_id(1));
            world.set_block(x, 2, 0, Block::new_id(69));
            world.set_block(x, 1, 1, Block::new_fluid(12));
        }
        let mut done = false;
        while !done {
            world.rand_block_update(RANDOM_UPDATE_INTERVAL, None, 0);
            total_time += RANDOM_UPDATE_INTERVAL;
            done = true;
            for x in 0..16 {
                let block = world.get_block(x, 4, 0);
                assert_eq!(world.get_block(x, 5, 0).id, EMPTY_BLOCK);
                if block.id != 69 {
                    done = false;
                }
            }
        }
        let minutes = total_time / 60.0;
        total += minutes;
        eprintln!(
            "({} / {iterations}) took {total_time} s ({minutes} min) to grow all sugarcane",
            i + 1
        );
    }
    total / iterations as f32
}

fn simulate_sapling_growth(iterations: i32) -> f32 {
    eprintln!("SAPLING GROWTH SIMULATION");
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat);
        let mut total_time = 0.0;
        world.set_block(0, 2, 0, Block::new_id(47));
        world.set_block(0, 1, 0, Block::new_id(1));
        let mut done = false;
        while !done {
            world.rand_block_update(RANDOM_UPDATE_INTERVAL, None, 0);
            total_time += RANDOM_UPDATE_INTERVAL;
            done = world.get_block(0, 2, 0).id == 8;
        }
        let minutes = total_time / 60.0;
        total += minutes;
        eprintln!(
            "({} / {iterations}) took {total_time} s ({minutes} min) to grow sapling",
            i + 1
        );
    }
    total / iterations as f32
}

pub fn run_test_simulations(args: &[String]) {
    if !args.contains(&"--run-test-sims".to_string()) {
        return;
    }
    //Run simulations and then quit the program
    let average_sapling_time = simulate_sapling_growth(100);
    let average_sugarcane_time = simulate_sugarcane_growth(100);
    let average_wheat_time = simulate_wheat_growth(100);
    let average_slow_wheat_time = simulate_slow_wheat_growth(100);
    //Output results
    eprintln!();
    eprintln!("Simulation Results");
    eprintln!("------------------");
    eprintln!("Average time to grow all wheat: {} min", average_wheat_time);
    eprintln!(
        "Average time to grow all wheat (slow): {} min",
        average_slow_wheat_time
    );
    eprintln!(
        "Average time to grow all sugarcane: {} min",
        average_sugarcane_time
    );
    eprintln!(
        "Average time to grow all sapling: {} min",
        average_sapling_time
    );
    //Exit program once all simulations are completed
    std::process::exit(0);
}
