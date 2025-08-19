use super::transparent_frame;
use super::{egui_backend, menu_text};
use super::{init_egui_input_state, set_ui_gl_state};
use crate::game::settings::{CloudDisplay, Settings};
use crate::game::{EventHandler, Game};
use crate::{gfx, gui};
use egui_backend::egui::{self, vec2, Color32, Pos2, Ui};
use glfw::{Context, CursorMode, Glfw, PWindow};

//Returns whether to quit to the main menu
fn display_settings_menu(ui: &mut Ui, settings: &mut Settings) {
    ui.heading(menu_text("Settings", 48.0, Color32::WHITE));

    ui.add_space(24.0);
    //Radio options for clouds
    ui.heading(menu_text("Clouds", 32.0, Color32::WHITE));
    let selected = settings.cloud_display == CloudDisplay::Fancy;
    let text = menu_text("Fancy", 20.0, Color32::WHITE);
    if ui.radio(selected, text).clicked() {
        settings.cloud_display = CloudDisplay::Fancy;
    }

    let selected = settings.cloud_display == CloudDisplay::Flat;
    let text = menu_text("Flat", 20.0, Color32::WHITE);
    if ui.radio(selected, text).clicked() {
        settings.cloud_display = CloudDisplay::Flat;
    }

    let selected = settings.cloud_display == CloudDisplay::Disabled;
    let text = menu_text("Disabled", 20.0, Color32::WHITE);
    if ui.radio(selected, text).clicked() {
        settings.cloud_display = CloudDisplay::Disabled;
    }

    ui.add_space(24.0);
    if ui
        .button(menu_text("Reset to Defaults", 24.0, Color32::WHITE))
        .clicked()
    {
        *settings = Settings::default();
    }

    ui.add_space(24.0);
}

//Display the settings menu
pub fn run_settings_menu(
    gamestate: &mut Game,
    window: &mut PWindow,
    glfw: &mut Glfw,
    events: &EventHandler,
) -> bool {
    let font = gamestate.get_font();
    let mut painter = egui_backend::Painter::new(window);
    let ctx = egui::Context::default();
    let native_pixels_per_point = window.get_content_scale().0;
    ctx.set_fonts(font);
    ctx.set_pixels_per_point(native_pixels_per_point);

    //Initialize egui input state
    let mut input_state = init_egui_input_state(window);

    set_ui_gl_state();
    window.set_cursor_mode(CursorMode::Normal);
    let start = std::time::Instant::now();
    let mut quit_to_menu = false;
    while !window.should_close() && !quit_to_menu {
        gfx::set_default_gl_state();
        //Display
        gfx::clear();

        //Update perspective matrix
        let persp = gfx::calculate_perspective(window, &gamestate.cam);
        gamestate.persp = persp;
        let aspect = gfx::calculate_aspect(window);
        gamestate.aspect = aspect;

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
                ui.vertical_centered(|ui| {
                    display_settings_menu(ui, &mut gamestate.settings);

                    if ui
                        .button(menu_text("Main Menu", 24.0, Color32::WHITE))
                        .clicked()
                    {
                        quit_to_menu = true;
                    }
                })
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

    quit_to_menu
}
