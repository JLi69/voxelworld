pub mod camera;
pub mod input;

pub use camera::Camera;
pub use glfw::{Context, CursorMode, Key, PWindow};
pub use input::{release_cursor, EventHandler, KeyState};
pub use std::collections::HashMap;

pub fn init_window(glfw: &mut glfw::Glfw) -> (PWindow, EventHandler) {
    let (mut window, events) = glfw
        .create_window(960, 640, "voxelworld", glfw::WindowMode::Windowed)
        .expect("Failed to init window!");
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(CursorMode::Disabled);
    window.make_current();
    (window, events)
}

pub struct Game {
    pub cam: Camera,
    key_states: HashMap<Key, KeyState>,
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
