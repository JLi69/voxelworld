use super::{init_egui_input_state, menu_text, set_ui_gl_state, transparent_frame};
use crate::game::{EventHandler, Game};
use crate::gfx;
use egui_backend::egui::{self, Color32};
use egui_gl_glfw as egui_backend;
use glfw::{Context, Glfw, PWindow};
use std::fs::File;
use std::io::Read;

//Read `assets/credits.txt` and return the contents as a vector of strings where
//each element of the vector is a line in the file
fn read_credits_text() -> Vec<String> {
    let path = "assets/credits.txt";
    match File::open(path) {
        Ok(mut file) => {
            let mut buf = String::new();
            let res = file.read_to_string(&mut buf);
            match res {
                Ok(sz) => eprintln!("read {sz} bytes from {path}"),
                Err(msg) => eprintln!("{msg}"),
            }
            buf.lines().map(|s| s.to_string()).collect()
        }
        Err(msg) => {
            eprintln!("Failed to open: {path}");
            eprintln!("{msg}");
            vec![]
        }
    }
}

//Display the credits, line by line
fn display_credits(ui: &mut egui::Ui, credits: &[String]) {
    for text in credits {
        ui.label(menu_text(text, 20.0, Color32::WHITE));
    }
}

//Display credits
pub fn run_credits_screen(
    gamestate: &mut Game,
    window: &mut PWindow,
    glfw: &mut Glfw,
    events: &EventHandler,
) -> bool {
    let mut painter = egui_backend::Painter::new(window);
    let ctx = egui::Context::default();
    let native_pixels_per_point = window.get_content_scale().0;
    let font = gamestate.get_font();
    ctx.set_fonts(font);

    //Initialize egui input state
    let mut input_state = init_egui_input_state(window);

    let credits_text = read_credits_text();
    set_ui_gl_state();
    window.set_cursor_mode(glfw::CursorMode::Normal);
    let start = std::time::Instant::now();
    let mut quit_to_menu = false;
    while !window.should_close() && !quit_to_menu {
        //Display
        gfx::clear();

        //Update input state
        input_state.input.time = Some(start.elapsed().as_secs_f64());
        input_state.pixels_per_point = native_pixels_per_point;
        let (w, h) = window.get_size();
        painter.set_size(w as u32, h as u32);

        ctx.begin_pass(input_state.input.take());

        //Display credits
        egui::CentralPanel::default()
            .frame(transparent_frame())
            .show(&ctx, |ui| {
                ui.vertical(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(menu_text("Credits", 48.0, Color32::WHITE));
                        display_credits(ui, &credits_text);

                        //Return to main menu
                        if ui
                            .button(menu_text("Main Menu", 24.0, Color32::WHITE))
                            .clicked()
                        {
                            quit_to_menu = true;
                        }
                    });
                });
            });

        //End frame
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point: _,
            viewport_output: _,
        } = ctx.end_pass();

        //Handle copy pasting
        if !platform_output.copied_text.is_empty() {
            egui_backend::copy_to_clipboard(&mut input_state, platform_output.copied_text);
        }

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
