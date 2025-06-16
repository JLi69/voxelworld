#![windows_subsystem = "windows"]

mod assets;
mod game;
mod gfx;
mod gui;
mod impfile;
mod voxel;

use game::{save, Game};
use gui::main_menu::MainMenuOutput;
use voxel::{flags::init_voxel_flags, World, CHUNK_SIZE_F32, EMPTY_BLOCK};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    //If the user provides the arguments --run-test-sims,
    //run the test simulations and then exit the program, however, this option
    //only exists for testing purposes.
    voxel::world::block_update::run_test_simulations(&args);

    //Attempt to create save directory
    save::create_save_dir();
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
    gamestate.load_config("cfg.impfile");
    gamestate.load_block_info("assets/block_info.impfile");
    gamestate
        .recipe_table
        .load_recipes("assets/item_aliases.impfile", "assets/recipes.impfile");
    gamestate.load_assets();
    gamestate.init_mouse_pos(&window);

    while !window.should_close() {
        let selected = gui::run_main_menu(&mut gamestate, &mut window, &mut glfw, &events);

        let quit_to_menu = match selected {
            MainMenuOutput::SelectWorld => {
                gui::run_select_world_menu(&mut gamestate, &mut window, &mut glfw, &events)
            }
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
