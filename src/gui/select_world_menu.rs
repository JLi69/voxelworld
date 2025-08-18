use super::{init_egui_input_state, menu_text, set_ui_gl_state, transparent_frame};
use crate::game::{save::SAVE_PATH, EventHandler, Game};
use crate::gfx;
use egui_backend::egui::{self, vec2, Color32, Pos2};
use egui_gl_glfw as egui_backend;
use glfw::{Context, Glfw, PWindow};

//State for the create world menu
struct SelectWorldMenuState {
    create_world: bool,
    quit_to_menu: bool,
    selected_world: String,
    to_delete: String,
    worlds: Vec<String>,
}

//Initialize the create world menu state
impl SelectWorldMenuState {
    fn new() -> Self {
        Self {
            create_world: false,
            quit_to_menu: false,
            selected_world: String::new(),
            to_delete: String::new(),
            worlds: vec![],
        }
    }

    fn should_quit(&self) -> bool {
        self.create_world || self.quit_to_menu
    }

    fn get_world_list(&mut self) {
        self.worlds.clear();
        match std::fs::read_dir(SAVE_PATH) {
            Ok(dir_contents) => {
                for entry in dir_contents.flatten() {
                    let name = entry.file_name().into_string().unwrap_or(String::new());
                    if name.is_empty() {
                        continue;
                    }

                    if entry.path().is_dir() {
                        self.worlds.push(name);
                    }
                }
            }
            Err(msg) => {
                eprintln!("Failed to read {SAVE_PATH}");
                eprintln!("{msg}");
            }
        }
        self.worlds.sort();
    }
}

//Display the select world gui
fn display_select_world(
    ui: &mut egui::Ui,
    menu_state: &mut SelectWorldMenuState,
    gamestate: &mut Game,
    h: i32,
) {
    ui.vertical_centered(|ui| {
        ui.label(" ");
        ui.label(menu_text("Select World", 32.0, Color32::WHITE));
        ui.label(" ");

        egui::ScrollArea::vertical()
            .max_height(h as f32 - 256.0)
            .show(ui, |ui| {
                for world in &menu_state.worlds {
                    ui.selectable_value(
                        &mut menu_state.selected_world,
                        world.clone(),
                        menu_text(world, 24.0, Color32::WHITE),
                    );
                }
            });

        ui.label(" ");
        if ui
            .button(menu_text("Play World!", 24.0, Color32::WHITE))
            .clicked()
            && !menu_state.selected_world.is_empty()
        {
            let path = SAVE_PATH.to_string() + menu_state.selected_world.clone().as_str() + "/";
            eprintln!("Attempting to load {path}...");
            gamestate.load_world(&path);
            menu_state.create_world = true;
        }

        //Delete world
        if ui
            .button(menu_text("Delete World", 24.0, Color32::WHITE))
            .clicked()
            && !menu_state.selected_world.is_empty()
        {
            menu_state.to_delete = menu_state.selected_world.clone();
        }

        if ui
            .button(menu_text("Main Menu", 24.0, Color32::WHITE))
            .clicked()
        {
            menu_state.quit_to_menu = true;
        }
    });
}

//Display the delete world gui
fn display_delete_world(ui: &mut egui::Ui, menu_state: &mut SelectWorldMenuState, h: i32) {
    ui.vertical_centered(|ui| {
        ui.add_space(h as f32 / 4.0);
        ui.label(menu_text(
            format!("You are about to delete: \"{}\"", menu_state.to_delete).as_str(),
            32.0,
            Color32::RED,
        ));
        ui.label(menu_text(
            "WARNING: Deleting a world is permanent, are you sure you want to do this?",
            40.0,
            Color32::RED,
        ));
        ui.label(" ");

        if ui
            .button(menu_text("Cancel", 24.0, Color32::WHITE))
            .clicked()
        {
            menu_state.to_delete = String::new();
        }

        //Delete world
        if ui
            .button(menu_text("Yes, delete it!", 24.0, Color32::RED))
            .clicked()
            && !menu_state.selected_world.is_empty()
        {
            let path = SAVE_PATH.to_string() + menu_state.selected_world.clone().as_str() + "/";
            eprintln!("Attempting to delete {path}...");
            if let Err(msg) = std::fs::remove_dir_all(&path) {
                eprintln!("Failed to delete: {path}");
                eprintln!("{msg}");
            }
            menu_state.to_delete = String::new();
            menu_state.selected_world = String::new();
            menu_state.get_world_list();
        }
    });
}

//Display the create world menu screen
pub fn run_select_world_menu(
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
    ctx.set_pixels_per_point(native_pixels_per_point);

    //Initialize egui input state
    let mut input_state = init_egui_input_state(window);

    set_ui_gl_state();
    window.set_cursor_mode(glfw::CursorMode::Normal);
    let start = std::time::Instant::now();
    let mut menu_state = SelectWorldMenuState::new();
    //Read world list
    menu_state.get_world_list();
    while !window.should_close() && !menu_state.should_quit() {
        //Display
        gfx::clear();

        //Update input state
        input_state.input.time = Some(start.elapsed().as_secs_f64());
        input_state.pixels_per_point = native_pixels_per_point;
        let (w, h) = window.get_size();
        painter.set_size(w as u32, h as u32);

        ctx.begin_pass(input_state.input.take());

        //Display create world menu
        let (w, h) = window.get_size();
        egui::Window::new("window")
            .movable(false)
            .title_bar(false)
            .fixed_size(vec2(w as f32 - 64.0, h as f32))
            .fixed_pos(Pos2::new(32.0, 0.0))
            .frame(transparent_frame())
            .show(&ctx, |ui| {
                if menu_state.to_delete.is_empty() {
                    display_select_world(ui, &mut menu_state, gamestate, h);
                } else {
                    display_delete_world(ui, &mut menu_state, h);
                }
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
        for command in platform_output.commands {
            if let egui::OutputCommand::CopyText(copied_text) = command {
                egui_backend::copy_to_clipboard(&mut input_state, copied_text);
            }
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

    menu_state.quit_to_menu
}
