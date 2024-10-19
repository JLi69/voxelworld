use crate::{impfile, voxel::World};
use super::{Game, Camera, player::Player};

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

impl Game {
    pub fn load_world(&mut self, world_path: &str) {
        let cam_path = world_path.to_string() + "camera.impfile";
        self.cam = load_camera(&cam_path);
        let player_path = world_path.to_string() + "player.impfile";
        self.player = load_player(&player_path);
        self.world = World::load_world_metadata(world_path);
        self.world.load_chunks();
    }
}