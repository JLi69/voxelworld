pub mod block_menu;
mod inventory;

use super::ChunkTables;
use crate::assets::Texture;
use crate::game::assets::models::draw_elements;
use crate::voxel;
use crate::{game::Game, BLOCK_REACH, EMPTY_BLOCK};
pub use block_menu::display_block_menu;
pub use inventory::display_hotbar;
use cgmath::{Matrix4, SquareMatrix, Vector3};

pub fn display_selected_outline(gamestate: &Game) {
    let outlineshader = gamestate.shaders.use_program("outline");
    outlineshader.uniform_vec4f("incolor", 0.1, 0.1, 0.1, 1.0);
    outlineshader.uniform_matrix4f("persp", &gamestate.persp);
    outlineshader.uniform_matrix4f("view", &gamestate.cam.get_view());
    outlineshader.uniform_float("outlinesz", 0.0075);

    //Calculate the selected voxel position
    let pos = gamestate.cam.position;
    let dir = gamestate.cam.forward();
    let (x, y, z, axis) = voxel::build::raycast(pos, dir, BLOCK_REACH, &gamestate.world);
    let (ix, iy, iz) = voxel::build::get_raycast_voxel(x, y, z, dir, axis);
    let (fx, fy, fz) = (ix as f32 + 0.5, iy as f32 + 0.5, iz as f32 + 0.5);
    let selectedv = Vector3::<f32>::new(fx, fy, fz);

    let mut transform: Matrix4<f32> = cgmath::Matrix4::identity();
    transform = transform * Matrix4::from_translation(selectedv);
    transform = transform * Matrix4::from_scale(1.001);
    outlineshader.uniform_matrix4f("transform", &transform);
    if gamestate.world.get_block(ix, iy, iz).id != EMPTY_BLOCK
        && !gamestate.world.get_block(ix, iy, iz).is_fluid()
    {
        let cube = gamestate.models.bind("cube");
        draw_elements(cube);
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
