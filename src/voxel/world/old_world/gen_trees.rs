use super::terrain::{get_height, is_noise_cave};
use super::{WorldGenerator, SAND_LEVEL};
use crate::voxel::{Block, Chunk, CHUNK_SIZE_I32, EMPTY_BLOCK};
use noise::{NoiseFn, Perlin};
use std::collections::HashSet;

fn gen_tree_positions(
    chunkx: i32,
    chunkz: i32,
    tree_noise: &Perlin,
    positions: &mut Vec<(i32, i32)>,
    heights: &mut Vec<i32>,
    world_seed: u32,
) {
    let xz = [chunkx as f64 / 3.0 + 0.5, 0.0, chunkz as f64 / 3.0 + 0.5];
    let noise_val = (tree_noise.get(xz) + 1.0) / 2.0;
    let tree_count = (noise_val * noise_val * 6.0).floor() as u32;
    let xu32 = chunkx as u32;
    let zu32 = chunkz as u32;
    let seed = ((xu32 as u64) << 32) | (zu32 as u64);
    let mut tree_generator = fastrand::Rng::with_seed(seed);
    let mut rng = fastrand::Rng::with_seed(seed + world_seed as u64);
    let mut generated = HashSet::<(i32, i32)>::new();
    for _ in 0..tree_count {
        let treex =
            (tree_generator.i32(0..CHUNK_SIZE_I32) + rng.i32(0..CHUNK_SIZE_I32)) % CHUNK_SIZE_I32;
        let treez =
            (tree_generator.i32(0..CHUNK_SIZE_I32) + rng.i32(0..CHUNK_SIZE_I32)) % CHUNK_SIZE_I32;
        if generated.contains(&(treex, treez)) {
            continue;
        }
        let x = treex + chunkx * CHUNK_SIZE_I32;
        let z = treez + chunkz * CHUNK_SIZE_I32;
        let h = tree_generator.i32(4..=6);
        positions.push((x, z));
        heights.push(h);
        generated.insert((treex, treez));
    }
}

fn place_leaves(chunk: &mut Chunk, x: i32, y: i32, z: i32) {
    if chunk.get_block(x, y, z).id != EMPTY_BLOCK {
        return;
    }
    chunk.set_block(x, y, z, Block::new_id(7));
}

fn generate_leaves(chunk: &mut Chunk, starty: i32, x: i32, y: i32, z: i32, height: i32) {
    if y == starty + height {
        place_leaves(chunk, x, y, z);
        place_leaves(chunk, x - 1, y, z);
        place_leaves(chunk, x + 1, y, z);
        place_leaves(chunk, x, y, z - 1);
        place_leaves(chunk, x, y, z + 1);
    } else if y == starty + height - 1 {
        for ix in (x - 1)..=(x + 1) {
            for iz in (z - 1)..=(z + 1) {
                place_leaves(chunk, ix, y, iz);
            }
        }
    } else if y >= starty + height - 3 {
        for ix in (x - 2)..=(x + 2) {
            for iz in (z - 2)..=(z + 2) {
                place_leaves(chunk, ix, y, iz);
            }
        }
    }
}

pub fn get_tree_gen_info(
    x: i32,
    z: i32,
    world_generator: &WorldGenerator,
) -> (Vec<(i32, i32)>, Vec<i32>) {
    let mut tree_positions = vec![];
    let mut tree_heights = vec![];
    for dx in -1..=1 {
        for dz in -1..=1 {
            gen_tree_positions(
                x + dx,
                z + dz,
                &world_generator.tree_generator,
                &mut tree_positions,
                &mut tree_heights,
                world_generator.world_seed,
            );
        }
    }

    (tree_positions, tree_heights)
}

pub fn generate_trees(
    chunk: &mut Chunk,
    tree_positions: &[(i32, i32)],
    tree_heights: &[i32],
    world_generator: &WorldGenerator,
) {
    let chunkpos = chunk.get_chunk_pos();
    let lower_x = chunkpos.x * CHUNK_SIZE_I32;
    let upper_x = chunkpos.x * CHUNK_SIZE_I32 + CHUNK_SIZE_I32 - 1;
    let lower_z = chunkpos.z * CHUNK_SIZE_I32;
    let upper_z = chunkpos.z * CHUNK_SIZE_I32 + CHUNK_SIZE_I32 - 1;

    for (i, (x, z)) in tree_positions.iter().enumerate() {
        if x - lower_x < -2 || x - upper_x > 2 || z - lower_z < -2 || z - upper_z > 2 {
            continue;
        }

        let h = get_height(
            *x,
            *z,
            &world_generator.terrain_generator,
            &world_generator.steepness,
        );

        //Below sea level
        if h <= SAND_LEVEL {
            continue;
        }

        //Check to make sure we are not in a cave (an empty block)
        if is_noise_cave(*x, h, *z, &world_generator.noise_cave_generator) {
            continue;
        }

        for y in (h + 1)..(h + 1 + tree_heights[i]) {
            //Generate trunk
            chunk.set_block(*x, y, *z, Block::new_id(8));

            //Generate leaves
            generate_leaves(chunk, h + 1, *x, y, *z, tree_heights[i]);
        }
        generate_leaves(
            chunk,
            h + 1,
            *x,
            h + 1 + tree_heights[i],
            *z,
            tree_heights[i],
        );
    }
}
