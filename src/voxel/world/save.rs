use super::{World, WorldGenType};
use crate::{impfile::{self, Entry}, voxel::Chunk};
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

pub fn save_chunk(chunk: &Chunk, world_path: &str) {
    if let Err(path) = chunk.save_chunk(world_path) {
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

    pub fn save(&self) {
        self.save_world_metadata();
        for chunk in self.chunks.values() {
            save_chunk(chunk, &self.path);
        }
    }

    pub fn save_all(&self) { 
        self.save_world_metadata();
        for chunk in self.chunks.values() { 
            save_chunk(chunk, &self.path);
        }

        for chunk in self.chunk_cache.values() {
            save_chunk(chunk, &self.path);
        }
    }
}
