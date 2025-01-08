use std::collections::HashMap;
use crate::voxel::CHUNK_SIZE_I32;
use noise::{Perlin, NoiseFn, Fbm};

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
    cave_noise.get(xyz) < cave_perc(y, -64, -51, 48)
}

pub fn get_height(x: i32, z: i32, terrain_generator: &Fbm<Perlin>) -> i32 {
    let point = [x as f64 / 192.0, z as f64 / 192.0];
    let noise_height = terrain_generator.get(point);
    (noise_height * 47.0) as i32 + 16
}

pub fn generate_heightmap(
    positions: &Vec<(i32, i32, i32)>,
    terrain_generator: &Fbm<Perlin>,
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
                let h = get_height(x, z, terrain_generator);
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
            let h = get_height(x, z, terrain_generator);
            heights[index] = h;
        }
    }
    heightmap.insert((chunkx, chunkz), heights);
}
