use super::menu_text;
use egui_backend::egui::{self, Color32};
use egui_gl_glfw as egui_backend;

pub enum PauseMenuAction {
    Unpause,
    QuitToMainMenu,
}

//Creates an egui frame that is grayed out
fn pause_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::from_rgba_unmultiplied(32, 32, 32, 200))
        .inner_margin(egui::Margin::symmetric(16.0, 16.0))
}

//Title for pause menu
fn puase_title(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel")
        .frame(pause_frame())
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(64.0);
                ui.label(menu_text("Paused", 64.0, Color32::WHITE));
            });
        });
}

//Display buttons for pause menu
fn display_pause_menu(ui: &mut egui::Ui) -> Option<PauseMenuAction> {
    let mut action = None;
    ui.vertical_centered(|ui| {
        ui.add_space(64.0);
        if ui
            .button(menu_text("Return to Game", 32.0, Color32::WHITE))
            .clicked()
        {
            action = Some(PauseMenuAction::Unpause);
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
pub fn run_pause_menu(
    ctx: &egui::Context,
    input_state: &mut egui_backend::EguiInputState,
    painter: &mut egui_backend::Painter,
) -> Option<PauseMenuAction> {
    //Begin frame
    ctx.begin_frame(input_state.input.take());

    puase_title(ctx);

    let mut action = None;
    egui::CentralPanel::default()
        .frame(pause_frame())
        .show(ctx, |ui| {
            action = display_pause_menu(ui);
        });

    //End frame
    let egui::FullOutput {
        platform_output: _,
        textures_delta,
        shapes,
        pixels_per_point,
        viewport_output: _,
    } = ctx.end_frame();

    //Display
    let clipped_shapes = ctx.tessellate(shapes, pixels_per_point);
    painter.paint_and_update_textures(1.0, &clipped_shapes, &textures_delta);

    action
}
