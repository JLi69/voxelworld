use super::{inventory::Hotbar, player::Player, Camera, Game};
use crate::{impfile, voxel::World};

fn load_camera(path: &str) -> Camera {
    let camera_file_entries = impfile::parse_file(path);
    if camera_file_entries.is_empty() {
        return Camera::new(0.0, 0.0, 0.0);
    }

    Camera::from_entry(&camera_file_entries[0])
}

fn load_player(path: &str) -> Player {
    let player_file_entries = impfile::parse_file(path);
    if player_file_entries.is_empty() {
        return Player::new(0.0, 0.0, 0.0);
    }

    Player::from_entry(&player_file_entries[0])
}

fn load_inventory(path: &str) -> Hotbar {
    let inventory_file_entries = impfile::parse_file(path);
    if inventory_file_entries.is_empty() {
        return Hotbar::empty_hotbar();
    }

    Hotbar::from_entry(&inventory_file_entries[0])
}

impl Game {
    pub fn load_world(&mut self, world_path: &str) {
        let cam_path = world_path.to_string() + "camera.impfile";
        self.cam = load_camera(&cam_path);
        let player_path = world_path.to_string() + "player.impfile";
        self.player = load_player(&player_path);
        let inventory_path = world_path.to_string() + "inventory.impfile";
        self.player.hotbar = load_inventory(&inventory_path);
        self.world = World::load_world_metadata(world_path);
        self.world.load_chunks();
    }
}
