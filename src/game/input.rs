use crate::gfx;
use crate::Game;
use egui_gl_glfw::EguiInputState;
use glfw::CursorMode;
use glfw::MouseButton;
use glfw::{Action, GlfwReceiver, Key, PWindow, WindowEvent};

//GLFW event handler
pub type EventHandler = GlfwReceiver<(f64, glfw::WindowEvent)>;

//Input states for buttons - released, just pressed, and held
#[derive(Eq, PartialEq, Clone, Copy)]
pub enum KeyState {
    Released,
    JustPressed,
    Held,
}

impl KeyState {
    //Returns if the key has just been pressed or is held
    pub fn is_held(&self) -> bool {
        *self == Self::Held || *self == Self::JustPressed
    }
}

impl Game {
    //Set key state from based on a key being pressed or released
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

    //Set mouse button state from based on a mouse button being pressed or released
    fn set_mouse_state(&mut self, button: MouseButton, action: Action) {
        match action {
            Action::Press => {
                self.mouse_states.insert(button, KeyState::JustPressed);
            }
            Action::Release => {
                self.mouse_states.insert(button, KeyState::Released);
            }
            _ => {}
        }
    }

    //Keep track of mouse position and the change in mouse position
    fn handle_mouse_pos(&mut self, x: f64, y: f64) {
        self.dmousex = x as f32 - self.mousex;
        self.dmousey = y as f32 - self.mousey;
        self.mousex = x as f32;
        self.mousey = y as f32;
    }

    //Handle glfw events
    #[allow(dead_code)]
    pub fn handle_events(&mut self, events: &EventHandler) {
        //Handle events
        for (_, event) in glfw::flush_messages(events) {
            match event {
                //Handle window resize
                WindowEvent::FramebufferSize(w, h) => {
                    gfx::handle_window_resize(w, h);
                }
                //Handle key event
                WindowEvent::Key(key, _scancode, action, _mods) => {
                    self.set_key_state(key, action);
                }
                //Handle cursor position changes
                WindowEvent::CursorPos(x, y) => {
                    self.handle_mouse_pos(x, y);
                }
                //Handle mouse button event
                WindowEvent::MouseButton(button, action, _mods) => {
                    self.set_mouse_state(button, action);
                }
                _ => {}
            }
        }
    }

    //Handle glfw events
    pub fn handle_events_egui(
        &mut self,
        events: &EventHandler,
        egui_input_state: &mut EguiInputState,
    ) {
        //Handle events
        for (_, event) in glfw::flush_messages(events) {
            egui_gl_glfw::handle_event(event.clone(), egui_input_state);
            match event {
                //Handle window resize
                WindowEvent::FramebufferSize(w, h) => {
                    gfx::handle_window_resize(w, h);
                }
                //Handle key event
                WindowEvent::Key(key, _scancode, action, _mods) => {
                    self.set_key_state(key, action);
                }
                //Handle cursor position changes
                WindowEvent::CursorPos(x, y) => {
                    self.handle_mouse_pos(x, y);
                }
                //Handle mouse button event
                WindowEvent::MouseButton(button, action, _mods) => {
                    self.set_mouse_state(button, action);
                }
                _ => {}
            }
        }
    }

    //This should be called before the events are polled so that the input states
    //from the previous frame are updated
    pub fn update_input_states(&mut self) {
        //Change any just pressed keys into held
        for state in self.key_states.values_mut() {
            if *state == KeyState::JustPressed {
                *state = KeyState::Held;
            }
        }

        //Change any just pressed mouse buttons into held
        for state in self.mouse_states.values_mut() {
            if *state == KeyState::JustPressed {
                *state = KeyState::Held;
            }
        }

        //Set change in mouse position to be 0
        self.dmousex = 0.0;
        self.dmousey = 0.0;
    }

    //Returns the change in mouse position
    pub fn get_mouse_diff(&self) -> (f32, f32) {
        (self.dmousex, self.dmousey)
    }

    //Initializes the mouse position to be where the mouse is on the window
    //This should be called once at the beginning of the game
    pub fn init_mouse_pos(&mut self, window: &PWindow) {
        let (mousex, mousey) = window.get_cursor_pos();
        self.mousex = mousex as f32;
        self.mousey = mousey as f32;
    }

    //Returns the key state for a key
    //if the key does not exist in the key state map, then return released
    pub fn get_key_state(&self, key: Key) -> KeyState {
        match self.key_states.get(&key) {
            Some(state) => *state,
            _ => KeyState::Released,
        }
    }

    //Returns the mouse button state for a mouse button
    //if the mouse button does not exist in the mouse button state map, then
    //it will just return released
    pub fn get_mouse_state(&self, button: MouseButton) -> KeyState {
        match self.mouse_states.get(&button) {
            Some(state) => *state,
            _ => KeyState::Released,
        }
    }
}

//Release/capture the mouse cursor if escape is pressed
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
