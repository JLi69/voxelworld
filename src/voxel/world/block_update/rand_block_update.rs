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
