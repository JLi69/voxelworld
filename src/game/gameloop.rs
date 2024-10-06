use super::{EventHandler, Game};
use crate::{assets, assets::Texture, game, gfx};
use glfw::{Context, Glfw, PWindow};

pub fn run(gamestate: &mut Game, window: &mut PWindow, glfw: &mut Glfw, events: &EventHandler) {
    if window.should_close() {
        return;
    }

    gfx::set_default_gl_state();
    //Generate chunk vaos
    let mut chunkvaos = gfx::ChunkVaoTable::new();
    chunkvaos.generate_chunk_vaos(&gamestate.world);

    //TODO: make a way to easily access and manage shaders, textures, and models
    //Create shaders
    //Chunk shader
    let vert = "assets/shaders/chunkvert.glsl";
    let frag = "assets/shaders/chunkfrag.glsl";
    let chunkshader = assets::program_from_vert_and_frag(vert, frag);
    //Cube outline shader
    let vert = "assets/shaders/vert.glsl";
    let frag = "assets/shaders/outlinefrag.glsl";
    let outlineshader = assets::program_from_vert_and_frag(vert, frag);

    //Load textures
    let blocktexture = match Texture::load_from_file("assets/textures/blocktextures.png") {
        Ok(tex) => tex,
        Err(msg) => {
            eprintln!("{msg}");
            Texture::new()
        }
    };

    //Generate models
    let cube = gfx::models::gen_cube_vao();

    //Main loop
    let mut dt = 0.0f32;
    let mut fps_timer = 0.0;
    let mut frames = 0;
    let mut chunks_drawn = 0;
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    while !window.should_close() {
        let start = std::time::Instant::now();

        //Display
        gfx::clear();
        //Update perspective matrix
        let persp = gfx::calculate_perspective(window, &gamestate.cam);
        gamestate.persp = persp;
        let aspect = gfx::calculate_aspect(window);
        gamestate.aspect = aspect;

        //Display chunks
        blocktexture.bind();
        let drawn = chunkvaos.display_chunks(&chunkshader, gamestate);
        chunks_drawn += drawn;
        //Display selection outline
        gfx::display::display_selected_outline(&outlineshader, gamestate, &cube);

        //Update gameobjects
        gamestate.update_player(dt, window.get_cursor_mode());
        //Destroy and place blocks
        gamestate.build(&mut chunkvaos);
        gamestate.update_build_cooldown(dt);
        //Generate new chunks
        gamestate.world.check_for_cache_clear();
        gamestate.world.clean_cache();
        gamestate
            .world
            .generate_more(gamestate.player.position, &mut chunkvaos);
        chunkvaos.update_chunks(&gamestate.world);

        //Output FPS
        fps_timer += dt;
        if fps_timer > 1.0 {
            eprintln!("FPS: {frames} | Chunks drawn: {chunks_drawn}");
            fps_timer = 0.0;
            frames = 0;
            chunks_drawn = 0;
        } else {
            frames += 1;
        }

        gfx::output_errors();
        window.swap_buffers();
        glfw.poll_events();
        //Release cursor
        game::release_cursor(gamestate, window);
        //Handle/update input states
        gamestate.update_input_states();
        gamestate.handle_events(events);
        let end = std::time::Instant::now();
        dt = (end - start).as_secs_f32();
    }

    //Clean up
    chunkvaos.clear();
}
