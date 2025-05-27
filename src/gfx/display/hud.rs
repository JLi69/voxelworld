use crate::game::{
    assets::models::draw_elements,
    player::{DEFAULT_MAX_HEALTH, DROWN_TIME},
    Game, GameMode,
};
use cgmath::{Matrix4, SquareMatrix, Vector3};

fn display_stamina(gamestate: &Game, w: i32, h: i32) {
    gamestate.shaders.use_program("icon2d");
    let shader2d = gamestate.shaders.get("icon2d");
    let quad = gamestate.models.bind("quad2d");

    //Set screen matrix
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_float("texscale", 1.0 / 4.0);

    let width = 9.0 * 32.0 / 2.0 - 16.0;
    let x = width + 48.0;
    let y = -h as f32 / 2.0 + 64.0 + 20.0;

    //Display the stamina bar background
    gamestate.textures.bind("hud_icons");
    shader2d.uniform_vec2f("texoffset", 0.5, 0.25);
    shader2d.uniform_float("alpha", 0.4);
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_nonuniform_scale(width, 7.0, 1.0) * transform;
    transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());

    //Display the stamina bar
    shader2d.uniform_vec2f("texoffset", 0.75, 0.25);
    shader2d.uniform_float("alpha", 0.8);
    let mut transform = Matrix4::identity();
    let stamina = (width - 2.0) * gamestate.player.stamina;
    transform = Matrix4::from_nonuniform_scale(stamina, 4.0, 1.0) * transform;
    let offset = (1.0 - gamestate.player.stamina) * (width - 2.0);
    transform = Matrix4::from_translation(Vector3::new(x - offset, y, 0.0)) * transform;
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());

    //Display icon
    shader2d.uniform_vec2f("texoffset", 0.25, 0.25);
    shader2d.uniform_float("alpha", 1.0);
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_scale(16.0) * transform;
    transform = Matrix4::from_translation(Vector3::new(32.0, y, 0.0)) * transform;
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());
}

fn display_health(gamestate: &Game, w: i32, h: i32) {
    gamestate.shaders.use_program("icon2d");
    let shader2d = gamestate.shaders.get("icon2d");
    let quad = gamestate.models.bind("quad2d");
    gamestate.textures.bind("hud_icons");
    //Set screen matrix
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_float("alpha", 1.0);
    shader2d.uniform_float("texscale", 1.0 / 4.0);
    if gamestate.player.health == 0 {
        //Display all empty hearts if health is 0
        shader2d.uniform_vec2f("texoffset", 0.0, 0.0);
    } else {
        shader2d.uniform_vec2f("texoffset", 0.5, 0.0);
    }
    for i in 0..(DEFAULT_MAX_HEALTH / 2) {
        let hp_ind = (i + 1) * 2;
        if hp_ind - gamestate.player.health == 1 {
            shader2d.uniform_vec2f("texoffset", 0.25, 0.0);
        }

        let time = gamestate.world.time * 10000.0 * std::f32::consts::PI;
        let time_offset = (i * i) as f32 * 3.0 + i as f32;
        let y_offset = if gamestate.player.health <= 4 {
            (time + time_offset).sin() * 2.0
        } else {
            0.0
        };
        let x = i as f32 * 30.0 - 32.0 * 9.0;
        let y = -h as f32 / 2.0 + 64.0 + 20.0 + y_offset;
        let mut transform = Matrix4::identity();
        transform = Matrix4::from_scale(15.0) * transform;
        transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
        shader2d.uniform_matrix4f("transform", &transform);
        draw_elements(quad.clone());

        if hp_ind == gamestate.player.health || hp_ind - 1 == gamestate.player.health {
            shader2d.uniform_vec2f("texoffset", 0.0, 0.0);
        }
    }
}

fn display_oxygen_bar(gamestate: &Game, w: i32, h: i32) {
    if gamestate.player.drowning_timer >= DROWN_TIME - 0.01 {
        return;
    }

    gamestate.shaders.use_program("icon2d");
    let shader2d = gamestate.shaders.get("icon2d");
    let quad = gamestate.models.bind("quad2d");
    gamestate.textures.bind("hud_icons");
    //Set screen matrix
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_float("alpha", 1.0);
    shader2d.uniform_float("texscale", 1.0 / 4.0);
    shader2d.uniform_vec2f("texoffset", 0.75, 0.0);
    let timer = gamestate.player.drowning_timer / (DROWN_TIME / 10.0);
    for i in 0..(timer.ceil() as i32) {
        let diff = timer - i as f32;
        if diff < 0.1 {
            shader2d.uniform_vec2f("texoffset", 0.0, 0.25);
        }

        let x = i as f32 * 30.0 - 32.0 * 9.0;
        let y = -h as f32 / 2.0 + 64.0 + 50.0;
        let mut transform = Matrix4::identity();
        transform = Matrix4::from_scale(15.0) * transform;
        transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
        shader2d.uniform_matrix4f("transform", &transform);
        draw_elements(quad.clone());
    }
}

//When the player takes damage, the screen flashes red
fn display_damage_flash(gamestate: &Game, w: i32, h: i32) {
    gamestate.shaders.use_program("icon2d");
    let shader2d = gamestate.shaders.get("icon2d");
    let quad = gamestate.models.bind("quad2d");
    gamestate.textures.bind("hud_icons");
    //Set screen matrix
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_float("alpha", gamestate.player.damage_timer_perc() * 0.4);
    shader2d.uniform_float("texscale", 1.0 / 4.0);
    shader2d.uniform_vec2f("texoffset", 0.0, 0.5);
    let transform = Matrix4::from_nonuniform_scale(w as f32, h as f32, 1.0);
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad);
}

//Displays health bar, stamina, breath
pub fn display_stats(gamestate: &Game, w: i32, h: i32) {
    if gamestate.game_mode() != GameMode::Survival {
        return;
    }

    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    //display stamina bar
    display_stamina(gamestate, w, h);
    //Display oxygen bar
    display_oxygen_bar(gamestate, w, h);
    //display health bar
    display_health(gamestate, w, h);
    //display damage flash
    display_damage_flash(gamestate, w, h);

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }
}
