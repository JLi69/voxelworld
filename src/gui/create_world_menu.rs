use super::{init_egui_input_state, menu_text, set_ui_gl_state, transparent_frame};
use crate::game::{save, EventHandler, Game, GameMode};
use crate::gfx;
use crate::voxel::world::WorldGenType;
use egui_backend::egui::{self, vec2, Color32, Pos2};
use egui_gl_glfw as egui_backend;
use glfw::{Context, Glfw, PWindow};
use std::hash::{DefaultHasher, Hash, Hasher};

//State for the create world menu
struct CreateWorldMenuState {
    world_name: String,
    seed: String,
    gen_type: WorldGenType,
    game_mode: GameMode,
    create_world: bool,
    quit_to_menu: bool,
}

//Initialize the create world menu state
impl CreateWorldMenuState {
    fn new() -> Self {
        Self {
            world_name: "New World".to_string(),
            seed: "".to_string(),
            gen_type: WorldGenType::DefaultGen,
            game_mode: GameMode::Survival,
            create_world: false,
            quit_to_menu: false,
        }
    }

    fn should_quit(&self) -> bool {
        self.create_world || self.quit_to_menu
    }
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
fn create_new_world(menu_state: &mut CreateWorldMenuState, gamestate: &mut Game) {
    let seed = if !menu_state.seed.is_empty() {
        convert_string_to_seed(menu_state.seed.clone())
    } else {
        fastrand::u32(..)
    };
    gamestate.generate_world(seed, 7, menu_state.gen_type, menu_state.game_mode);
    gamestate.world.init_block_light();
    gamestate.world.init_sky_light();
}

//Display the create world gui
fn display_create_world(
    ui: &mut egui::Ui,
    menu_state: &mut CreateWorldMenuState,
    gamestate: &mut Game,
) {
    ui.vertical_centered(|ui| {
        ui.label(menu_text("Create world", 32.0, Color32::WHITE));
        ui.label(menu_text("World Name", 24.0, Color32::WHITE));
        let world_name_edit = egui::TextEdit::singleline(&mut menu_state.world_name)
            .char_limit(128)
            .font(egui::TextStyle::Heading);
        ui.add(world_name_edit);

        ui.label(" ");
        ui.label(menu_text("World Generation", 24.0, Color32::WHITE));

        //Radio options for world generation
        let selected = menu_state.gen_type == WorldGenType::DefaultGen;
        let text = menu_text("Default", 20.0, Color32::WHITE);
        if ui.radio(selected, text).clicked() {
            menu_state.gen_type = WorldGenType::DefaultGen;
        }

        let selected = menu_state.gen_type == WorldGenType::OldGen;
        let text = menu_text("Old", 20.0, Color32::WHITE);
        if ui.radio(selected, text).clicked() {
            menu_state.gen_type = WorldGenType::OldGen;
        }

        let selected = menu_state.gen_type == WorldGenType::Flat;
        let text = menu_text("Flat", 20.0, Color32::WHITE);
        if ui.radio(selected, text).clicked() {
            menu_state.gen_type = WorldGenType::Flat;
        }

        ui.add_space(8.0);
        ui.label(menu_text("Game Mode", 24.0, Color32::WHITE));
        //Radio options for game mode
        let selected = menu_state.game_mode == GameMode::Survival;
        let text = menu_text("Survival", 20.0, Color32::WHITE);
        if ui.radio(selected, text).clicked() {
            menu_state.game_mode = GameMode::Survival;
        }

        let selected = menu_state.game_mode == GameMode::Creative;
        let text = menu_text("Creative", 20.0, Color32::WHITE);
        if ui.radio(selected, text).clicked() {
            menu_state.game_mode = GameMode::Creative;
        }

        ui.label(" ");
        ui.label(menu_text("Seed", 24.0, Color32::WHITE));
        let world_seed_edit =
            egui::TextEdit::singleline(&mut menu_state.seed).font(egui::TextStyle::Heading);
        ui.add(world_seed_edit);

        if ui
            .button(menu_text("Create", 24.0, Color32::WHITE))
            .clicked()
            && !menu_state.world_name.is_empty()
        {
            menu_state.create_world = true;
            let path = save::get_world_path(&menu_state.world_name);
            create_new_world(menu_state, gamestate);
            gamestate.world.path = path;
            eprintln!("Created world: {}", gamestate.world.path);
            if let Err(msg) = save::create_world_dir(&gamestate.world.path) {
                eprintln!("Error: failed to create world: {msg}");
                menu_state.create_world = false;
            }
        }

        if ui
            .button(menu_text("Main Menu", 24.0, Color32::WHITE))
            .clicked()
        {
            menu_state.quit_to_menu = true;
        }

        ui.add_space(24.0);
    });
}

//Display the create world menu screen
pub fn run_create_world_menu(
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

    set_ui_gl_state();
    window.set_cursor_mode(glfw::CursorMode::Normal);
    let start = std::time::Instant::now();
    let mut menu_state = CreateWorldMenuState::new();
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
        let (width, height) = window.get_size();
        egui::Window::new("window")
            .movable(false)
            .title_bar(false)
            .fixed_size(vec2(width as f32, height as f32))
            .fixed_pos(Pos2::new(0.0, 0.0))
            .frame(transparent_frame())
            .scroll(true)
            .show(&ctx, |ui| {
                display_create_world(ui, &mut menu_state, gamestate);
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

    menu_state.quit_to_menu
}
