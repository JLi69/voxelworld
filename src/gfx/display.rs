pub mod block_menu;
mod hand;
mod inventory;

use super::chunktable::set_fog;
use super::ChunkTables;
use crate::assets::shader::ShaderProgram;
use crate::assets::Texture;
use crate::game::assets::models::{draw_elements, draw_elements_instanced};
use crate::game::physics::Hitbox;
use crate::voxel::{self, CHUNK_SIZE_F32};
use crate::{game::Game, EMPTY_BLOCK};
pub use block_menu::display_block_menu;
use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};
pub use hand::display_hand_item;
pub use inventory::display_hotbar;

pub fn display_selected_outline(gamestate: &Game) {
    let outlineshader = gamestate.shaders.use_program("outline");
    outlineshader.uniform_vec4f("incolor", 0.1, 0.1, 0.1, 1.0);
    outlineshader.uniform_matrix4f("persp", &gamestate.persp);
    outlineshader.uniform_matrix4f("view", &gamestate.cam.get_view());
    outlineshader.uniform_float("outlinesz", 0.005);

    //Calculate the selected voxel position
    let pos = gamestate.cam.position;
    let dir = gamestate.cam.forward();
    let (ix, iy, iz) = voxel::build::get_selected(pos, dir, &gamestate.world);
    let block = gamestate.world.get_block(ix, iy, iz);
    let bbox = Hitbox::from_block_bbox(ix, iy, iz, block);

    let mut transform: Matrix4<f32> = cgmath::Matrix4::identity();
    transform = transform * Matrix4::from_translation(bbox.position);
    let (sx, sy, sz) = (
        bbox.dimensions.x * 1.005,
        bbox.dimensions.y * 1.005,
        bbox.dimensions.z * 1.005,
    );
    transform = transform * Matrix4::from_nonuniform_scale(sx, sy, sz);
    outlineshader.uniform_matrix4f("transform", &transform);
    outlineshader.uniform_vec3f("scale", sx, sy, sz);
    if gamestate.world.get_block(ix, iy, iz).id != EMPTY_BLOCK
        && !gamestate.world.get_block(ix, iy, iz).is_fluid()
    {
        let cube = gamestate.models.bind("cube");

        unsafe {
            gl::Disable(gl::CULL_FACE);
        }

        draw_elements(cube);

        unsafe {
            gl::Enable(gl::CULL_FACE);
        }
    }
}

pub fn display_water(
    gamestate: &Game,
    chunktables: &ChunkTables,
    water_framebuffer: u32,
    water_frame: &Texture,
    w: i32,
    h: i32,
) {
    let fluid_shader = gamestate.shaders.get("fluid");
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, water_framebuffer);
        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, water_framebuffer);
        gl::BlitFramebuffer(0, 0, w, h, 0, 0, w, h, gl::DEPTH_BUFFER_BIT, gl::NEAREST);

        gl::BindFramebuffer(gl::FRAMEBUFFER, water_framebuffer);
    }
    fluid_shader.uniform_float("flowspeed", 0.25);
    chunktables
        .water_vaos
        .display_with_backface(gamestate, "fluid");
    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }

    //Display quad with water textured on
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }
    let quadshader = gamestate.shaders.get("quad");
    quadshader.use_program();
    water_frame.bind();
    let quad = gamestate.models.bind("quad2d");
    quadshader.uniform_float("alpha", 0.7);
    draw_elements(quad);
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }
}

pub fn display_crosshair(gamestate: &Game, w: i32, h: i32) {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    gamestate.textures.bind("crosshair");
    gamestate.shaders.use_program("2d");
    let shader2d = gamestate.shaders.get("2d");
    let quad = gamestate.models.bind("quad2d");

    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    const CROSSHAIR_SIZE: f32 = 12.0;
    let transform = Matrix4::from_scale(CROSSHAIR_SIZE);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_matrix4f("transform", &transform);
    shader2d.uniform_float("alpha", 0.4);

    draw_elements(quad);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }
}

pub fn display_suffocation_screen(gamestate: &Game, w: i32, h: i32) {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    gamestate.textures.bind("black_bg");
    gamestate.shaders.use_program("2d");
    let shader2d = gamestate.shaders.get("2d");
    let quad = gamestate.models.bind("quad2d");

    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    let transform = Matrix4::from_nonuniform_scale(w as f32, h as f32, 0.0);
    shader2d.uniform_matrix4f("transform", &transform);
    shader2d.uniform_float("alpha", 1.0);
    draw_elements(quad);

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::CULL_FACE);
    }
}

pub fn display_clouds(gamestate: &Game, time_passed: f32) {
    unsafe {
        gl::Disable(gl::CULL_FACE);
    }

    gamestate.textures.bind("clouds");
    gamestate.shaders.use_program("clouds");
    let cloud_shader = gamestate.shaders.get("clouds");
    let quad = gamestate.models.bind("quad2d");

    let persp = gamestate.persp;
    cloud_shader.uniform_matrix4f("persp", &persp);
    let view = gamestate.cam.get_view();
    cloud_shader.uniform_matrix4f("view", &view);
    let mut transform = Matrix4::identity();
    let sz = gamestate.world.get_range() as f32 * CHUNK_SIZE_F32 * 16.0;
    transform = Matrix4::from_nonuniform_scale(sz, 0.0, sz) * transform;
    let camx = gamestate.cam.position.x;
    let camz = gamestate.cam.position.z;
    let height = 160.0 + gamestate.cam.position.y;
    transform = Matrix4::from_translation(Vector3::new(camx, height, camz)) * transform;
    cloud_shader.uniform_matrix4f("transform", &transform);
    cloud_shader.uniform_vec3f(
        "campos",
        gamestate.cam.position.x,
        gamestate.cam.position.y,
        gamestate.cam.position.z,
    );
    cloud_shader.uniform_float("total_time", time_passed);
    set_fog(gamestate, &cloud_shader, get_skycolor(gamestate.world.time));
    draw_elements(quad);

    unsafe {
        gl::Enable(gl::CULL_FACE);
    }
}

//Assumes 0.0 < t < 1.0
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    a * (1.0 - t) + b * t
}

fn lerp_col(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> (f32, f32, f32) {
    let (ar, ag, ab) = a;
    let (br, bg, bb) = b;
    (lerp(ar, br, t), lerp(ag, bg, t), lerp(ab, bb, t))
}

const WHITE: (f32, f32, f32) = (1.0, 1.0, 1.0);
const DAY: (f32, f32, f32) = (0.4, 0.8, 1.0);
const NIGHT: (f32, f32, f32) = (0.1, 0.1, 0.1);
const ORANGE: (f32, f32, f32) = (1.0, 0.25, 0.0);
const YELLOW: (f32, f32, f32) = (1.0, 0.8, 0.0);
const TRANSITION_TIME: f32 = 0.04;

//Returns rgb
pub fn get_skycolor(t: f32) -> (f32, f32, f32) {
    if t < TRANSITION_TIME {
        lerp_col(NIGHT, DAY, (t + TRANSITION_TIME) / (2.0 * TRANSITION_TIME))
    } else if t > 1.0 - TRANSITION_TIME {
        lerp_col(
            NIGHT,
            DAY,
            (t - 1.0 + TRANSITION_TIME) / (2.0 * TRANSITION_TIME),
        )
    } else if (t - 0.5).abs() < TRANSITION_TIME {
        lerp_col(
            DAY,
            NIGHT,
            (t - (0.5 - TRANSITION_TIME)) / (2.0 * TRANSITION_TIME),
        )
    } else if t > 0.5 {
        //Night
        NIGHT
    } else {
        //Day
        DAY
    }
}

fn get_bot_skycolor(t: f32) -> (f32, f32, f32) {
    if t < TRANSITION_TIME {
        lerp_col(ORANGE, DAY, t / TRANSITION_TIME)
    } else if t > 1.0 - TRANSITION_TIME {
        lerp_col(NIGHT, ORANGE, (t - 1.0 + TRANSITION_TIME) / TRANSITION_TIME)
    } else if t < 0.5 && t > 0.5 - TRANSITION_TIME {
        lerp_col(DAY, ORANGE, (t - 0.5 + TRANSITION_TIME) / TRANSITION_TIME)
    } else if t > 0.5 && t < 0.5 + TRANSITION_TIME {
        lerp_col(ORANGE, NIGHT, (t - 0.5) / TRANSITION_TIME)
    } else if t > 0.5 && t < 1.0 {
        NIGHT
    } else {
        DAY
    }
}

fn set_sky_color(shader: &ShaderProgram, t: f32) {
    let (topr, topg, topb) = get_skycolor(t);
    shader.uniform_vec3f("topcolor", topr, topg, topb);
    let (botr, botg, botb) = get_bot_skycolor(t);
    shader.uniform_vec3f("botcolor", botr, botg, botb);
}

fn get_sun_color(t: f32) -> (f32, f32, f32) {
    if t < TRANSITION_TIME {
        lerp_col(
            YELLOW,
            WHITE,
            (t + TRANSITION_TIME) / (TRANSITION_TIME * 2.0),
        )
    } else if t > 1.0 - TRANSITION_TIME {
        lerp_col(
            YELLOW,
            WHITE,
            (t - 1.0 + TRANSITION_TIME) / (TRANSITION_TIME * 2.0),
        )
    } else if t > 0.5 - TRANSITION_TIME && t < 0.5 + TRANSITION_TIME {
        lerp_col(
            WHITE,
            YELLOW,
            (t - 0.5 + TRANSITION_TIME) / (TRANSITION_TIME * 2.0),
        )
    } else if t > TRANSITION_TIME && t < 0.5 - TRANSITION_TIME {
        WHITE
    } else {
        YELLOW
    }
}

fn get_star_alpha(t: f32) -> f32 {
    if t > 1.0 - TRANSITION_TIME {
        lerp(1.0, 0.0, (t - 1.0 + TRANSITION_TIME) / TRANSITION_TIME)
    } else if t > 0.5 && t < 0.5 + TRANSITION_TIME {
        lerp(0.0, 1.0, (t - 0.5) / TRANSITION_TIME)
    } else if t >= 0.0 && t <= 0.5 {
        0.0
    } else {
        1.0
    }
}

pub fn display_sky(gamestate: &Game) {
    unsafe {
        gl::Disable(gl::CULL_FACE);
    }

    let persp = gamestate.persp;
    let view = gamestate.cam.get_view_no_translate();

    //Display skybox
    let cube = gamestate.models.bind("cube");
    gamestate.shaders.use_program("skybox");
    let skybox_shader = gamestate.shaders.get("skybox");
    skybox_shader.uniform_matrix4f("persp", &persp);
    skybox_shader.uniform_matrix4f("view", &view);
    skybox_shader.uniform_matrix4f("transform", &Matrix4::identity());
    set_sky_color(&skybox_shader, gamestate.world.time);
    draw_elements(cube);

    let rotation = -gamestate.world.time * 360.0;
    let quad = gamestate.models.bind("quad2d");

    //Draw stars
    gamestate.shaders.use_program("stars");
    let star_shader = gamestate.shaders.get("stars");
    star_shader.uniform_matrix4f("persp", &persp);
    star_shader.uniform_matrix4f("view", &view);
    star_shader.uniform_float("alpha", get_star_alpha(gamestate.world.time));
    star_shader.uniform_vec2f("tcScale", 1.0, 1.0);
    star_shader.uniform_vec2f("tcOffset", 0.0, 0.0);
    gamestate.textures.bind("star");
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_angle_z(Deg(90.0)) * transform;
    transform = Matrix4::from_angle_x(Deg(45.0)) * transform;
    transform = Matrix4::from_translation(Vector3::new(-320.0, 0.0, 0.0)) * transform;
    star_shader.uniform_float("rotation", rotation);
    star_shader.uniform_matrix4f("transform", &transform);
    star_shader.uniform_vec4f("tint", 1.0, 1.0, 1.0, 1.0);
    draw_elements_instanced(quad.clone(), 800);

    gamestate.shaders.use_program("skyobject");
    let shader = gamestate.shaders.get("skyobject");
    shader.uniform_matrix4f("persp", &persp);
    shader.uniform_matrix4f("view", &view);
    shader.uniform_float("alpha", 1.0);

    //Draw the sun
    shader.uniform_vec2f("tcScale", 1.0, 1.0);
    shader.uniform_vec2f("tcOffset", 0.0, 0.0);
    gamestate.textures.bind("sun");
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_scale(8.0) * transform;
    transform = Matrix4::from_angle_z(Deg(90.0)) * transform;
    transform = Matrix4::from_translation(Vector3::new(-160.0, 0.0, 0.0)) * transform;
    transform = Matrix4::from_angle_z(Deg(rotation)) * transform;
    shader.uniform_matrix4f("transform", &transform);
    let (sunr, sung, sunb) = get_sun_color(gamestate.world.time);
    shader.uniform_vec4f("tint", sunr, sung, sunb, 1.0);
    draw_elements(quad.clone());

    //Draw the moon
    //Calculate the phase of the moon
    let phase = if gamestate.world.time > 0.25 {
        //This is to prevent the player from seeing the moon suddenly change
        //phases at the end of the day when the moon is still visible
        ((gamestate.world.days_passed + 8) % 16) / 2
    } else {
        ((gamestate.world.days_passed + 8 - 1) % 16) / 2
    };
    shader.uniform_vec2f("tcScale", 1.0 / 8.0, 1.0);
    shader.uniform_vec2f("tcOffset", 1.0 / 8.0 * phase as f32, 0.0);
    shader.uniform_vec4f("tint", 1.0, 1.0, 1.0, 1.0);
    gamestate.textures.bind("moon");
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_scale(8.0) * transform;
    transform = Matrix4::from_angle_z(Deg(90.0)) * transform;
    transform = Matrix4::from_translation(Vector3::new(160.0, 0.0, 0.0)) * transform;
    transform = Matrix4::from_angle_z(Deg(rotation)) * transform;
    shader.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());

    unsafe {
        gl::Enable(gl::CULL_FACE);
    }
}
