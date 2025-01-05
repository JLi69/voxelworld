use crate::{
    game::{assets::models::draw_elements, Game, inventory::Item},
    gfx::{
        buildchunk::{
            add_block_vertices, 
            add_block_vertices_fluid,
            add_block_vertices_transparent,
            add_block_vertices_flat,
        },
        chunktable::ChunkVao,
    },
    voxel::{Chunk, Block},
};
use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};

pub fn get_block_item_transform(
    size: f32,
    position: Vector3<f32>,
    block: Block,
) -> Matrix4<f32> {
    let mut transform = Matrix4::identity();
    if !block.is_flat_item() {
        transform = Matrix4::from_angle_y(Deg(45.0)) * transform;
        transform = Matrix4::from_angle_x(Deg(30.0)) * transform;
    } else {
        transform = Matrix4::from_angle_y(Deg(90.0)) * transform;
        transform = Matrix4::from_scale((2.0f32).sqrt()) * transform;
    }
    transform = Matrix4::from_scale(size) * transform;
    transform = Matrix4::from_translation(position) * transform;
    transform
}

pub fn display_block_item(chunk: &mut Chunk, block: Block) {
    //This probably isn't the most efficient way to display a block
    //icon but it works and I only really need to display a few of
    //these so it should be fine
    chunk.set_block_relative(1, 1, 1, block);
    let mut vert_data = vec![];
    let adj_chunks = [None; 6];
    if block.is_flat_item() {
        add_block_vertices_flat(chunk, (1, 1, 1), &mut vert_data);
    } else {
        add_block_vertices(chunk, adj_chunks, (1, 1, 1), &mut vert_data);
        add_block_vertices_transparent(chunk, adj_chunks, (1, 1, 1), &mut vert_data);
        add_block_vertices_fluid(chunk, adj_chunks, (1, 1, 1), &mut vert_data);
    }
    let vao = ChunkVao::generate_new(&vert_data);
    vao.draw();
    vao.delete();
}

pub fn display_hotbar(gamestate: &Game, w: i32, h: i32) {
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
            0.0,
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
            0.0,
        );

        let size = if i == gamestate.player.hotbar.selected {
            HOTBAR_SIZE * 18.0 / 16.0 * 14.0 / 16.0
        } else {
            HOTBAR_SIZE * 14.0 / 16.0
        }; 

        match item {
            Item::BlockItem(block, _amt) => {
                let transform = get_block_item_transform(size, position, *block);
                orthographic_shader.uniform_matrix4f("transform", &transform);
                display_block_item(&mut chunk, *block);
            }
            Item::EmptyItem => {}
        }
    }

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
    }
}
