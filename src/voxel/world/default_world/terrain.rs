use crate::voxel::CHUNK_SIZE_I32;
use noise::{Fbm, NoiseFn, Perlin};
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

pub fn get_height(x: i32, z: i32, terrain_generator: &Fbm<Perlin>, steepness: &Perlin) -> i32 {
    let point = [x as f64 / 192.0, z as f64 / 192.0];
    let noise_height = terrain_generator.get(point);
    let steepness_point = [x as f64 / 24.0, z as f64 / 24.0];
    let steepness = (steepness.get(steepness_point) + 1.0) / 2.0;
    let transformed_noise = if noise_height > 0.0 {
        let v = steepness.powf(1.0 - steepness.sqrt());
        noise_height.powf(1.0 - noise_height.powf(0.25 * v)) * v * v
    } else {
        noise_height * 0.8 * steepness.powf(1.0 - noise_height.abs())
    };
    (transformed_noise * 64.0) as i32 + 12
}

pub fn generate_heightmap(
    positions: &Vec<(i32, i32, i32)>,
    terrain_generator: &Fbm<Perlin>,
    steepness: &Perlin,
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
                let h = get_height(x, z, terrain_generator, steepness);
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
    terrain_generator: &Fbm<Perlin>,
    steepness: &Perlin,
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
            let h = get_height(x, z, terrain_generator, steepness);
            heights[index] = h;
        }
    }
    heightmap.insert((chunkx, chunkz), heights);
}
