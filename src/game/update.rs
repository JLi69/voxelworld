use super::{Game, KeyState};
use crate::gfx::{self, ChunkVaoTable};
use crate::voxel::{destroy_block, place_block};
use glfw::{CursorMode, Key};
use glfw::{MouseButtonLeft, MouseButtonRight};

impl Game {
    //Update camera
    pub fn update_camera(&mut self, dt: f32, cursormode: CursorMode) {
        if cursormode == CursorMode::Disabled {
            let (dmousex, dmousey) = self.get_mouse_diff();
            self.cam.rotate(dmousex, dmousey, 0.04);
        }

        self.cam.update(dt);
        //Move camera
        let w = self.get_key_state(Key::W);
        let s = self.get_key_state(Key::S);
        let a = self.get_key_state(Key::A);
        let d = self.get_key_state(Key::D);
        let shift = self.get_key_state(Key::LeftShift);
        let space = self.get_key_state(Key::Space);
        self.cam.strafe(a, d);
        self.cam.move_forward(w, s);
        self.cam.fly(shift, space);
    }

    //Place and destroy blocks
    pub fn build(&mut self, chunkvaos: &mut ChunkVaoTable) {
        //Destroy blocks
        let pos = self.cam.position();
        let dir = self.cam.forward();
        if self.get_mouse_state(MouseButtonLeft) == KeyState::JustPressed {
            let destroyed = destroy_block(pos, dir, &mut self.world);
            gfx::update_chunk_vaos(chunkvaos, destroyed, &self.world);
        }
        //Place blocks
        if self.get_mouse_state(MouseButtonRight) == KeyState::JustPressed {
            let placed = place_block(pos, dir, &mut self.world);
            gfx::update_chunk_vaos(chunkvaos, placed, &self.world);
        }
    }
}
