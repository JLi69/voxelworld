use glfw::Context;
use glfw::WindowEvent;
use glfw::{Action, Key, Modifiers, Scancode};

fn handle_window_resize(w: i32, h: i32) {
    unsafe {
        gl::Viewport(0, 0, w, h);
    }
}

//TODO: add code for handling key input
fn handle_key_input(_key: Key, _scancode: Scancode, _action: Action, _mods: Modifiers) {}

fn handle_window_events(events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>) {
    //Handle events
    for (_, event) in glfw::flush_messages(events) {
        match event {
            WindowEvent::Size(w, h) => {
                handle_window_resize(w, h);
            }
            WindowEvent::Key(key, scancode, action, mods) => {
                handle_key_input(key, scancode, action, mods);
            }
            _ => {}
        }
    }
}

fn main() {
    //Attempt to initialize glfw
    let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw!");

    //Attempt to initialize the window
    let (mut window, events) = glfw
        .create_window(960, 640, "voxelworld", glfw::WindowMode::Windowed)
        .expect("Failed to init window!");
    window.set_key_polling(true);
    window.make_current();

    //Initialize gl
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    //Main loop
    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        window.swap_buffers();
        glfw.poll_events();
        handle_window_events(&events);
    }
}
