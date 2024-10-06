pub mod main_menu;

use egui_backend::{
    egui::{vec2, FontData, FontDefinitions, FontFamily, Pos2, RawInput, Rect},
    glfw::PWindow,
    EguiInputState,
};
use egui_gl_glfw as egui_backend;
pub use main_menu::run_main_menu;
use std::{fs::File, io::Read};

//Initialized the egui input state
fn init_egui_input_state(window: &PWindow) -> EguiInputState {
    let (w, h) = window.get_framebuffer_size();
    let native_pixels_per_point = window.get_content_scale().0;
    let dimensions = vec2(w as f32, h as f32) / native_pixels_per_point;
    let rect = Rect::from_min_size(Pos2::new(0.0, 0.0), dimensions);
    let raw_input = RawInput {
        screen_rect: Some(rect),
        ..Default::default()
    };
    EguiInputState::new(raw_input, native_pixels_per_point)
}

//Sets the OpenGL state for rendering gui components
pub fn set_ui_gl_state() {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::ClearColor(0.4, 0.8, 1.0, 1.0);
    }
}

//TODO: add a better way of loading fonts
pub fn load_font() -> FontDefinitions {
    //Load fonts
    let mut fonts = FontDefinitions::default();
    let path = "assets/fonts/8BitOperator/8bitOperatorPlus-Regular.ttf".to_string();
    let font_file = File::open(&path);
    match font_file {
        Ok(mut font_file) => {
            let mut bytes = vec![];
            let res = font_file.read_to_end(&mut bytes);
            match res {
                Ok(sz) => eprintln!("read {sz} bytes from {path}"),
                Err(msg) => eprintln!("{msg}"),
            }
            fonts
                .font_data
                .insert("8BitOperator".to_string(), FontData::from_owned(bytes));
        }
        Err(msg) => {
            eprintln!("Failed to open: {path}");
            eprintln!("{msg}");
        }
    }

    if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
        prop.insert(0, "8BitOperator".to_string());
    }

    if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
        mono.insert(0, "8BitOperator".to_string());
    }

    fonts
}
