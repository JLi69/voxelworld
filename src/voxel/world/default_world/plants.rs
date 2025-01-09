use crate::voxel::{Chunk, CHUNK_SIZE_I32, Block};
use super::{WorldGenerator, terrain::get_height, SAND_LEVEL, is_noise_cave};
use fastrand::Rng;

pub fn get_plant_positions(chunkx: i32, chunkz: i32, world_seed: u32) -> Vec<(i32, i32)> {
    let xu32 = chunkx as u32;
    let zu32 = chunkz as u32;
    let seed = ((zu32 as u64) << 32) | (xu32 as u64);
    let mut plant_generator = fastrand::Rng::with_seed(seed);
    let mut rng = fastrand::Rng::with_seed(seed + ((world_seed as u64) << 32));
    let count = plant_generator.i32(0..80);
    let mut positions = vec![];
    for _ in 0..count {
        let plantx = (plant_generator.i32(0..32) + rng.i32(0..32)) % 32;
        let plantz = (plant_generator.i32(0..32) + rng.i32(0..32)) % 32;
        let x = plantx + chunkx * CHUNK_SIZE_I32;
        let z = plantz + chunkz * CHUNK_SIZE_I32;
        positions.push((x, z));
    }
    positions
}

pub fn generate_plants(
    chunk: &mut Chunk,
    plant_positions: &[(i32, i32)],
    rng: &mut Rng,
    world_generator: &WorldGenerator,
) {
    for (x, z) in plant_positions {
        let rand_val = rng.i32(0..80);
        let h = get_height(*x, *z, &world_generator.terrain_generator);

        if chunk.get_block(*x, h + 1, *z) != Block::new() {
            continue;
        }

        //Below sea level
        if h <= SAND_LEVEL {
            continue;
        }

        //Check to make sure we are not in a cave (an empty block)
        if is_noise_cave(*x, h, *z, &world_generator.noise_cave_generator) {
            continue;
        }

        match rand_val {
            //Tall grass
            0..50 => chunk.set_block(*x, h + 1, *z, Block::new_id(49)), 
            //Red flower
            50..58 => chunk.set_block(*x, h + 1, *z, Block::new_id(54)),
            //Yellow flower
            58..66 => chunk.set_block(*x, h + 1, *z, Block::new_id(55)),
            //Blue flower
            66..70 => chunk.set_block(*x, h + 1, *z, Block::new_id(56)),
            //Mushroom
            70..72 => chunk.set_block(*x, h + 1, *z, Block::new_id(48)),
            //Nothing
            _ => {}
        }
    }
}
