mod assets;
mod game;
mod gfx;
mod voxel;

use game::Game;
use glfw::Context;
use voxel::{World, CHUNK_SIZE_F32};

fn main() {
    //Attempt to initialize glfw
    let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw!");
    let (mut window, events) = game::init_window(&mut glfw);
    //Initialize game state
    let mut gamestate = Game::new();
    gamestate.init();
    gamestate.init_mouse_pos(&window);
    gamestate.generate_world(3);
    //Initialize gl
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    //Generate chunk vaos
    let mut chunkvaos = gfx::ChunkVaoTable::new(gamestate.world.get_chunk_count());
    chunkvaos.generate_chunk_vaos(&gamestate.world);

    //Create shaders
    let vert = "assets/shaders/chunkvert.glsl";
    let frag = "assets/shaders/chunkfrag.glsl";
    let chunkshader = assets::program_from_vert_and_frag(vert, frag);
    chunkshader.use_program();

    gfx::set_default_gl_state();
    //Main loop
    let mut dt = 0.0f32;
    while !window.should_close() {
        let start = std::time::Instant::now();

        //Display
        gfx::clear();
        //Update perspective matrix
        let persp = gfx::calculate_perspective(&window);
        gamestate.persp = persp;
        //Display chunks
        chunkvaos.display_chunks(&chunkshader, &gamestate);

        //Update gameobjects
        gamestate.update_camera(dt, window.get_cursor_mode());
        //Destroy and place blocks
        gamestate.build(&mut chunkvaos);

        gfx::output_errors();
        window.swap_buffers();
        glfw.poll_events();
        //Release cursor
        game::release_cursor(&gamestate, &mut window);
        //Handle/update input states
        gamestate.update_input_states();
        gamestate.handle_events(&events);
        let end = std::time::Instant::now();
        dt = (end - start).as_secs_f32();
    }
}
