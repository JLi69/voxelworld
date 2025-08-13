use std::collections::HashSet;
use crate::{game::crafting::RecipeTable, voxel::World, gfx::ChunkTables};

impl World {
    fn update_chunk_tile_data(
        &mut self,
        chunkx: i32,
        chunky: i32,
        chunkz: i32,
        dt: f32,
        recipes: &RecipeTable,
    ) -> Vec<(i32, i32, i32)> {
        if let Some(chunk) = self.get_mut_chunk(chunkx, chunky, chunkz) {
            return chunk.update_tile_data(dt, recipes);
        }
        vec![]
    }

    pub fn update_tile_data(
        &mut self,
        dt: f32,
        chunk_sim_dist: i32,
        recipes: &RecipeTable,
        chunktables: &mut ChunkTables,
    ) {
        let mut block_updates = vec![];
        for x in (self.centerx - chunk_sim_dist)..=(self.centerx + chunk_sim_dist) {
            for y in (self.centery - chunk_sim_dist)..=(self.centery + chunk_sim_dist) {
                for z in (self.centerz - chunk_sim_dist)..=(self.centerz + chunk_sim_dist) {
                    block_updates.extend(self.update_chunk_tile_data(x, y, z, dt, recipes));
                }
            }
        }

        let mut update_mesh = HashSet::<(i32, i32, i32)>::new();
        update_mesh.extend(self.update_block_light(&block_updates));
        for (x, y, z) in update_mesh {
            chunktables.update_table(self, x, y, z);
        }
    }
}
