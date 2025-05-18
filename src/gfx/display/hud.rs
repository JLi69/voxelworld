use cgmath::{Matrix4, SquareMatrix, Vector3};
use crate::game::{Game, assets::models::draw_elements, GameMode};

fn display_stamina(gamestate: &Game, w: i32, h: i32) { 
    gamestate.shaders.use_program("2d");
    let shader2d = gamestate.shaders.get("2d");
    let quad = gamestate.models.bind("quad2d");

    //Set screen matrix
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat); 

    let width = 9.0 * 32.0 / 2.0 - 16.0;
    let x = width + 32.0;
    let y = -h as f32 / 2.0 + 64.0 + 20.0;

    //Display the stamina bar background
    gamestate.textures.bind("black_bg");
    shader2d.uniform_float("alpha", 0.4); 
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_nonuniform_scale(width, 7.0, 1.0) * transform; 
    transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());

    //Display the stamina bar
    gamestate.textures.bind("stamina_bg");
    shader2d.uniform_float("alpha", 0.8);
    let mut transform = Matrix4::identity();
    let stamina = (width - 2.0) * gamestate.player.stamina;
    transform = Matrix4::from_nonuniform_scale(stamina, 4.0, 1.0) * transform;
    let offset = (1.0 - gamestate.player.stamina) * (width - 2.0);
    transform = Matrix4::from_translation(Vector3::new(x - offset, y, 0.0)) * transform;
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());

    //Display icon
    gamestate.textures.bind("stamina_icon");
    shader2d.uniform_float("alpha", 1.0);
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_scale(16.0) * transform;
    transform = Matrix4::from_translation(Vector3::new(16.0, y, 0.0)) * transform;
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());
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

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }
}
