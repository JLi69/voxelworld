use super::{World, WorldGenType, WorldGenerator};
use crate::{
    impfile::{self, Entry},
    voxel::region::{chunkpos_to_regionpos, get_region_chunks, save::serialize_region, Region},
};
use std::collections::{HashMap, HashSet};
use std::{fs::File, io::Write};

fn gen_type_to_string(world_gen_type: WorldGenType) -> String {
    match world_gen_type {
        WorldGenType::Flat => "flat".to_string(),
        WorldGenType::DefaultGen => "default".to_string(),
    }
}

fn string_to_gen_type(s: &str) -> WorldGenType {
    if s == "flat" {
        WorldGenType::Flat
    } else {
        WorldGenType::DefaultGen
    }
}

pub fn save_region(region: &Region, world_path: &str) {
    if let Err(path) = serialize_region(world_path, region) {
        if let Err(msg) = std::fs::remove_file(&path) {
            eprintln!("Failed to remove {path}");
            eprintln!("{msg}");
        }
    }
}

impl World {
    fn save_world_metadata(&self) {
        let mut entry = Entry::new("world");
        entry.add_integer("range", self.range as i64);
        entry.add_integer("centerx", self.centerx as i64);
        entry.add_integer("centery", self.centery as i64);
        entry.add_integer("centerz", self.centerz as i64);
        entry.add_integer("seed", self.world_seed as i64);
        entry.add_integer("days_passed", self.days_passed as i64);
        entry.add_float("time", self.time);
        entry.add_string("gen_type", &gen_type_to_string(self.gen_type));

        let world_save_path = self.path.clone() + "world.impfile";
        let world_entry_str = entry.to_impfile_string();
        let res = match File::create(world_save_path) {
            Ok(mut player_file) => {
                impfile::write_comment(&mut player_file, "World metadata");
                player_file.write_all(world_entry_str.as_bytes())
            }
            Err(msg) => Err(msg),
        };

        if let Err(msg) = res {
            eprintln!("E: Failed to save player: {msg}");
        }
    }

    pub fn save(&mut self) {
        self.save_world_metadata();
        let mut regions_to_save = HashSet::new();
        for (x, y, z) in &self.to_save {
            let regionpos = chunkpos_to_regionpos(*x, *y, *z);
            regions_to_save.insert(regionpos);
        }
        self.to_save.clear();
        let savedcount = regions_to_save.len();
        for (rx, ry, rz) in regions_to_save {
            let mut region = Region::new(rx, ry, rz);
            get_region_chunks(&mut region, &self.chunks);
            get_region_chunks(&mut region, &self.chunk_cache);
            save_region(&region, &self.path);
        }
        eprintln!("Saved {savedcount} regions.");
    }

    pub fn save_all(&self) {
        self.save_world_metadata();
        let mut chunks_to_save = HashMap::new();
        let mut regions = HashSet::new();
        for (pos, chunk) in &self.chunk_cache {
            chunks_to_save.insert(*pos, chunk.clone());
            let (x, y, z) = pos;
            let regionpos = chunkpos_to_regionpos(*x, *y, *z);
            regions.insert(regionpos);
        }
        for (pos, chunk) in &self.chunks {
            chunks_to_save.insert(*pos, chunk.clone());
            let (x, y, z) = pos;
            let regionpos = chunkpos_to_regionpos(*x, *y, *z);
            regions.insert(regionpos);
        }

        assert_eq!(
            self.chunks.len() + self.chunk_cache.len(),
            chunks_to_save.len()
        );
        let count = regions.len();
        for (x, y, z) in regions {
            let mut region = Region::new(x, y, z);
            get_region_chunks(&mut region, &chunks_to_save);
            save_region(&region, &self.path);
        }
        eprintln!("Saved {count} regions.");
    }

    pub fn load_world_metadata(world_dir_path: &str) -> Self {
        let path = world_dir_path.to_string() + "world.impfile";
        let world_metadata_entries = impfile::parse_file(&path);
        if world_metadata_entries.is_empty() {
            return Self::empty();
        }

        let rand_seed = fastrand::u32(..);
        let seed = world_metadata_entries[0]
            .get_var("seed")
            .parse::<u32>()
            .unwrap_or(rand_seed);

        Self {
            chunks: HashMap::new(),
            skylightmap: HashMap::new(),
            range: world_metadata_entries[0]
                .get_var("range")
                .parse::<i32>()
                .unwrap_or(7)
                .max(2),
            centerx: world_metadata_entries[0]
                .get_var("centerx")
                .parse::<i32>()
                .unwrap_or(0),
            centery: world_metadata_entries[0]
                .get_var("centery")
                .parse::<i32>()
                .unwrap_or(0),
            centerz: world_metadata_entries[0]
                .get_var("centerz")
                .parse::<i32>()
                .unwrap_or(0),
            chunk_cache: HashMap::new(),
            world_generator: WorldGenerator::new(seed),
            world_seed: seed,
            gen_type: string_to_gen_type(&world_metadata_entries[0].get_var("gen_type")),
            path: world_dir_path.to_string(),
            block_update_timer: 0.0,
            random_update_timer: 0.0,
            updating: HashSet::new(),
            in_update_range: HashSet::new(),
            ticks: 0,
            time: world_metadata_entries[0]
                .get_var("time")
                .parse::<f32>()
                .unwrap_or(0.0),
            days_passed: world_metadata_entries[0]
                .get_var("days_passed")
                .parse::<u64>()
                .unwrap_or(0),
            to_save: HashSet::new(),
            removed_from_cache: vec![],
        }
    }

    //Adds the chunks in a region to the world
    pub fn add_region(&mut self, region: Region) {
        for chunk in region.chunks.iter().flatten() {
            let chunkpos = chunk.get_chunk_pos();
            let pos = (chunkpos.x, chunkpos.y, chunkpos.z);

            if self.chunks.contains_key(&pos) || self.chunk_cache.contains_key(&pos) {
                continue;
            }

            if (chunkpos.x - self.centerx).abs() > self.range {
                self.chunk_cache.insert(pos, chunk.clone());
                continue;
            }
            if (chunkpos.y - self.centery).abs() > self.range {
                self.chunk_cache.insert(pos, chunk.clone());
                continue;
            }
            if (chunkpos.z - self.centerz).abs() > self.range {
                self.chunk_cache.insert(pos, chunk.clone());
                continue;
            }
            self.chunks.insert(pos, chunk.clone());
        }
    }

    pub fn load_chunks(&mut self) {
        let mut loaded = HashSet::new();
        for y in (self.centery - self.range)..=(self.centery + self.range) {
            for z in (self.centerz - self.range)..=(self.centerz + self.range) {
                for x in (self.centerx - self.range)..=(self.centerx + self.range) {
                    if self.chunks.contains_key(&(x, y, z)) {
                        continue;
                    }

                    let (rx, ry, rz) = chunkpos_to_regionpos(x, y, z);
                    if loaded.contains(&(rx, ry, rz)) {
                        continue;
                    }
                    loaded.insert((rx, ry, rz));

                    if let Some(region) = Region::load_region(&self.path, rx, ry, rz) {
                        self.add_region(region);
                    }
                }
            }
        }

        match self.gen_type {
            WorldGenType::Flat => self.gen_flat_on_load(),
            WorldGenType::DefaultGen => self.gen_default_on_load(),
        }
    }
}
