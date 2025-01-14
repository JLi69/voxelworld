use crate::voxel::{Block, Chunk};
use fastrand::Rng;

use super::{BOTTOM_OF_WORLD, LAVA_LEVEL};

fn gen_vein(
    chunk: &mut Chunk,
    xyz: (i32, i32, i32),
    size1: i32,
    size2: i32,
    ore: u8,
    probability: f64,
    rng: &mut Rng,
) {
    let (x, y, z) = xyz;
    let ore_block = Block::new_id(ore);
    chunk.set_block(x, y, z, ore_block);
    for ix in (x - size2)..=(x + size1) {
        for iy in (y - size2)..=(y + size1) {
            for iz in (z - size2)..=(z + size1) {
                let block = chunk.get_block(ix, iy, iz);
                //Do not generate any ore in blocks that are not stone
                if block.id != 2 && block.id != 15 {
                    continue;
                }

                if rng.f64() < probability {
                    chunk.set_block(ix, iy, iz, ore_block);
                }
            }
        }
    }
}

//starty > y > endy
fn get_probability(y: i32, starty: i32, endy: i32, min_prob: f64, max_prob: f64) -> f64 {
    if y > starty {
        return 0.0;
    }
    let frac = 1.0 + (y - endy) as f64 / (endy - starty) as f64;
    frac * (max_prob - min_prob) + min_prob
}

pub fn generate_ore(chunk: &mut Chunk, x: i32, y: i32, z: i32, rng: &mut Rng) {
    //If it is not stone, ignore it
    let block = chunk.get_block(x, y, z);
    if block.id != 2 && block.id != 15 {
        return;
    }

    //Generate coal ore
    //Generates where ever stone generates
    if y < 0 && rng.i32(0..2_000) == 0 {
        gen_vein(chunk, (x, y, z), 2, 2, 18, 0.05, rng);
    } else if y >= 0 && rng.i32(0..1_000) == 0 {
        gen_vein(chunk, (x, y, z), 1, 1, 18, 0.2, rng);
    }

    //Generate crimson crystal ore
    //Generates below y = -32
    if rng.f64() < get_probability(y, -32, BOTTOM_OF_WORLD, 1.0 / 8_000.0, 1.0 / 2_000.0) {
        gen_vein(chunk, (x, y, z), 1, 1, 23, 0.3, rng);
    }

    //Generate iron ore
    //Generates below y = -16
    if rng.f64() < get_probability(y, 0, BOTTOM_OF_WORLD, 1.0 / 4_000.0, 1.0 / 2_000.0) {
        gen_vein(chunk, (x, y, z), 0, 1, 19, 0.5, rng);
    }

    //Generate gold ore
    //Generates below y = -32
    if rng.f64() < get_probability(y, -32, BOTTOM_OF_WORLD, 1.0 / 5_000.0, 1.0 / 4_000.0) {
        gen_vein(chunk, (x, y, z), 0, 1, 20, 0.33, rng);
    }

    //Generate diamond ore
    //Generates below y = -40
    if rng.f64() < get_probability(y, -40, BOTTOM_OF_WORLD, 1.0 / 5_500.0, 1.0 / 4_500.0) {
        gen_vein(chunk, (x, y, z), 0, 1, 21, 0.25, rng);
    }

    //Generate rainbow ore
    //Generates below y = -50
    if y < -50 && rng.i32(0..5_000) == 0 {
        chunk.set_block(x, y, z, Block::new_id(22));
    }
}

pub fn generate_magma_blocks(chunk: &mut Chunk, x: i32, y: i32, z: i32, rng: &mut Rng) {
    //If it is not stone, ignore it
    if chunk.get_block(x, y, z).id != 2 {
        return;
    }

    if rng.f64()
        < get_probability(
            y,
            LAVA_LEVEL + 4,
            BOTTOM_OF_WORLD,
            1.0 / 3_000.0,
            1.0 / 1_000.0,
        )
    {
        gen_vein(chunk, (x, y, z), 3, 3, 15, 0.66, rng);
    }
}
