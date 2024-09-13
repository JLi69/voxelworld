pub mod camera;
pub mod input;

use crate::{
    gfx::ChunkVaoTable,
    voxel::{raycast, world_to_chunk_position, Block, World, EMPTY_BLOCK},
};
pub use camera::Camera;
use cgmath::Vector3;
use glfw::MouseButton;
pub use glfw::{Context, CursorMode, Key, PWindow};
pub use input::{release_cursor, EventHandler, KeyState};
pub use std::collections::HashMap;

const BLOCK_REACH: f32 = 4.0;

pub fn init_window(glfw: &mut glfw::Glfw) -> (PWindow, EventHandler) {
    let (mut window, events) = glfw
        .create_window(960, 640, "voxelworld", glfw::WindowMode::Windowed)
        .expect("Failed to init window!");
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_mode(CursorMode::Disabled);
    window.make_current();
    (window, events)
}

pub struct Game {
    pub cam: Camera,
    key_states: HashMap<Key, KeyState>,
    mouse_states: HashMap<MouseButton, KeyState>,
    mousex: f32, //Mouse cursor position
    mousey: f32,
    dmousex: f32, //Change in mouse position since last frame
    dmousey: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            cam: Camera::new(0.0, 0.0, 0.0),
            key_states: HashMap::new(),
            mouse_states: HashMap::new(),
            mousex: 0.0,
            mousey: 0.0,
            dmousex: 0.0,
            dmousey: 0.0,
        }
    }

    pub fn init(&mut self) {
        self.cam = Camera::new(0.0, 1.7, 0.0);
        self.mousex = 0.0;
        self.mousey = 0.0;
    }
}

pub fn destroy_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
    chunkvaos: &mut ChunkVaoTable,
) {
    let (x, y, z) = raycast(pos, dir, BLOCK_REACH, world);
    let (ix, iy, iz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let blockid = world.get_block(ix, iy, iz).id;
    world.set_block(ix, iy, iz, Block::new_id(0));
    if blockid != EMPTY_BLOCK {
        chunkvaos.update_chunk_with_adj(ix, iy, iz, world);
    }
}

pub fn place_block(
    pos: Vector3<f32>,
    dir: Vector3<f32>,
    world: &mut World,
    chunkvaos: &mut ChunkVaoTable,
) {
    //TODO: add code to check for collision with player to make sure we
    //don't place blocks on top of the player
    let (mut x, mut y, mut z) = raycast(pos, dir, BLOCK_REACH, world);
    let blockid1 = {
        let (ix, iy, iz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
        world.get_block(ix, iy, iz).id
    };
    x -= 0.2 * dir.x;
    y -= 0.2 * dir.y;
    z -= 0.2 * dir.z;
    let (ix, iy, iz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let blockid2 = world.get_block(ix, iy, iz).id;
    if blockid2 == EMPTY_BLOCK && blockid1 != EMPTY_BLOCK {
        world.set_block(ix, iy, iz, Block::new_id(1));
        chunkvaos.update_chunk_with_adj(ix, iy, iz, world);
    }
}
