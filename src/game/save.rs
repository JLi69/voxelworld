use super::Game;
use crate::impfile;
use std::{fs::File, io::Write, path::Path};

pub const SAVE_PATH: &str = "saves/";
pub const CHUNK_PATH: &str = "chunkdata/";

pub fn create_save_dir() {
    if Path::new(SAVE_PATH).exists() {
        return;
    }

    if let Err(msg) = std::fs::create_dir_all(SAVE_PATH) {
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

    std::fs::create_dir_all(world_path).map_err(|e| e.to_string())?;
    let chunk_path = world_path.to_string() + CHUNK_PATH;
    std::fs::create_dir_all(chunk_path).map_err(|e| e.to_string())?;
    Ok(())
}

impl Game {
    fn save_player(&self) {
        //Save player
        let player_entry = self.player.to_entry();
        let player_save_path = self.world.path.clone() + "player.impfile";
        let player_entry_str = player_entry.to_impfile_string();
        let res = match File::create(player_save_path) {
            Ok(mut player_file) => {
                impfile::write_comment(&mut player_file, "This file contains saved player data");
                player_file.write_all(player_entry_str.as_bytes())
            }
            Err(msg) => Err(msg),
        };

        if let Err(msg) = res {
            eprintln!("E: Failed to save player: {msg}");
        }
    }

    fn save_camera(&self) {
        //Save camera
        let camera_entry = self.cam.to_entry();
        let camera_save_path = self.world.path.clone() + "camera.impfile";
        let camera_entry_str = camera_entry.to_impfile_string();
        let res = match File::create(camera_save_path) {
            Ok(mut camera_file) => {
                impfile::write_comment(&mut camera_file, "This file contains saved camera data");
                camera_file.write_all(camera_entry_str.as_bytes())
            }
            Err(msg) => Err(msg),
        };

        if let Err(msg) = res {
            eprintln!("E: Failed to save camera: {msg}");
        }
    }

    fn save_inventory(&self) {
        let save_path = self.world.path.clone() + "inventory.impfile";
        //Save hotbar
        let hotbar_entry = self.player.hotbar.to_entry();
        let hotbar_entry_str = hotbar_entry.to_impfile_string();
        //Save inventory
        let inventory_entry = self.player.inventory.to_entry();
        let inventory_entry_str = inventory_entry.to_impfile_string();
        let res = match File::create(save_path) {
            Ok(mut inventory_file) => {
                let save_str = hotbar_entry_str + "\n\n" + &inventory_entry_str;
                impfile::write_comment(&mut inventory_file, "This files contains inventory data");
                inventory_file.write_all(save_str.as_bytes())
            }
            Err(msg) => Err(msg),
        };

        if let Err(msg) = res {
            eprintln!("E: Failed to save hotbar: {msg}");
        }
    }

    pub fn save_game(&mut self) {
        self.save_camera();
        self.save_player();
        self.save_inventory();
        self.world.save();
    }

    pub fn save_entire_world(&self) {
        self.save_camera();
        self.save_player();
        self.save_inventory();
        self.world.save_all();
    }
}
