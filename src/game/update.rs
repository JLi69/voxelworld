use super::player::CAMERA_OFFSET;
use super::{Game, KeyState};
use crate::gfx::{self, ChunkTables};
use crate::voxel::{destroy_block, place_block};
use cgmath::Vector3;
use glfw::{CursorMode, Key};
use glfw::{MouseButtonLeft, MouseButtonRight};

const BUILD_COOLDOWN: f32 = 0.15;

impl Game {
    //Update player and camera
    pub fn update_player(&mut self, dt: f32, cursormode: CursorMode) {
        if cursormode == CursorMode::Disabled {
            let (dmousex, dmousey) = self.get_mouse_diff();
            //Rotate camera
            self.cam.rotate(dmousex, dmousey, 0.06);
        }

        //Set rotation of player
        self.player.rotation = self.cam.yaw;
        //Update player
        self.player.update(dt, &self.world);
        //Set position of camera
        self.cam.position = self.player.position + Vector3::new(0.0, CAMERA_OFFSET, 0.0);
        //Move player
        let shift = self.get_key_state(Key::LeftShift);
        self.player.sprint(shift);
        let w = self.get_key_state(Key::W);
        let s = self.get_key_state(Key::S);
        let a = self.get_key_state(Key::A);
        let d = self.get_key_state(Key::D);
        self.player.strafe(a, d);
        self.player.move_forward(w, s);
        let space = self.get_key_state(Key::Space);
        //Jump
        self.player.jump(space);
        //Swim
        self.player.swim(space, &self.world);
        //Select items from the hotbar
        self.player.select_hotbar_item(self.get_key_state(Key::Num1), 0);
        self.player.select_hotbar_item(self.get_key_state(Key::Num2), 1);
        self.player.select_hotbar_item(self.get_key_state(Key::Num3), 2);
        self.player.select_hotbar_item(self.get_key_state(Key::Num4), 3);
        self.player.select_hotbar_item(self.get_key_state(Key::Num5), 4);
        self.player.select_hotbar_item(self.get_key_state(Key::Num6), 5);
        self.player.select_hotbar_item(self.get_key_state(Key::Num7), 6);
        self.player.select_hotbar_item(self.get_key_state(Key::Num8), 7);
        self.player.select_hotbar_item(self.get_key_state(Key::Num9), 8);
    }

    pub fn update_build_cooldown(&mut self, dt: f32) {
        self.build_cooldown -= dt;
        self.destroy_cooldown -= dt;
    }

    //Place and destroy blocks
    pub fn build(&mut self, chunktables: &mut ChunkTables) {
        //Destroy blocks
        let pos = self.cam.position;
        let dir = self.cam.forward();
        if !self.get_mouse_state(MouseButtonLeft).is_held() {
            self.destroy_cooldown = 0.0;
        }

        if self.get_mouse_state(MouseButtonLeft).is_held() && self.destroy_cooldown <= 0.0 {
            let destroyed = destroy_block(pos, dir, &mut self.world);
            gfx::update_chunk_vaos(chunktables, destroyed, &self.world);
            if destroyed.is_some() {
                self.destroy_cooldown = BUILD_COOLDOWN;
            } else {
                self.destroy_cooldown = 0.0;
            }
        }

        //Place blocks
        if !self.get_mouse_state(MouseButtonRight).is_held() {
            self.build_cooldown = 0.0;
        }

        if self.get_mouse_state(MouseButtonRight).is_held() && self.build_cooldown <= 0.0 {
            let placed = place_block(pos, dir, &mut self.world, &self.player);
            gfx::update_chunk_vaos(chunktables, placed, &self.world);
            if placed.is_some() {
                self.build_cooldown = BUILD_COOLDOWN;
            } else {
                self.build_cooldown = 0.0;
            }
        }
    }

    //Handle pausing
    pub fn pause(&mut self) {
        if self.get_key_state(Key::Escape) == KeyState::JustPressed {
            self.paused = !self.paused;
        }
    }
}
