use crate::voxel::{Block, Chunk, CHUNK_SIZE_I32};
use fastrand::Rng;

use super::{BOTTOM_OF_WORLD, LAVA_LEVEL, SEA_LEVEL};

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
    if y < -16 && rng.i32(0..2_000) == 0 {
        gen_vein(chunk, (x, y, z), 2, 2, 18, 0.05, rng);
    } else if y >= 0 && rng.i32(0..1_000) == 0 {
        gen_vein(chunk, (x, y, z), 1, 1, 18, 0.2, rng);
    }

    //Generate crimson crystal ore
    //Generates below y = -32
    if rng.f64() < get_probability(y, -32, BOTTOM_OF_WORLD, 1.0 / 8_000.0, 1.0 / 2_000.0) {
        gen_vein(chunk, (x, y, z), 1, 1, 23, 0.3, rng);
    }

    //Generate uranium ore
    //Generate below y = -40
    if rng.f64() < get_probability(y, -40, BOTTOM_OF_WORLD, 1.0 / 5_500.0, 1.0 / 4_000.0) {
        gen_vein(chunk, (x, y, z), 1, 1, 94, 0.4, rng);
    }

    //Generate iron ore
    //Generates below y = 0
    if rng.f64() < get_probability(y, 0, BOTTOM_OF_WORLD, 1.0 / 4_000.0, 1.0 / 2_000.0) {
        gen_vein(chunk, (x, y, z), 0, 1, 19, 0.5, rng);
    }

    //Generate gold ore
    //Generates below y = -32
    if rng.f64() < get_probability(y, -32, BOTTOM_OF_WORLD, 1.0 / 4_500.0, 1.0 / 3_000.0) {
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

const CLAY_RADIUS: i32 = 2;

pub fn generate_clay(chunk: &mut Chunk, x: i32, y: i32, z: i32, rng: &mut Rng) {
    //If it is not sand, ignore it
    if chunk.get_block(x, y, z).id != 11 {
        return;
    }

    let pos = chunk.get_chunk_pos();
    let minx = pos.x * CHUNK_SIZE_I32 + CLAY_RADIUS;
    let miny = pos.y * CHUNK_SIZE_I32 + CLAY_RADIUS;
    let minz = pos.z * CHUNK_SIZE_I32 + CLAY_RADIUS;
    if x < minx || y < miny || z < minz {
        return;
    }

    let maxx = pos.x * CHUNK_SIZE_I32 + CHUNK_SIZE_I32 - 1 - CLAY_RADIUS;
    let maxy = pos.y * CHUNK_SIZE_I32 + CHUNK_SIZE_I32 - 1 - CLAY_RADIUS;
    let maxz = pos.z * CHUNK_SIZE_I32 + CHUNK_SIZE_I32 - 1 - CLAY_RADIUS;
    if x > maxx || y > maxy || z > maxz {
        return;
    }

    if y >= SEA_LEVEL - 2 {
        return;
    }

    if rng.f64() > 1.0 / 640.0 {
        return;
    }

    for ix in (x - CLAY_RADIUS)..=(x + CLAY_RADIUS) {
        for iy in (y - CLAY_RADIUS)..=(y + CLAY_RADIUS) {
            for iz in (z - CLAY_RADIUS)..=(z + CLAY_RADIUS) {
                let dist = (ix - x).pow(2) + (iy - y).pow(2) + (iz - z).pow(2);
                if dist > CLAY_RADIUS * CLAY_RADIUS {
                    continue;
                }

                let block = chunk.get_block(ix, iy, iz);
                if block.id != 11 {
                    continue;
                }
                if y >= SEA_LEVEL - 1 {
                    continue;
                }
                chunk.set_block(ix, iy, iz, Block::new_id(93));
            }
        }
    }
}

const AQUA_VEIN_SIZE: i32 = 1;

pub fn generate_aqua_ore(chunk: &mut Chunk, x: i32, y: i32, z: i32, rng: &mut Rng) {
    //If it is not sand, ignore it
    if chunk.get_block(x, y, z).id != 11 {
        return;
    }

    let pos = chunk.get_chunk_pos();
    let minx = pos.x * CHUNK_SIZE_I32 + AQUA_VEIN_SIZE;
    let miny = pos.y * CHUNK_SIZE_I32 + AQUA_VEIN_SIZE;
    let minz = pos.z * CHUNK_SIZE_I32 + AQUA_VEIN_SIZE;
    if x < minx || y < miny || z < minz {
        return;
    }

    let maxx = pos.x * CHUNK_SIZE_I32 + CHUNK_SIZE_I32 - 1 - AQUA_VEIN_SIZE;
    let maxy = pos.y * CHUNK_SIZE_I32 + CHUNK_SIZE_I32 - 1 - AQUA_VEIN_SIZE;
    let maxz = pos.z * CHUNK_SIZE_I32 + CHUNK_SIZE_I32 - 1 - AQUA_VEIN_SIZE;
    if x > maxx || y > maxy || z > maxz {
        return;
    }

    if y >= SEA_LEVEL - 4 {
        return;
    }

    if rng.f64() > 1.0 / 1600.0 {
        return;
    }

    for ix in (x - AQUA_VEIN_SIZE)..=(x + AQUA_VEIN_SIZE) {
        for iy in (y - AQUA_VEIN_SIZE)..=(y + AQUA_VEIN_SIZE) {
            for iz in (z - AQUA_VEIN_SIZE)..=(z + AQUA_VEIN_SIZE) {
                let block = chunk.get_block(ix, iy, iz);
                if block.id != 11 {
                    continue;
                }
                if y >= SEA_LEVEL - 1 {
                    continue;
                }
                if rng.f64() > 0.66 {
                    continue;
                }
                chunk.set_block(ix, iy, iz, Block::new_id(96));
            }
        }
    }
}
