use super::rand_block_update::RANDOM_UPDATE_INTERVAL;
use crate::{
    game::GameMode,
    voxel::{world::WorldGenType, Block, World, EMPTY_BLOCK},
};

/*
 * Test simulations for testing purposes
 * Each simulation takes in the number of iterations to run and returns
 * the average amount of time it takes in minutes
 * */

fn simulate_sugarcane_growth(iterations: i32) -> String {
    eprintln!("SUGAR CANE GROWTH SIMULATION");
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat, GameMode::Creative);
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
    format!(
        "Average time to grow all sugarcane: {}",
        total / iterations as f32
    )
}

fn simulate_cactus_growth(iterations: i32) -> String {
    eprintln!("CACTUS GROWTH SIMULATION");
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat, GameMode::Creative);
        let mut total_time = 0.0;
        for x in 0..9 {
            for z in 0..9 {
                world.set_block(x, 1, z, Block::new_id(11));
                world.set_block(x, 2, z, Block::new_id(88));
            }
        }
        let mut done = false;
        while !done {
            world.rand_block_update(RANDOM_UPDATE_INTERVAL, None, 0);
            total_time += RANDOM_UPDATE_INTERVAL;
            done = true;
            for x in 0..9 {
                for z in 0..9 {
                    let block = world.get_block(x, 4, z);
                    assert_eq!(world.get_block(x, 5, 0).id, EMPTY_BLOCK);
                    if block.id != 88 {
                        done = false;
                    }
                }
            }
        }
        let minutes = total_time / 60.0;
        total += minutes;
        eprintln!(
            "({} / {iterations}) took {total_time} s ({minutes} min) to grow all cacti",
            i + 1
        );
    }
    format!(
        "Average time to grow all cacti: {} min",
        total / iterations as f32
    )
}

fn simulate_sapling_growth(iterations: i32) -> String {
    eprintln!("SAPLING GROWTH SIMULATION");
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat, GameMode::Creative);
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
    format!(
        "Average time to grow all saplings: {} min",
        total / iterations as f32
    )
}

fn simulate_snow_sapling_growth(iterations: i32) -> String {
    eprintln!("SNOW SAPLING GROWTH SIMULATION");
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat, GameMode::Creative);
        let mut total_time = 0.0;
        world.set_block(0, 2, 0, Block::new_id(92));
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
    format!(
        "Average time to grow all snow saplings: {}",
        total / iterations as f32
    )
}

fn simulate_crop_growth(
    iterations: i32,
    crop_name: &str,
    seed_id: u8,
    crop_id: u8,
    farmland_id: u8,
) -> String {
    eprintln!("{} GROWTH SIMULATION", crop_name.to_uppercase());
    let mut total = 0.0f32;
    for i in 0..iterations {
        let mut world = World::new(0, 1, WorldGenType::Flat, GameMode::Creative);
        let mut total_time = 0.0;
        for x in 0..4 {
            for z in 0..4 {
                world.set_block(x, 0, z, Block::new_id(seed_id));
                world.set_block(x, -1, z, Block::new_id(farmland_id));
            }
        }
        let mut done = false;
        while !done {
            world.rand_block_update(RANDOM_UPDATE_INTERVAL, None, 0);
            total_time += RANDOM_UPDATE_INTERVAL;
            done = true;
            for x in 0..4 {
                for z in 0..4 {
                    let block = world.get_block(x, 0, z);
                    if block.id != crop_id {
                        done = false;
                    }
                }
            }
        }
        let minutes = total_time / 60.0;
        total += minutes;
        eprintln!(
            "({} / {iterations}) took {total_time} s ({minutes} min) to grow all {crop_name}",
            i + 1
        );
    }
    format!(
        "Average time to grow all {crop_name}: {} min",
        total / iterations as f32
    )
}

pub fn run_test_simulations(args: &[String]) {
    if !args.contains(&"--run-test-sims".to_string()) {
        return;
    }
    //Run simulations and then quit the program
    let results = vec![
        simulate_sapling_growth(100),
        simulate_sugarcane_growth(100),
        simulate_crop_growth(100, "wheat", 77, 53, 43),
        simulate_crop_growth(100, "wheat (slow)", 77, 53, 45),
        simulate_cactus_growth(100),
        simulate_snow_sapling_growth(100),
        simulate_crop_growth(100, "cotton", 98, 102, 43),
        simulate_crop_growth(100, "cotton (slow)", 98, 102, 45),
        simulate_crop_growth(100, "red flowers", 103, 54, 43),
        simulate_crop_growth(100, "red flowers (slow)", 103, 54, 45),
        simulate_crop_growth(100, "yellow flowers", 105, 55, 43),
        simulate_crop_growth(100, "yellow flowers (slow)", 105, 55, 45),
        simulate_crop_growth(100, "blue flowers", 107, 56, 43),
        simulate_crop_growth(100, "blue flowers (slow)", 107, 56, 45),
    ];
    //Output results
    eprintln!();
    eprintln!("Simulation Results");
    eprintln!("------------------");
    for line in results {
        eprintln!("{line}");
    }
    //Exit program once all simulations are completed
    std::process::exit(0);
}
