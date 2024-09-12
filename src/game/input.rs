use crate::gfx;
use crate::Game;
use glfw::CursorMode;
use glfw::{Action, GlfwReceiver, Key, PWindow, WindowEvent};

pub type EventHandler = GlfwReceiver<(f64, glfw::WindowEvent)>;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum KeyState {
    Released,
    JustPressed,
    Held,
}

impl KeyState {
    pub fn is_held(&self) -> bool {
        *self == Self::Held || *self == Self::JustPressed
    }
}

impl Game {
    fn set_key_state(&mut self, key: Key, action: Action) {
        match action {
            Action::Press => {
                self.key_states.insert(key, KeyState::JustPressed);
            }
            Action::Release => {
                self.key_states.insert(key, KeyState::Released);
            }
            _ => {}
        }
    }

    fn handle_mouse_pos(&mut self, x: f64, y: f64) {
        self.dmousex = x as f32 - self.mousex;
        self.dmousey = y as f32 - self.mousey;
        self.mousex = x as f32;
        self.mousey = y as f32;
    }

    pub fn handle_events(&mut self, events: &EventHandler) {
        //Handle events
        for (_, event) in glfw::flush_messages(events) {
            match event {
                WindowEvent::FramebufferSize(w, h) => {
                    gfx::handle_window_resize(w, h);
                }
                WindowEvent::Key(key, _scancode, action, _mods) => {
                    self.set_key_state(key, action);
                }
                WindowEvent::CursorPos(x, y) => {
                    self.handle_mouse_pos(x, y);
                }
                _ => {}
            }
        }
    }

    pub fn update_input_states(&mut self) {
        for state in self.key_states.values_mut() {
            if *state == KeyState::JustPressed {
                *state = KeyState::Held;
            }
        }

        self.dmousex = 0.0;
        self.dmousey = 0.0;
    }

    pub fn get_mouse_diff(&self) -> (f32, f32) {
        (self.dmousex, self.dmousey)
    }

    pub fn init_mouse_pos(&mut self, window: &PWindow) {
        let (mousex, mousey) = window.get_cursor_pos();
        self.mousex = mousex as f32;
        self.mousey = mousey as f32;
    }

    pub fn get_key_state(&self, key: Key) -> KeyState {
        match self.key_states.get(&key) {
            Some(state) => *state,
            _ => KeyState::Released,
        }
    }
}

pub fn release_cursor(gamestate: &Game, window: &mut PWindow) {
    if gamestate.get_key_state(Key::Escape) == KeyState::JustPressed {
        let cursormode = window.get_cursor_mode();

        if cursormode == CursorMode::Disabled {
            window.set_cursor_mode(CursorMode::Normal);
        } else {
            window.set_cursor_mode(CursorMode::Disabled);
        }
    }
}
