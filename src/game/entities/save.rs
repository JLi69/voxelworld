use super::{
    region::{get_region_entities, serialize_entities, EntityRegion},
    EntitiesTable,
};
use crate::voxel::{region::chunkpos_to_regionpos, World, world::{in_sim_range, get_simulation_dist}};
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
            get_region_entities(&mut region, self, world);
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

        for (x, y, z) in world.chunks.keys().copied() {
            let regionpos = chunkpos_to_regionpos(x, y, z);
            regions_to_save.insert(regionpos);
        }

        for (x, y, z) in world.chunk_cache.keys().copied() {
            let regionpos = chunkpos_to_regionpos(x, y, z);
            regions_to_save.insert(regionpos);
        }

        for (rx, ry, rz) in regions_to_save {
            let mut region = EntityRegion::new(rx, ry, rz);
            get_region_entities(&mut region, self, world);
            save_entity_region(&world.path, region);
        }

        eprintln!("Saved entities.");
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
                for dropped_item in &region.dropped_items {
                    self.dropped_items.add_item(dropped_item.clone());
                }
            }
        }
    }
}
