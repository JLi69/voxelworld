use egui_backend::egui::{Color32, RichText};
use egui_gl_glfw::egui;
use egui_gl_glfw as egui_backend;
use crate::game::Game;
use super::transparent_frame;

pub fn debug_text(s: &str) -> RichText {
    RichText::new(s).heading().color(Color32::WHITE).background_color(Color32::DARK_GRAY)
}

fn debug_info(
    ctx: &egui::Context, 
    input_state: &egui_backend::EguiInputState,
    gamestate: &Game, 
    fps: i32
) {
    let playerx = gamestate.player.position.x;
    let playery = gamestate.player.position.y;
    let playerz = gamestate.player.position.z;
    let player_pos = format!("x = {playerx:.4}, y = {playery:.4}, z = {playerz:.4}");

    let native_pixels_per_point = format!(
        "ctx.native_pixels_per_point() = {} | input_state.pixels_per_point = {}",
        ctx.native_pixels_per_point().unwrap_or(1.0),
        input_state.pixels_per_point,
    );

    let pixels_per_point = format!(
        "ctx.pixels_per_point() = {}",
        ctx.pixels_per_point(),
    );

    let paused = format!("paused = {}", gamestate.is_paused());

    let fps_text = format!("{fps} FPS");

    egui::TopBottomPanel::top("debug")
        .frame(transparent_frame())
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.heading(debug_text("Debug info - press F3 to hide"));
            ui.heading(debug_text(&player_pos));
            ui.heading(debug_text(&native_pixels_per_point));
            ui.heading(debug_text(&pixels_per_point));
            ui.heading(debug_text(&paused));
            ui.heading(debug_text(&fps_text));
        });
}

pub fn display_debug_window(
    ctx: &egui::Context,
    input_state: &mut egui_backend::EguiInputState,
    painter: &mut egui_backend::Painter,
    gamestate: &Game,
    fps: i32,
) {
    //Begin frame
    ctx.begin_pass(input_state.input.take());

    debug_info(ctx, input_state, gamestate, fps);

    //End frame
    let egui::FullOutput {
        platform_output: _,
        textures_delta,
        shapes,
        pixels_per_point,
        viewport_output: _,
    } = ctx.end_pass();

    //Display
    let clipped_shapes = ctx.tessellate(shapes, pixels_per_point);
    painter.paint_and_update_textures(1.0, &clipped_shapes, &textures_delta);
}
