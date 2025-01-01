use crate::assets::Texture;
use crate::game::assets::models::draw_elements;
use crate::game::inventory::Item;
use crate::gfx::buildchunk::{add_block_vertices, add_block_vertices_transparent, add_block_vertices_fluid};
use crate::gfx::chunktable::ChunkVao;
use crate::voxel::{self, Chunk};
use crate::{game::Game, BLOCK_REACH, EMPTY_BLOCK};
use cgmath::{Matrix4, SquareMatrix, Vector3, Deg};
use super::ChunkTables;

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

pub fn display_hotbar(
    gamestate: &Game,
    w: i32,
    h: i32,
) {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    gamestate.textures.bind("hotbar_icon");
    gamestate.shaders.use_program("2d");
    let shader2d = gamestate.shaders.get("2d");
    let quad = gamestate.models.bind("quad2d");

    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    const HOTBAR_SIZE: f32 = 32.0; //In pixels
    let hotbar_sz = gamestate.player.hotbar.items.len();
    for i in 0..hotbar_sz {
        let position = Vector3::new(
            i as f32 * HOTBAR_SIZE * 2.0 - HOTBAR_SIZE * hotbar_sz as f32 + HOTBAR_SIZE,
            -h as f32 / 2.0 + HOTBAR_SIZE,
            0.0
        );
        let mut transform = Matrix4::identity();

        let size = if i == gamestate.player.hotbar.selected {
            shader2d.uniform_float("alpha", 1.0);
            HOTBAR_SIZE * 18.0 / 16.0
        } else {
            shader2d.uniform_float("alpha", 0.6);
            HOTBAR_SIZE
        };

        transform = Matrix4::from_scale(size) * transform;
        transform = Matrix4::from_translation(position) * transform;
        shader2d.uniform_matrix4f("transform", &transform);
        draw_elements(quad.clone());
    }

    unsafe {
        gl::Enable(gl::CULL_FACE);
    }

    gamestate.textures.bind("blocks");
    gamestate.shaders.use_program("orthographic");
    let orthographic_shader = gamestate.shaders.get("orthographic");
    orthographic_shader.uniform_matrix4f("screen", &screen_mat);
    orthographic_shader.uniform_vec3f("offset", -1.5, -1.5, -1.5);
    let mut chunk = Chunk::new(0, 0, 0);
    for (i, item) in gamestate.player.hotbar.items.iter().enumerate() {
        let position = Vector3::new(
            i as f32 * HOTBAR_SIZE * 2.0 - HOTBAR_SIZE * hotbar_sz as f32 + HOTBAR_SIZE,
            -h as f32 / 2.0 + HOTBAR_SIZE,
            0.0
        );

        let mut transform = Matrix4::identity();
        let size = if i == gamestate.player.hotbar.selected {
            HOTBAR_SIZE * 18.0 / 16.0 * 14.0 / 16.0
        } else {
            HOTBAR_SIZE * 14.0 / 16.0
        };
        transform = Matrix4::from_angle_y(Deg(45.0)) * transform;
        transform = Matrix4::from_angle_x(Deg(30.0)) * transform;
        transform = Matrix4::from_scale(size) * transform;
        transform = Matrix4::from_translation(position) * transform;
        orthographic_shader.uniform_matrix4f("transform", &transform);

        match item {
            Item::BlockItem(block, _amt) => {
                //This probably isn't the most efficient way to display a block
                //icon but it works and I only really need to display a few of
                //these so it should be fine
                chunk.set_block_relative(1, 1, 1, *block);
                let mut vert_data = vec![];
                let adj_chunks = [ None; 6 ];
                add_block_vertices(&chunk, adj_chunks, (1, 1, 1), &mut vert_data);
                add_block_vertices_transparent(&chunk, adj_chunks, (1, 1, 1), &mut vert_data);
                add_block_vertices_fluid(&chunk, adj_chunks, (1, 1, 1), &mut vert_data);
                let vao = ChunkVao::generate_new(&vert_data);
                vao.draw();
                vao.delete();
            }
            Item::EmptyItem => {}
        }
    }

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }
}
