pub mod camera;
pub mod input;
pub mod update;
pub mod player;
pub mod physics;

use physics::Hitbox;
use player::Player;
use crate::World;
pub use camera::Camera;
use cgmath::{Matrix4, SquareMatrix};
use glfw::MouseButton;
pub use glfw::{Context, CursorMode, Key, PWindow};
pub use input::{release_cursor, EventHandler, KeyState};
pub use std::collections::HashMap;

//Initialize window, call this at the beginning of the game
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

//Game state struct
pub struct Game {
    pub cam: Camera,
    pub player: Player,
    key_states: HashMap<Key, KeyState>,
    mouse_states: HashMap<MouseButton, KeyState>,
    mousex: f32, //Mouse cursor position
    mousey: f32,
    dmousex: f32, //Change in mouse position since last frame
    dmousey: f32,
    pub world: World,
    pub persp: Matrix4<f32>,
}

impl Game {
    //Create game state
    pub fn new() -> Self {
        Self {
            cam: Camera::new(0.0, 0.0, 0.0),
            player: Player::new(0.0, 0.0, 0.0),
            key_states: HashMap::new(),
            mouse_states: HashMap::new(),
            mousex: 0.0,
            mousey: 0.0,
            dmousex: 0.0,
            dmousey: 0.0,
            world: World::empty(),
            persp: Matrix4::identity(),
        }
    }

    //Initialize game state
    pub fn init(&mut self) {
        self.cam = Camera::new(0.0, 1.7, 0.0);
        self.player = Player::new(0.0, 0.9, 0.0); 
        self.mousex = 0.0;
        self.mousey = 0.0;
    }

    //Generate world
    pub fn generate_world(&mut self, range: usize) {
        self.world = World::new(range);
        self.world.gen_flat();
    }
}
