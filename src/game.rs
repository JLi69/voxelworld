pub mod assets;
pub mod camera;
pub mod gameloop;
pub mod input;
pub mod physics;
pub mod player;
pub mod update;

use crate::voxel::world::WorldGenType;
use crate::World;
use crate::{assets::texture::load_image_pixels, game::player::PLAYER_HEIGHT};
pub use camera::Camera;
use cgmath::{Matrix4, SquareMatrix};
use egui_gl_glfw::egui::FontDefinitions;
pub use gameloop::run;
use glfw::MouseButton;
pub use glfw::{Context, CursorMode, Key, PWindow};
pub use input::{release_cursor, EventHandler, KeyState};
use physics::Hitbox;
use player::Player;
pub use std::collections::HashMap;

//Initialize window, call this at the beginning of the game
pub fn init_window(glfw: &mut glfw::Glfw) -> (PWindow, EventHandler) {
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    let (mut window, events) = glfw
        .create_window(960, 640, "voxelworld", glfw::WindowMode::Windowed)
        .expect("Failed to init window!");
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_char_polling(true);
    window.set_cursor_mode(CursorMode::Disabled);
    window.make_current();

    //Set icon for window
    match load_image_pixels("assets/icon.png") {
        Ok((pixel_data, info)) => {
            window.set_icon_from_pixels(vec![glfw::PixelImage {
                width: info.width,
                height: info.height,
                pixels: pixel_data,
            }]);
        }
        Err(msg) => {
            eprintln!("{msg}");
        }
    }

    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
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
    build_cooldown: f32,
    destroy_cooldown: f32,
    pub world: World,
    pub persp: Matrix4<f32>,
    pub aspect: f32,
    fonts: FontDefinitions,
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
            build_cooldown: 0.0,
            destroy_cooldown: 0.0,
            world: World::empty(),
            persp: Matrix4::identity(),
            aspect: 1.0,
            fonts: FontDefinitions::default(),
        }
    }

    //Initialize game state
    pub fn init(&mut self) {
        self.cam = Camera::new(0.0, 1.7, 0.0);
        self.player = Player::new(15.5, 0.9, 15.5);
        self.mousex = 0.0;
        self.mousey = 0.0;
    }

    //Generate world
    pub fn generate_world(&mut self, seed: u32, range: i32, gen_type: WorldGenType) {
        self.world = World::new(seed, range, gen_type);
        self.world.generate_world();

        //Set position of the player
        for ref y in (-64..=128).rev() {
            self.player.position.y = *y as f32;
            if self.player.check_collision(&self.world).is_some() {
                self.player.position.y += PLAYER_HEIGHT / 2.0;
                break;
            }
        }
    }
}
