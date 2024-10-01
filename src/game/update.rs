use super::player::CAMERA_OFFSET;
use super::Game;
use crate::gfx::{self, ChunkVaoTable};
use crate::voxel::{destroy_block, place_block};
use cgmath::Vector3;
use glfw::{CursorMode, Key};
use glfw::{MouseButtonLeft, MouseButtonRight};

const BUILD_COOLDOWN: f32 = 0.2;

impl Game {
    //Update player and camera
    pub fn update_player(&mut self, dt: f32, cursormode: CursorMode) {
        if cursormode == CursorMode::Disabled {
            let (dmousex, dmousey) = self.get_mouse_diff();
            //Rotate camera
            self.cam.rotate(dmousex, dmousey, 0.04);
        }

        //Set rotation of player
        self.player.rotation = self.cam.yaw;
        //Update player
        self.player.update(dt, &self.world);
        //Set position of camera
        self.cam.position = self.player.position + Vector3::new(0.0, CAMERA_OFFSET, 0.0);
        //Move player
        let w = self.get_key_state(Key::W);
        let s = self.get_key_state(Key::S);
        let a = self.get_key_state(Key::A);
        let d = self.get_key_state(Key::D);
        self.player.strafe(a, d);
        self.player.move_forward(w, s);
        //Select blocks
        self.player.select_block(self.get_key_state(Key::Num1), 1);
        self.player.select_block(self.get_key_state(Key::Num2), 2);
        self.player.select_block(self.get_key_state(Key::Num3), 4);
        self.player.select_block(self.get_key_state(Key::Num4), 5);
        self.player.select_block(self.get_key_state(Key::Num5), 6);
        self.player.select_block(self.get_key_state(Key::Num6), 7);
        self.player.select_block(self.get_key_state(Key::Num7), 8);
        self.player.select_block(self.get_key_state(Key::Num8), 9);
        self.player.select_block(self.get_key_state(Key::Num9), 10);
        self.player.select_block(self.get_key_state(Key::Num0), 11);
        //Jump
        let space = self.get_key_state(Key::Space);
        self.player.jump(space);
    }

    pub fn update_build_cooldown(&mut self, dt: f32) {
        self.build_cooldown -= dt;
        self.destroy_cooldown -= dt;
    }

    //Place and destroy blocks
    pub fn build(&mut self, chunkvaos: &mut ChunkVaoTable) {
        //Destroy blocks
        let pos = self.cam.position;
        let dir = self.cam.forward();
        if self.get_mouse_state(MouseButtonLeft).is_held() && self.destroy_cooldown <= 0.0 {
            let destroyed = destroy_block(pos, dir, &mut self.world);
            gfx::update_chunk_vaos(chunkvaos, destroyed, &self.world);
            self.destroy_cooldown = BUILD_COOLDOWN;
        }

        //Place blocks
        if self.get_mouse_state(MouseButtonRight).is_held() && self.build_cooldown <= 0.0 {
            let placed = place_block(pos, dir, &mut self.world, &self.player);
            gfx::update_chunk_vaos(chunkvaos, placed, &self.world);
            self.build_cooldown = BUILD_COOLDOWN;
        }
    }
}
