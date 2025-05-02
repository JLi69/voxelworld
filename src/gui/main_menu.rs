use super::egui_backend;
use super::{init_egui_input_state, set_ui_gl_state};
use super::{menu_text, transparent_frame};
use crate::game::{EventHandler, Game};
use crate::{gfx, gui};
use egui_backend::egui::{self, vec2, Color32, Pos2};
use glfw::{Context, CursorMode, Glfw, PWindow};

//Selections that the user can make on the main menu
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MainMenuOutput {
    CreateWorld,
    SelectWorld,
    Credits,
    Quit,
}

//Displays all the buttons on the main menu
fn display_main_menu(ui: &mut egui::Ui) -> Option<MainMenuOutput> {
    let mut selected = None;

    ui.vertical_centered(|ui| {
        if ui
            .button(menu_text("Play World", 28.0, Color32::WHITE))
            .clicked()
        {
            selected = Some(MainMenuOutput::SelectWorld);
        }

        ui.label(" ");
        if ui
            .button(menu_text("New World", 28.0, Color32::WHITE))
            .clicked()
        {
            selected = Some(MainMenuOutput::CreateWorld);
        }

        ui.label(" ");
        if ui
            .button(menu_text("Credits", 28.0, Color32::WHITE))
            .clicked()
        {
            selected = Some(MainMenuOutput::Credits);
        }

        ui.label(" ");
        if ui
            .button(menu_text("Quit Game", 28.0, Color32::WHITE))
            .clicked()
        {
            selected = Some(MainMenuOutput::Quit);
        }
    });

    selected
}

//Display the main menu
pub fn run_main_menu(
    gamestate: &mut Game,
    window: &mut PWindow,
    glfw: &mut Glfw,
    events: &EventHandler,
) -> MainMenuOutput {
    let font = gamestate.get_font();
    let mut painter = egui_backend::Painter::new(window);
    let ctx = egui::Context::default();
    let native_pixels_per_point = window.get_content_scale().0;
    ctx.set_fonts(font);

    //Initialize egui input state
    let mut input_state = init_egui_input_state(window);

    set_ui_gl_state();
    window.set_cursor_mode(CursorMode::Normal);
    let start = std::time::Instant::now();
    let mut selected = None;
    let cloud_offset = fastrand::f32() * 2048.0;
    while !window.should_close() && selected.is_none() {
        gfx::set_default_gl_state();
        //Display
        gfx::clear();

        //Update perspective matrix
        let persp = gfx::calculate_perspective(window, &gamestate.cam);
        gamestate.persp = persp;
        let aspect = gfx::calculate_aspect(window);
        gamestate.aspect = aspect;
        let cloud_time = (start.elapsed().as_secs_f32() + cloud_offset) * 2.0;
        gfx::display::display_clouds_menu(gamestate, cloud_time);

        gui::set_ui_gl_state();
        //Update input state
        input_state.input.time = Some(start.elapsed().as_secs_f64());
        input_state.pixels_per_point = native_pixels_per_point;
        let (w, h) = window.get_size();
        painter.set_size(w as u32, h as u32);

        ctx.begin_pass(input_state.input.take());

        //Display main menu
        let (width, height) = window.get_size();
        egui::Window::new("window")
            .movable(false)
            .title_bar(false)
            .fixed_size(vec2(width as f32, height as f32 - 32.0))
            .fixed_pos(Pos2::new(0.0, 0.0))
            .scroll(true)
            .frame(transparent_frame())
            .show(&ctx, |ui| {
                //Display the main title
                ui.vertical_centered(|ui| {
                    ui.add_space(64.0);
                    ui.label(menu_text("VOXELWORLD", 64.0, Color32::DARK_GRAY));
                    ui.add_space(64.0);
                });
                selected = display_main_menu(ui);
            });

        //End frame
        let egui::FullOutput {
            platform_output: _,
            textures_delta,
            shapes,
            pixels_per_point: _,
            viewport_output: _,
        } = ctx.end_pass();

        //Display
        let clipped_shapes = ctx.tessellate(shapes, native_pixels_per_point);
        painter.paint_and_update_textures(
            native_pixels_per_point,
            &clipped_shapes,
            &textures_delta,
        );

        //Handle/update input states
        gamestate.update_input_states();
        gamestate.handle_events_egui(events, &mut input_state);
        gfx::output_errors();
        window.swap_buffers();
        glfw.poll_events();
    }

    let selected = selected.unwrap_or(MainMenuOutput::Quit);

    if selected == MainMenuOutput::Quit {
        window.set_should_close(true);
    }

    selected
}
