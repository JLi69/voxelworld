use super::{Game, KeyState};
use crate::gfx::{self, ChunkVaoTable};
use crate::voxel::{destroy_block, place_block};
use super::player::CAMERA_OFFSET;
use cgmath::Vector3;
use glfw::{CursorMode, Key};
use glfw::{MouseButtonLeft, MouseButtonRight};

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
        //Jump
        let space = self.get_key_state(Key::Space);
        self.player.jump(space);
    }

    //Place and destroy blocks
    pub fn build(&mut self, chunkvaos: &mut ChunkVaoTable) {
        //Destroy blocks
        let pos = self.cam.position;
        let dir = self.cam.forward();
        if self.get_mouse_state(MouseButtonLeft) == KeyState::JustPressed {
            let destroyed = destroy_block(pos, dir, &mut self.world);
            gfx::update_chunk_vaos(chunkvaos, destroyed, &self.world);
        }
        //Place blocks
        if self.get_mouse_state(MouseButtonRight) == KeyState::JustPressed {
            let placed = place_block(pos, dir, &mut self.world, &self.player);
            gfx::update_chunk_vaos(chunkvaos, placed, &self.world);
        }
    }
}
