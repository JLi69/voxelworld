use super::egui_backend;
use super::{init_egui_input_state, load_font, set_ui_gl_state};
use crate::game::{EventHandler, Game};
use crate::gfx;
use crate::voxel::world::WorldGenType;
use egui_backend::egui::{self, Color32};
use glfw::{Context, CursorMode, Glfw, PWindow};
use rand::Rng;
use std::hash::{DefaultHasher, Hash, Hasher};

//State for the main menu
struct MainMenuState {
    world_name: String,
    seed: String,
    gen_type: WorldGenType,
    quit_menu: bool,
}

//Initialize the main menu state
impl MainMenuState {
    fn new() -> Self {
        Self {
            world_name: "New World".to_string(),
            seed: "".to_string(),
            gen_type: WorldGenType::DefaultGen,
            quit_menu: false,
        }
    }
}

//Creates an egui frame that is completely transparent
fn transparent_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(egui::Color32::TRANSPARENT)
        .inner_margin(egui::Margin::symmetric(16.0, 16.0))
}

//Generates text to be displayed
fn menu_text(text: &str, sz: f32, col: Color32) -> egui::RichText {
    egui::RichText::new(text).size(sz).color(col)
}

//Displays the main title
fn display_main_title(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel")
        .frame(transparent_frame())
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(64.0);
                ui.label(menu_text("VOXELWORLD", 64.0, Color32::WHITE));
            });
        });
}

//Converts a string into a u32 seed
fn convert_string_to_seed(seed: String) -> u32 {
    match seed.parse::<u32>() {
        Ok(val) => val,
        Err(_) => {
            let mut hashstate = DefaultHasher::new();
            seed.hash(&mut hashstate);
            hashstate.finish() as u32
        }
    }
}

//For when the user wants to create a new world
fn create_new_world(menu_state: &mut MainMenuState, gamestate: &mut Game) {
    let seed = if !menu_state.seed.is_empty() {
        convert_string_to_seed(menu_state.seed.clone())
    } else {
        rand::thread_rng().gen()
    };
    gamestate.generate_world(seed, 3, menu_state.gen_type);
}

//Display the create world gui
fn display_create_world(ui: &mut egui::Ui, menu_state: &mut MainMenuState, gamestate: &mut Game) {
    ui.vertical_centered(|ui| {
        ui.label(menu_text("Create world", 32.0, Color32::WHITE));
        ui.label(menu_text("World Name", 24.0, Color32::WHITE));
        ui.text_edit_singleline(&mut menu_state.world_name);

        ui.label(" ");
        ui.label(menu_text("World Generation", 24.0, Color32::WHITE));
        if ui
            .radio(
                menu_state.gen_type == WorldGenType::DefaultGen,
                menu_text("Default", 20.0, Color32::WHITE),
            )
            .clicked()
        {
            menu_state.gen_type = WorldGenType::DefaultGen;
        }
        if ui
            .radio(
                menu_state.gen_type == WorldGenType::Flat,
                menu_text("Flat", 20.0, Color32::WHITE),
            )
            .clicked()
        {
            menu_state.gen_type = WorldGenType::Flat;
        }

        ui.label(" ");
        ui.label(menu_text("Seed", 24.0, Color32::WHITE));
        ui.text_edit_singleline(&mut menu_state.seed);
        if ui
            .button(menu_text("Create", 24.0, Color32::WHITE))
            .clicked()
        {
            menu_state.quit_menu = true;
            create_new_world(menu_state, gamestate);
        }
    });
}

//Display the main menu
pub fn run_main_menu(
    gamestate: &mut Game,
    window: &mut PWindow,
    glfw: &mut Glfw,
    events: &EventHandler,
) {
    let fonts = load_font();
    let mut painter = egui_backend::Painter::new(window);
    let ctx = egui::Context::default();
    let native_pixels_per_point = window.get_content_scale().0;
    ctx.set_fonts(fonts);

    //Initialize egui input state
    let mut input_state = init_egui_input_state(window);

    set_ui_gl_state();
    window.set_cursor_mode(CursorMode::Normal);
    let start = std::time::Instant::now();
    let mut menu_state = MainMenuState::new();
    while !window.should_close() && !menu_state.quit_menu {
        //Display
        gfx::clear();

        //Update input state
        input_state.input.time = Some(start.elapsed().as_secs_f64());
        input_state.pixels_per_point = native_pixels_per_point;
        let (w, h) = window.get_framebuffer_size();
        painter.set_size(w as u32, h as u32);

        ctx.begin_frame(input_state.input.take());

        //Display main menu
        display_main_title(&ctx);
        egui::CentralPanel::default()
            .frame(transparent_frame())
            .show(&ctx, |ui| {
                display_create_world(ui, &mut menu_state, gamestate);
            });

        //End frame
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output: _,
        } = ctx.end_frame();

        //Handle copy pasting
        if !platform_output.copied_text.is_empty() {
            egui_backend::copy_to_clipboard(&mut input_state, platform_output.copied_text);
        }

        //Display
        let clipped_shapes = ctx.tessellate(shapes, pixels_per_point);
        painter.paint_and_update_textures(1.0, &clipped_shapes, &textures_delta);

        //Handle/update input states
        gamestate.update_input_states();
        gamestate.handle_events_egui(events, &mut input_state);
        gfx::output_errors();
        window.swap_buffers();
        glfw.poll_events();
    }
}
