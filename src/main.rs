mod assets;
mod game;
mod gfx;
mod voxel;

use game::Game;
use gfx::ChunkVaoTable;
use glfw::{Context, Key};
use voxel::{World, CHUNK_SIZE_F32};

fn main() {
    //Attempt to initialize glfw
    let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw!");
    let (mut window, events) = game::init_window(&mut glfw);
    //Initialize game state
    let mut gamestate = Game::new();
    gamestate.init();
    gamestate.init_mouse_pos(&window);
    //Initialize gl
    gl::load_with(|s| window.get_proc_address(s) as *const _);

    //Generate world
    let mut world = World::new(2);
    world.gen_flat();
    //Generate chunk vaos
    let mut chunkvaos = ChunkVaoTable::new(world.get_chunk_count());
    chunkvaos.generate_chunk_vaos(&world);

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
        gfx::clear();

        let persp = gfx::calculate_perspective(&window);
        let view = gamestate.cam.get_view();

        //Display chunks
        chunkshader.uniform_matrix4f("persp", &persp);
        chunkshader.uniform_matrix4f("view", &view);
        unsafe {
            for (i, vao) in chunkvaos.vaos.iter().enumerate() {
                if chunkvaos.vertex_count[i] == 0 {
                    continue;
                }

                let pos = chunkvaos.chunk_positions[i];
                let x = pos.x as f32 * CHUNK_SIZE_F32;
                let y = pos.y as f32 * CHUNK_SIZE_F32;
                let z = pos.z as f32 * CHUNK_SIZE_F32;
                chunkshader.uniform_vec3f("chunkpos", x, y, z);
                gl::BindVertexArray(*vao);
                gl::DrawArrays(gl::TRIANGLES, 0, chunkvaos.vertex_count[i]);
            }
        }

        //Update gameobjects
        gamestate.cam.update(dt);

        gfx::output_errors();
        window.swap_buffers();
        glfw.poll_events();

        //Handle input
        let (dmousex, dmousey) = gamestate.get_mouse_diff();
        gamestate.cam.rotate(dmousex, dmousey, 0.04);
        //Move camera
        let w = gamestate.get_key_state(Key::W);
        let s = gamestate.get_key_state(Key::S);
        let a = gamestate.get_key_state(Key::A);
        let d = gamestate.get_key_state(Key::D);
        let shift = gamestate.get_key_state(Key::LeftShift);
        let space = gamestate.get_key_state(Key::Space);
        gamestate.cam.strafe(a, d);
        gamestate.cam.move_forward(w, s);
        gamestate.cam.fly(shift, space);
        //Release cursor
        game::release_cursor(&gamestate, &mut window);
        //Handle/update input states
        gamestate.update_input_states();
        gamestate.handle_events(&events);

        let end = std::time::Instant::now();
        dt = (end - start).as_secs_f32();
    }
}
