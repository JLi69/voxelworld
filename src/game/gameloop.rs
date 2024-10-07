use super::{EventHandler, Game};
use crate::{game, gfx};
use glfw::{Context, Glfw, PWindow};

pub fn run(gamestate: &mut Game, window: &mut PWindow, glfw: &mut Glfw, events: &EventHandler) {
    if window.should_close() {
        return;
    }

    gfx::set_default_gl_state();
    //Generate chunk vaos
    let mut chunkvaos = gfx::ChunkVaoTable::new();
    chunkvaos.generate_chunk_vaos(&gamestate.world);

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
        let drawn = chunkvaos.display_chunks(gamestate);
        chunks_drawn += drawn;
        //Display selection outline
        gfx::display::display_selected_outline(gamestate);

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
