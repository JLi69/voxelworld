use std::{path::Path, fs::File, io::Write};
use crate::impfile;
use super::Game;

const SAVE_PATH: &str = "saves/";

pub fn create_save_dir() {
    if Path::new(SAVE_PATH).exists() {
        return;
    }

    if let Err(msg) = std::fs::create_dir(SAVE_PATH) {
        eprintln!("{msg}");
        panic!("Failed to create save directory, killing program.");
    }
}

pub fn get_world_path(world_name: &str) -> String {
    let mut path = SAVE_PATH.to_string() + world_name.to_lowercase().as_str() + "/";

    let mut id = 1i32;
    while Path::new(&path).exists() {
        path = SAVE_PATH.to_string()
            + world_name.to_lowercase().as_str()
            + "_"
            + id.to_string().as_str()
            + "/";
        id += 1;
    }

    path
}

pub fn create_world_dir(world_path: &str) -> Result<(), String> {
    if Path::new(world_path).exists() {
        return Err("Path exists".to_string());
    }

    std::fs::create_dir(world_path).map_err(|e| e.to_string())?;
    let chunk_path = world_path.to_string() + "chunkdata/";
    std::fs::create_dir(chunk_path).map_err(|e| e.to_string())?;
    Ok(())
}

impl Game {
    pub fn save_game(&self) {
        //Save player
        let player_entry = self.player.to_entry();
        let player_save_path = self.world.path.clone() + "player.impfile"; 
        let player_entry_str = player_entry.to_impfile_string();
        let res = match File::create(player_save_path) {
            Ok(mut player_file) => {
                impfile::write_comment(
                    &mut player_file, 
                    "This file contains saved player data",
                );
                player_file.write_all(player_entry_str.as_bytes())
            }
            Err(msg) => {
                Err(msg) 
            }
        };

        if let Err(msg) = res {
            eprintln!("E: Failed to save player: {msg}");
        }

        //Save world
        self.world.save();
    }
}
