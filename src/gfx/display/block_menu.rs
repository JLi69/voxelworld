use crate::{
    game::{
        assets::models::draw_elements,
        block_menu::{get_positions, get_selected, ICON_SIZE},
        input::convert_mouse_pos,
        Game,
    },
    gfx::{
        buildchunk::{
            add_block_vertices, add_block_vertices_fluid, add_block_vertices_transparent,
        },
        chunktable::ChunkVao,
    },
    voxel::{Block, Chunk},
};
use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};

pub const BLOCK_MENU_WIDTH: f32 = 384.0;
pub const BLOCK_MENU_HEIGHT: f32 = 224.0;

pub fn display_block_menu(gamestate: &Game, w: i32, h: i32, mousex: i32, mousey: i32) {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    gamestate.textures.bind("block_menu_background");
    gamestate.shaders.use_program("2d");
    let shader2d = gamestate.shaders.get("2d");
    let quad = gamestate.models.bind("quad2d");

    //Draw background
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    let transform = Matrix4::from_nonuniform_scale(400.0, 240.0, 0.0);
    shader2d.uniform_matrix4f("transform", &transform);
    shader2d.uniform_float("alpha", 0.5);
    draw_elements(quad);

    unsafe {
        gl::Enable(gl::CULL_FACE);
    }

    //Display blocks
    //Probably not the most efficient way to do this but it shouldn't have
    //too much of a bad effect on performance
    let (mousex_f32, mousey_f32) = convert_mouse_pos(mousex, mousey, w, h);
    gamestate.textures.bind("blocks");
    gamestate.shaders.use_program("orthographic");
    let orthographic_shader = gamestate.shaders.get("orthographic");
    orthographic_shader.uniform_matrix4f("screen", &screen_mat);
    orthographic_shader.uniform_vec3f("offset", -1.5, -1.5, -1.5);
    let mut chunk = Chunk::new(0, 0, 0);
    let block_menu = get_positions(gamestate, -BLOCK_MENU_WIDTH, BLOCK_MENU_HEIGHT);
    let selected = get_selected(&block_menu, mousex_f32, mousey_f32);
    for (i, (block, pos)) in block_menu.iter().enumerate() {
        let position = Vector3::new(pos.x, pos.y, 0.0);

        //Check if the cursor is inside
        let size = if selected.unwrap_or(block_menu.len()) == i {
            ICON_SIZE * 20.0 / 16.0
        } else {
            ICON_SIZE
        };

        let mut transform = Matrix4::identity();
        transform = Matrix4::from_angle_y(Deg(45.0)) * transform;
        transform = Matrix4::from_angle_x(Deg(30.0)) * transform;
        transform = Matrix4::from_scale(size) * transform;
        transform = Matrix4::from_translation(position) * transform;
        orthographic_shader.uniform_matrix4f("transform", &transform);

        chunk.set_block_relative(1, 1, 1, Block::new_id(*block));
        let mut vert_data = vec![];
        let adj_chunks = [None; 6];
        add_block_vertices(&chunk, adj_chunks, (1, 1, 1), &mut vert_data);
        add_block_vertices_transparent(&chunk, adj_chunks, (1, 1, 1), &mut vert_data);
        add_block_vertices_fluid(&chunk, adj_chunks, (1, 1, 1), &mut vert_data);
        let vao = ChunkVao::generate_new(&vert_data);
        vao.draw();
        vao.delete();
    }

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }
}
