use super::{
    region::{get_region_entities, serialize_entities, EntityRegion},
    EntitiesTable,
};
use crate::voxel::{
    region::{chunkpos_to_regionpos, regionpos_to_chunkpos, REGION_SIZE_I32},
    world::{get_simulation_dist, in_sim_range},
    World,
};
use std::collections::HashSet;

fn save_entity_region(worldpath: &str, region: EntityRegion) {
    if let Err(path) = serialize_entities(worldpath, region) {
        if let Err(msg) = std::fs::remove_file(&path) {
            eprintln!("Failed to remove {path}");
            eprintln!("{msg}");
        }
    }
}

impl EntitiesTable {
    //Save only entities in loaded chunks
    pub fn save(&self, world: &World) {
        let mut regions_to_save = HashSet::new();
        for (x, y, z) in world.chunks.keys().copied() {
            let regionpos = chunkpos_to_regionpos(x, y, z);
            regions_to_save.insert(regionpos);
        }

        for (rx, ry, rz) in regions_to_save {
            let mut region = EntityRegion::new(rx, ry, rz);
            get_region_entities(&mut region, self);
            save_entity_region(&world.path, region);
        }
    }

    //Save everything that is loaded
    pub fn save_all(&self, world: &World) {
        let mut regions_to_save = HashSet::new();
        for (x, y, z) in self.dropped_items.items().keys().copied() {
            let regionpos = chunkpos_to_regionpos(x, y, z);
            regions_to_save.insert(regionpos);
        }

        for (rx, ry, rz) in regions_to_save {
            let mut region = EntityRegion::new(rx, ry, rz);
            get_region_entities(&mut region, self);
            save_entity_region(&world.path, region);
        }

        eprintln!("Saved entities.");
    }

    pub fn deload(&mut self, world: &World) {
        let mut to_deload = vec![];
        let mut regions_to_save = HashSet::new();

        for pos in self.dropped_items.items().keys().copied() {
            if world.is_loaded(pos) {
                continue;
            }
            to_deload.push(pos);
            let (x, y, z) = pos;
            regions_to_save.insert(chunkpos_to_regionpos(x, y, z));
        }

        for (rx, ry, rz) in regions_to_save {
            let mut region = EntityRegion::new(rx, ry, rz);
            get_region_entities(&mut region, self);
            save_entity_region(&world.path, region);
        }

        for pos in to_deload {
            self.dropped_items.remove(pos);
        }
    }

    pub fn add_region(&mut self, region: EntityRegion) {
        for dropped_item in &region.dropped_items {
            self.dropped_items.add_item(dropped_item.clone());
        }

        let (startx, starty, startz) = regionpos_to_chunkpos(region.x, region.y, region.z);
        for x in startx..(startx + REGION_SIZE_I32) {
            for y in starty..(starty + REGION_SIZE_I32) {
                for z in startz..(startz + REGION_SIZE_I32) {
                    self.dropped_items.add_empty(x, y, z);
                }
            }
        }
    }

    //Returns true if something was loaded
    pub fn load_chunk(&mut self, world: &World, x: i32, y: i32, z: i32) -> bool {
        if self.dropped_items.items().contains_key(&(x, y, z)) {
            return false;
        }

        if let Some(region) = EntityRegion::load_region(&world.path, x, y, z) {
            self.add_region(region);
            return true;
        }
        false
    }

    pub fn load_from_list(&mut self, world: &World, to_load: HashSet<(i32, i32, i32)>) {
        let mut loaded_count = 0;
        for (x, y, z) in to_load {
            if self.load_chunk(world, x, y, z) {
                loaded_count += 1;
            }
        }

        if loaded_count > 0 {
            eprintln!("[entities] Loaded {loaded_count} regions");
        }
    }

    //Load all entities in the 3 x 3 column surrounding the player
    pub fn force_load(&mut self, world: &World) {
        let (centerx, centery, centerz) = world.get_center();

        let bot = centery - world.get_range();
        let top = centery + world.get_range();

        for x in (centerx - 1)..=(centerx + 1) {
            for z in (centerz - 1)..=(centerz + 1) {
                for y in bot..=top {
                    self.load_chunk(world, x, y, z);
                }
            }
        }
    }

    pub fn load(&mut self, world: &World) {
        let center = world.get_center();
        let sim_dist = get_simulation_dist(world);
        let mut to_load = HashSet::new();
        for pos in world.chunks.keys().copied() {
            if !in_sim_range(center, pos, sim_dist) {
                continue;
            }
            let (x, y, z) = pos;
            let region_pos = chunkpos_to_regionpos(x, y, z);
            to_load.insert(region_pos);
        }

        for (x, y, z) in to_load {
            if let Some(region) = EntityRegion::load_region(&world.path, x, y, z) {
                self.add_region(region);
            }
        }
    }
}
