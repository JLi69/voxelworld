use super::input::convert_mouse_pos;
use super::{EventHandler, Game};
use crate::assets::Texture;
use crate::gfx::buildchunk::generate_chunk_vertex_data;
use crate::gfx::display::block_menu::{BLOCK_MENU_HEIGHT, BLOCK_MENU_WIDTH};
use crate::gfx::fluid::generate_fluid_vertex_data;
use crate::gui;
use crate::{game, gfx, gui::pause_menu::PauseMenuAction};
use egui_backend::egui;
use egui_gl_glfw as egui_backend;
use glfw::{Context, Glfw, PWindow};

const SAVE_TIME_INTERVAL: f32 = 60.0;

pub fn run(gamestate: &mut Game, window: &mut PWindow, glfw: &mut Glfw, events: &EventHandler) {
    if window.should_close() {
        return;
    }

    //Generate chunk vaos
    let mut chunktables = gfx::ChunkTables::new();
    chunktables
        .chunk_vaos
        .generate_chunk_vaos(&gamestate.world, |chunk, world| {
            generate_chunk_vertex_data(chunk, world.get_adjacent(chunk))
        });
    chunktables
        .lava_vaos
        .generate_chunk_vaos(&gamestate.world, |chunk, world| {
            generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 13)
        });
    chunktables
        .water_vaos
        .generate_chunk_vaos(&gamestate.world, |chunk, world| {
            generate_fluid_vertex_data(chunk, world.get_adjacent(chunk), world, 12)
        });
    //water framebuffer
    let mut water_framebuffer = 0u32;
    let mut water_frame_color = Texture::new();
    let mut depth_rbo = 0u32;
    water_frame_color.gen_texture();
    unsafe {
        //Create render buffer
        gl::GenRenderbuffers(1, &mut depth_rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, depth_rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, 960, 640);

        //Initialize frame buffer
        gl::GenFramebuffers(1, &mut water_framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, water_framebuffer);
        water_frame_color.bind();
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            960,
            640,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            water_frame_color.get_id(),
            0,
        );
        gl::FramebufferRenderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_STENCIL_ATTACHMENT,
            gl::RENDERBUFFER,
            depth_rbo,
        );

        //Check framebuffer status
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            eprintln!("ERROR: framebuffer is not complete!");
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    //egui
    let font = gamestate.get_font();
    let mut painter = egui_backend::Painter::new(window);
    let ctx = egui::Context::default();
    let native_pixels_per_point = window.get_content_scale().0;
    ctx.set_fonts(font);
    //Initialize egui input state
    let mut input_state = gui::init_egui_input_state(window);

    gamestate.save_entire_world();
    gamestate.world.update_all_chunks();
    //Main loop
    let mut dt = 0.0f32;
    let mut fps = 0;
    let mut fps_timer = 0.0;
    let mut save_timer = 0.0;
    let mut time_passed = 0.0;
    let mut frames = 0;
    let mut chunks_drawn = 0;
    let mut quit = false;
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    let game_start = std::time::Instant::now();
    while !window.should_close() && !quit {
        let start = std::time::Instant::now();

        //Get mouse position
        let (mousex, mousey) = window.get_cursor_pos();
        let (mousex, mousey) = (mousex as i32, mousey as i32); //Convert to i32
                                                               //Update render buffer and water frame dimensions
        let (w, h) = window.get_size();
        unsafe {
            gl::BindRenderbuffer(gl::RENDERBUFFER, depth_rbo);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, w, h);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0); //Unbind render buffer
            water_frame_color.bind();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                w,
                h,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::BindTexture(gl::TEXTURE_2D, 0); //Unbind texture
        }

        //Display
        gfx::set_default_gl_state();
        gfx::set_nondefault_background_color(gamestate);
        gfx::clear();
        //Update perspective matrix
        let persp = gfx::calculate_perspective(window, &gamestate.cam);
        gamestate.persp = persp;
        let aspect = gfx::calculate_aspect(window);
        gamestate.aspect = aspect;

        //Display chunks
        chunks_drawn += chunktables.chunk_vaos.display_chunks(gamestate, "chunk");
        let fluid_shader = gamestate.shaders.get("fluid");
        fluid_shader.use_program();
        fluid_shader.uniform_float("timepassed", time_passed);
        fluid_shader.uniform_float("flowspeed", 0.07);
        chunktables
            .lava_vaos
            .display_with_backface(gamestate, "fluid");
        gfx::display::display_water(
            gamestate,
            &chunktables,
            water_framebuffer,
            &water_frame_color,
            w,
            h,
        );

        if gamestate.display_hud {
            //Display selection outline
            gfx::display::display_selected_outline(gamestate);
        }

        if gamestate.player.suffocating(&gamestate.world) {
            gfx::display::display_suffocation_screen(gamestate, w, h);
        }

        if gamestate.display_hud {
            //Display crosshair
            gfx::display::display_crosshair(gamestate, w, h);
            //Display hotbar
            gfx::display::display_hotbar(gamestate, w, h);
        }
        //Display gui
        gui::set_ui_gl_state();
        gamestate.update_display_debug();
        let mut pause_action = None;
        if gamestate.display_debug {
            //Display debug screen
            gui::display_debug_window(&ctx, &mut input_state, &mut painter, gamestate, fps);
            if gamestate.get_display_block_menu() {
                gfx::display::display_block_menu(gamestate, w, h, mousex, mousey);
            }
        } else if gamestate.display_block_menu {
            gfx::display::display_block_menu(gamestate, w, h, mousex, mousey);
            let menu =
                game::block_menu::get_positions(gamestate, -BLOCK_MENU_WIDTH, BLOCK_MENU_HEIGHT);
            let (mousex_f32, mousey_f32) = convert_mouse_pos(mousex, mousey, w, h);
            game::block_menu::select_block(gamestate, &menu, mousex_f32, mousey_f32);
        } else if gamestate.paused {
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
                    quit = true;
                    window.set_cursor_mode(glfw::CursorMode::Normal);
                }
            }
        }

        gamestate.toggle_hud();
        gamestate.pause();
        if !gamestate.paused || gamestate.display_block_menu {
            gamestate.player.hotbar.scroll(gamestate.get_scroll_state());
        }

        if !gamestate.paused {
            //Update gameobjects
            gamestate.update_player(dt, window.get_cursor_mode());
            //Destroy and place blocks
            gamestate.build(&mut chunktables);
            gamestate.update_build_cooldown(dt);
            //Update blocks
            gamestate.world.update_blocks(dt, &mut chunktables, 1);
        }
        //Generate new chunks
        gamestate.world.check_for_cache_clear();
        gamestate.world.clean_cache();
        gamestate
            .world
            .generate_more(gamestate.player.position, &mut chunktables);
        chunktables.update_tables(gamestate);

        //Handle save
        save_timer -= dt;
        if save_timer < 0.0 {
            gamestate.save_game();
            save_timer = SAVE_TIME_INTERVAL;
        }

        //Output FPS
        fps_timer += dt;
        if fps_timer > 1.0 {
            eprintln!("FPS: {frames} | Chunks drawn: {chunks_drawn}");
            fps_timer = 0.0;
            fps = frames;
            frames = 0;
            chunks_drawn = 0;
        } else {
            frames += 1;
        }

        if !gamestate.paused {
            time_passed += dt;
        }

        gfx::output_errors();
        window.swap_buffers();
        glfw.poll_events();
        //Release cursor
        game::release_cursor(gamestate, window);
        //Update egui input state
        input_state.input.time = Some(game_start.elapsed().as_secs_f64());
        input_state.pixels_per_point = native_pixels_per_point;
        painter.set_size(w as u32, h as u32);
        //Handle/update input states
        gamestate.update_input_states();
        gamestate.handle_events_egui(events, &mut input_state);
        let end = std::time::Instant::now();
        dt = (end - start).as_secs_f32().min(1.0);
    }

    unsafe {
        gl::DeleteFramebuffers(1, &water_framebuffer);
        gl::DeleteRenderbuffers(1, &depth_rbo);
    }
    gamestate.save_entire_world();
    gamestate.reset();
    //Clean up
    chunktables.clear();
}
