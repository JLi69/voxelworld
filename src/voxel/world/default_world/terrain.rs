use crate::voxel::{CHUNK_SIZE_I32, world::WorldGenerator};
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

pub type HeightMap = HashMap<(i32, i32), Vec<i32>>;

//cave_y1 = cave lower y bound
//cave_y2 = middle
//cave_y3 = cave upper y bound
fn cave_perc(y: i32, cave_y1: i32, cave_y2: i32, cave_y3: i32) -> f64 {
    if y > cave_y2 {
        let t = (y - cave_y2) as f64 / (cave_y3 - cave_y2) as f64;
        ((1.0 - t) * 0.8 - 1.0).max(-0.7)
    } else if y <= cave_y2 && y > cave_y1 {
        let t = (y - cave_y2) as f64 / (cave_y2 - cave_y1) as f64;
        (1.0 - t * t) * 0.8 - 1.0
    } else {
        -1.0
    }
}

pub fn is_noise_cave(x: i32, y: i32, z: i32, cave_noise: &Perlin) -> bool {
    let xyz = [x as f64 / 8.0, y as f64 / 8.0, z as f64 / 8.0];
    cave_noise.get(xyz) < cave_perc(y, -64, -51, 64)
}

pub fn get_height(x: i32, z: i32, world_generator: &WorldGenerator) -> i32 {
    let base = world_generator.get_base_elevation(x, z); 
    let steepness = world_generator.get_steepness(x, z); 
    let elevation = world_generator.get_elevation(x, z);
    
    let transformed_noise = base + steepness * elevation;
    (transformed_noise * 64.0) as i32
}

pub fn get_height_mountain(x: i32, z: i32, world_generator: &WorldGenerator) -> i32 {
    let mountain_h = (world_generator.get_mountain(x, z) * 80.0) as i32;
    get_height(x, z, world_generator).max(mountain_h)
}

pub fn generate_heightmap(
    positions: &Vec<(i32, i32, i32)>,
    world_generator: &WorldGenerator,
) -> HeightMap {
    let mut heightmap = HeightMap::new();

    for (chunkx, _, chunkz) in positions {
        if heightmap.contains_key(&(*chunkx, *chunkz)) {
            continue;
        }
        let posx = chunkx * CHUNK_SIZE_I32;
        let posz = chunkz * CHUNK_SIZE_I32;
        let mut heights = vec![0; (CHUNK_SIZE_I32 * CHUNK_SIZE_I32) as usize];
        for x in posx..(posx + CHUNK_SIZE_I32) {
            for z in posz..(posz + CHUNK_SIZE_I32) {
                let index = ((z - posz) * CHUNK_SIZE_I32 + (x - posx)) as usize;
                let h = get_height(x, z, world_generator);
                heights[index] = h;
            }
        }
        heightmap.insert((*chunkx, *chunkz), heights);
    }

    heightmap
}

pub fn add_to_heightmap(
    chunkx: i32,
    chunkz: i32,
    heightmap: &mut HeightMap,
    world_generator: &WorldGenerator,
) {
    if heightmap.contains_key(&(chunkx, chunkz)) {
        return;
    }
    let posx = chunkx * CHUNK_SIZE_I32;
    let posz = chunkz * CHUNK_SIZE_I32;
    let mut heights = vec![0; (CHUNK_SIZE_I32 * CHUNK_SIZE_I32) as usize];
    for x in posx..(posx + CHUNK_SIZE_I32) {
        for z in posz..(posz + CHUNK_SIZE_I32) {
            let index = ((z - posz) * CHUNK_SIZE_I32 + (x - posx)) as usize;
            let h = get_height(x, z, world_generator);
            heights[index] = h;
        }
    }
    heightmap.insert((chunkx, chunkz), heights);
}
