use super::{EventHandler, Game};
use crate::gui;
use crate::{game, gfx, gui::pause_menu::PauseMenuAction};
use egui_backend::egui;
use egui_gl_glfw as egui_backend;
use glfw::{Context, Glfw, PWindow};

pub fn run(gamestate: &mut Game, window: &mut PWindow, glfw: &mut Glfw, events: &EventHandler) {
    if window.should_close() {
        return;
    }

    //Generate chunk vaos
    let mut chunkvaos = gfx::ChunkVaoTable::new();
    chunkvaos.generate_chunk_vaos(&gamestate.world);

    //egui
    let font = gamestate.get_font();
    let mut painter = egui_backend::Painter::new(window);
    let ctx = egui::Context::default();
    let native_pixels_per_point = window.get_content_scale().0;
    ctx.set_fonts(font);
    //Initialize egui input state
    let mut input_state = gui::init_egui_input_state(window);

    //Main loop
    let mut dt = 0.0f32;
    let mut fps_timer = 0.0;
    let mut frames = 0;
    let mut chunks_drawn = 0;
    let mut quit = false;
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    let game_start = std::time::Instant::now();
    while !window.should_close() && !quit {
        let start = std::time::Instant::now();

        //Display
        gfx::set_default_gl_state();
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

        //Display gui
        gui::set_ui_gl_state();
        let mut pause_action = None;
        if gamestate.paused {
            pause_action = gui::run_pause_menu(&ctx, &mut input_state, &mut painter);
        }

        //Handle pause menu action
        if let Some(pause_action) = pause_action {
            match pause_action {
                //Unpause the game
                PauseMenuAction::Unpause => {
                    gamestate.paused = false;
                    window.set_cursor_mode(glfw::CursorMode::Disabled);
                }
                //Quit to the main menu
                PauseMenuAction::QuitToMainMenu => {
                    gamestate.paused = false;
                    quit = true;
                    window.set_cursor_mode(glfw::CursorMode::Normal);
                    gamestate.reset();
                }
            }
        }

        gamestate.pause();
        if !gamestate.paused {
            //Update gameobjects
            gamestate.update_player(dt, window.get_cursor_mode());
            //Destroy and place blocks
            gamestate.build(&mut chunkvaos);
            gamestate.update_build_cooldown(dt);
        }
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
        //Update egui input state
        input_state.input.time = Some(game_start.elapsed().as_secs_f64());
        input_state.pixels_per_point = native_pixels_per_point;
        let (w, h) = window.get_framebuffer_size();
        painter.set_size(w as u32, h as u32);
        //Handle/update input states
        gamestate.update_input_states();
        gamestate.handle_events_egui(events, &mut input_state);
        let end = std::time::Instant::now();
        dt = (end - start).as_secs_f32();
    }

    //Clean up
    chunkvaos.clear();
}
