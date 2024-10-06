mod assets;
mod game;
mod gfx;
mod gui;
mod voxel;

use game::Game;
use voxel::{build::BLOCK_REACH, flags::init_voxel_flags, World, CHUNK_SIZE_F32, EMPTY_BLOCK};

fn main() {
    //Attempt to initialize glfw
    let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw!");
    let (mut window, events) = game::init_window(&mut glfw);
    //Initialize voxel flags
    init_voxel_flags();
    //Initialize game state
    let mut gamestate = Game::new();
    gamestate.init();
    gamestate.init_mouse_pos(&window);
    //Initialize gl
    gl::load_with(|s| window.get_proc_address(s) as *const _);
    gfx::set_default_gl_state();

    gui::run_main_menu(&mut gamestate, &mut window, &mut glfw, &events);
    game::run(&mut gamestate, &mut window, &mut glfw, &events);
}
