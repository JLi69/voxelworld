mod assets;
mod game;
mod gfx;
mod gui;
mod impfile;
mod voxel;

use game::Game;
use gui::main_menu::MainMenuOutput;
use voxel::{build::BLOCK_REACH, flags::init_voxel_flags, World, CHUNK_SIZE_F32, EMPTY_BLOCK};

fn main() {
    //Attempt to initialize glfw
    let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw!");
    let (mut window, events) = game::init_window(&mut glfw);
    //Initialize gl
    gl::load_with(|s| window.get_proc_address(s) as *const _);
    //Initialize voxel flags
    init_voxel_flags();
    //Initialize game state
    let mut gamestate = Game::new();
    gamestate.init();
    gamestate.load_assets();
    gamestate.init_mouse_pos(&window);

    while !window.should_close() {
        let selected = gui::run_main_menu(&mut gamestate, &mut window, &mut glfw, &events);

        let quit_to_menu = match selected {
            MainMenuOutput::CreateWorld => {
                gui::run_create_world_menu(&mut gamestate, &mut window, &mut glfw, &events)
            }
            MainMenuOutput::Credits => {
                gui::run_credits_screen(&mut gamestate, &mut window, &mut glfw, &events)
            }
            _ => true,
        };

        if !quit_to_menu {
            game::run(&mut gamestate, &mut window, &mut glfw, &events);
        }
    }
}
