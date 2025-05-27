pub mod create_world_menu;
pub mod credits_screen;
pub mod death_screen;
pub mod debug;
pub mod main_menu;
pub mod pause_menu;
pub mod select_world_menu;

pub use create_world_menu::run_create_world_menu;
pub use credits_screen::run_credits_screen;
pub use death_screen::run_death_screen;
pub use debug::display_debug_window;
use egui_backend::{
    egui::{self, vec2, Color32, Pos2, RawInput, Rect},
    glfw::PWindow,
    EguiInputState,
};
use egui_gl_glfw as egui_backend;
pub use main_menu::run_main_menu;
pub use pause_menu::run_pause_menu;
pub use select_world_menu::run_select_world_menu;

//Initialized the egui input state
pub fn init_egui_input_state(window: &PWindow) -> EguiInputState {
    let (w, h) = window.get_size();
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

//Creates an egui frame that is completely transparent
fn transparent_frame() -> egui::Frame {
    egui::Frame::none().inner_margin(egui::Margin::symmetric(16.0, 16.0))
}

//Generates text to be displayed
fn menu_text(text: &str, sz: f32, col: Color32) -> egui::RichText {
    egui::RichText::new(text).size(sz).color(col)
}
