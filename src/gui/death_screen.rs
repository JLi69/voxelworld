use super::{menu_text, pause_menu::PauseMenuAction};
use egui_backend::egui::{self, Color32};
use egui_gl_glfw as egui_backend;

//Creates an egui frame that is grayed out
fn death_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_unmultiplied(255, 0, 0, 200))
        .inner_margin(egui::Margin::symmetric(16, 16))
}

//Title for pause menu
fn death_title(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel")
        .frame(death_frame())
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(64.0);
                ui.label(menu_text("You Died!", 64.0, Color32::WHITE));
            });
        });
}

//Display buttons for pause menu
fn display_death_screen(ui: &mut egui::Ui, msg: &str) -> Option<PauseMenuAction> {
    let mut action = None;
    ui.vertical_centered(|ui| {
        ui.add_space(32.0);
        ui.label(menu_text(msg, 24.0, Color32::WHITE));

        ui.add_space(64.0);
        if ui
            .button(menu_text("Respawn", 32.0, Color32::WHITE))
            .clicked()
        {
            action = Some(PauseMenuAction::Respawn);
        }

        ui.add_space(64.0);
        if ui
            .button(menu_text("Quit to Main Menu", 32.0, Color32::WHITE))
            .clicked()
        {
            action = Some(PauseMenuAction::QuitToMainMenu);
        }
    });

    action
}

//Returns action chosen by the user
//should be run in a main game loop
pub fn run_death_screen(
    ctx: &egui::Context,
    input_state: &mut egui_backend::EguiInputState,
    painter: &mut egui_backend::Painter,
    msg: &str,
) -> Option<PauseMenuAction> {
    //Begin frame
    ctx.begin_pass(input_state.input.take());

    death_title(ctx);

    let mut action = None;
    egui::CentralPanel::default()
        .frame(death_frame())
        .show(ctx, |ui| {
            action = display_death_screen(ui, msg);
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
    let native_pixels_per_point = input_state.pixels_per_point;
    let clipped_shapes = ctx.tessellate(shapes, native_pixels_per_point);
    painter.paint_and_update_textures(native_pixels_per_point, &clipped_shapes, &textures_delta);

    action
}
