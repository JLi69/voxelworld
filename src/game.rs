pub mod assets;
pub mod camera;
pub mod gameloop;
pub mod input;
pub mod inventory;
pub mod load;
pub mod physics;
pub mod player;
pub mod save;
pub mod update;

use crate::impfile;
use crate::voxel::world::WorldGenType;
use crate::{assets::texture::load_image_pixels, game::player::PLAYER_HEIGHT, World};
use assets::models::ModelManager;
use assets::shaders::ShaderManager;
use assets::textures::TextureManager;
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

//Application config values, these are not meant to be changed by normal users
struct Config {
    font_path: String,
}

impl Config {
    pub fn default() -> Self {
        Self {
            font_path: String::new(),
        }
    }
}

//Initialize window, call this at the beginning of the game
pub fn init_window(glfw: &mut glfw::Glfw) -> (PWindow, EventHandler) {
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    let (mut window, events) = glfw
        .create_window(960, 640, "voxelworld", glfw::WindowMode::Windowed)
        .expect("Failed to init window!");
    window.set_key_polling(true);
    window.set_scroll_polling(true);
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
    //Game objects
    pub cam: Camera,
    pub player: Player,
    //World
    pub world: World,
    //Input state
    paused: bool,
    key_states: HashMap<Key, KeyState>,
    mouse_states: HashMap<MouseButton, KeyState>,
    scroll_state: f32,
    mousex: f32, //Mouse cursor position
    mousey: f32,
    dmousex: f32, //Change in mouse position since last frame
    dmousey: f32,
    build_cooldown: f32,
    destroy_cooldown: f32,
    //Perspective matrix and aspect
    pub persp: Matrix4<f32>,
    pub aspect: f32,
    //Manage fonts, textures, models, and shaders
    pub fonts: FontDefinitions,
    pub models: ModelManager,
    pub shaders: ShaderManager,
    pub textures: TextureManager,
    //Config
    cfg: Config,
    //Debug info
    display_debug: bool,
}

impl Game {
    //Create game state
    pub fn new() -> Self {
        Self {
            paused: false,
            cam: Camera::new(0.0, 0.0, 0.0),
            player: Player::new(0.0, 0.0, 0.0),
            key_states: HashMap::new(),
            mouse_states: HashMap::new(),
            scroll_state: 0.0,
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
            models: ModelManager::new(),
            shaders: ShaderManager::new(),
            textures: TextureManager::new(),
            cfg: Config::default(),
            display_debug: false,
        }
    }

    pub fn reset(&mut self) {
        self.cam = Camera::new(0.0, 1.7, 0.0);
        self.player = Player::new(15.5, 0.0, 15.5);
        self.build_cooldown = 0.0;
        self.destroy_cooldown = 0.0;
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
        eprintln!("Created world with seed: {}", self.world.get_seed());
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

    pub fn load_config(&mut self, path: &str) {
        let entries = impfile::parse_file(path);
        if entries.is_empty() {
            eprintln!("Error: empty config file");
            return;
        }
        let e = &entries[0];
        self.cfg.font_path = e.get_var("font_path");
    }
}
