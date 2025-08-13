use crate::{game::crafting::RecipeTable, voxel::World};

impl World {
    fn update_chunk_tile_data(
        &mut self,
        chunkx: i32,
        chunky: i32,
        chunkz: i32,
        dt: f32,
        recipes: &RecipeTable,
    ) {
        if let Some(chunk) = self.get_mut_chunk(chunkx, chunky, chunkz) {
            chunk.update_tile_data(dt, recipes);
        }
    }

    pub fn update_tile_data(&mut self, dt: f32, chunk_sim_dist: i32, recipes: &RecipeTable) {
        for x in (self.centerx - chunk_sim_dist)..=(self.centerx + chunk_sim_dist) {
            for y in (self.centery - chunk_sim_dist)..=(self.centery + chunk_sim_dist) {
                for z in (self.centerz - chunk_sim_dist)..=(self.centerz + chunk_sim_dist) {
                    self.update_chunk_tile_data(x, y, z, dt, recipes);
                }
            }
        }
    }
}
