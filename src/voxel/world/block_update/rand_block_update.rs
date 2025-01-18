use super::{get_chunktable_updates, UpdateList, World, CHUNK_SIZE_I32};
use crate::{
    gfx::ChunkTables,
    voxel::{Block, EMPTY_BLOCK},
};
use std::collections::HashSet;

pub const RANDOM_UPDATE_INTERVAL: f32 = 0.25;

fn grow_wheat(world: &World, x: i32, y: i32, z: i32, id: u8, to_update: &mut UpdateList) {
    let below = world.get_block(x, y - 1, z);
    let skip_dry_farmland = below.id == 45 && fastrand::i32(0..4) != 0;
    let skip_wet_farmland = below.id == 43 && fastrand::i32(0..2) != 0;
    if skip_wet_farmland || skip_dry_farmland {
        return;
    }
    to_update.insert((x, y, z), Block::new_id(id + 1));
}

fn update_grass(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let above = world.get_block(x, y + 1, z);
    if (above.transparent() || above.id == EMPTY_BLOCK) && !above.is_fluid() {
        return;
    }
    if above.id == 12 && above.geometry < 7 {
        return;
    }
    //If a non-trasparent block is above the block, then have the grass die
    to_update.insert((x, y, z), Block::new_id(4));
}

fn update_dirt(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let above = world.get_block(x, y + 1, z);
    if !(above.transparent() || above.id == EMPTY_BLOCK) || above.is_fluid() {
        return;
    }
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }

                let above = world.get_block(x + dx, y + dy + 1, z + dz);
                if !(above.transparent() || above.id == EMPTY_BLOCK) || above.is_fluid() {
                    continue;
                }
                let block = world.get_block(x + dx, y + dy, z + dz);

                if block.id == 1 && fastrand::bool() {
                    to_update.insert((x, y, z), Block::new_id(1));
                    return;
                }
            }
        }
    }
}

//Convert dry farmland that is near water into wet farmland
fn update_dry_farmland(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let above = world.get_block(x, y + 1, z);
    if above.id == 12 {
        to_update.insert((x, y, z), Block::new_id(43));
        return;
    }

    for dx in -4..=4 {
        for dz in -4..=4 {
            let block = world.get_block(x + dx, y, z + dz);
            if block.id == 12 {
                to_update.insert((x, y, z), Block::new_id(43));
                return;
            }
            let block = world.get_block(x + dx, y - 1, z + dz);
            if block.id == 12 {
                to_update.insert((x, y, z), Block::new_id(43));
                return;
            }
        }
    }
}

//Convert wet farmland that is far away from water into dry farmland
fn update_wet_farmland(world: &World, x: i32, y: i32, z: i32, to_update: &mut UpdateList) {
    let above = world.get_block(x, y + 1, z);
    if above.id == 12 {
        return;
    }

    for dx in -4..=4 {
        for dz in -4..=4 {
            let block = world.get_block(x + dx, y, z + dz);
            if block.id == 12 {
                return;
            }
            let block = world.get_block(x + dx, y - 1, z + dz);
            if block.id == 12 {
                return;
            }
        }
    }

    to_update.insert((x, y, z), Block::new_id(45));
}

impl World {
    fn rand_block_chunk_update(
        &self,
        chunkx: i32,
        chunky: i32,
        chunkz: i32,
        to_update: &mut UpdateList,
    ) {
        if let Some(chunk) = self.chunks.get(&(chunkx, chunky, chunkz)) {
            if chunk.is_empty() {
                return;
            }
        }

        let startx = chunkx * CHUNK_SIZE_I32;
        let starty = chunky * CHUNK_SIZE_I32;
        let startz = chunkz * CHUNK_SIZE_I32;
        let block_count = fastrand::i32(80..96);
        (0..block_count)
            .map(|_| fastrand::i32(0..CHUNK_SIZE_I32.pow(3)))
            .map(|index| {
                let x = index % CHUNK_SIZE_I32;
                let y = (index / CHUNK_SIZE_I32) % CHUNK_SIZE_I32;
                let z = index / (CHUNK_SIZE_I32 * CHUNK_SIZE_I32);
                (startx + x, starty + y, startz + z)
            })
            .for_each(|(x, y, z)| {
                let block = self.get_block(x, y, z);
                match block.id {
                    //Grass
                    1 => update_grass(self, x, y, z, to_update),
                    //Dirt
                    4 => update_dirt(self, x, y, z, to_update),
                    //Wet farmland
                    43 => update_wet_farmland(self, x, y, z, to_update),
                    //Dry farmland
                    45 => update_dry_farmland(self, x, y, z, to_update),
                    //Growing wheat
                    50..=52 => grow_wheat(self, x, y, z, block.id, to_update),
                    _ => {}
                }
            });
    }

    //If chunktables is None, we do not update any chunktable
    pub fn rand_block_update(
        &mut self,
        dt: f32,
        chunktables: Option<&mut ChunkTables>,
        chunk_sim_dist: i32,
    ) {
        self.random_update_timer -= dt;
        if self.random_update_timer > 0.0 {
            return;
        }
        self.random_update_timer = RANDOM_UPDATE_INTERVAL;

        let mut to_update = UpdateList::new();
        for x in (self.centerx - chunk_sim_dist)..=(self.centerx + chunk_sim_dist) {
            for y in (self.centery - chunk_sim_dist)..=(self.centery + chunk_sim_dist) {
                for z in (self.centerz - chunk_sim_dist)..=(self.centerz + chunk_sim_dist) {
                    self.rand_block_chunk_update(x, y, z, &mut to_update);
                }
            }
        }

        let mut update_mesh = HashSet::<(i32, i32, i32)>::new();
        for ((x, y, z), block) in to_update {
            if self.get_block(x, y, z) == block {
                continue;
            }

            self.set_block(x, y, z, block);
            get_chunktable_updates(x, y, z, &mut update_mesh);
        }

        if let Some(chunktables) = chunktables {
            for (x, y, z) in update_mesh {
                chunktables.update_table(self, x, y, z);
            }
        }
    }
}
