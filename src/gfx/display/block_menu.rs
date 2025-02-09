use crate::{
    game::{
        assets::models::draw_elements,
        block_menu::{get_positions, get_selected, get_shape_icon_positions, ICON_SIZE},
        input::convert_mouse_pos,
        set_block_shape, BlockMenuShape, Game,
    },
    voxel::{Block, Chunk},
};
use cgmath::{Matrix4, Vector3};

use super::inventory::{display_block_item, get_block_item_transform};

pub const BLOCK_MENU_WIDTH: f32 = 384.0;
pub const BLOCK_MENU_HEIGHT: f32 = 224.0;

fn display_shape_icons(gamestate: &Game, w: i32, h: i32, mousex: i32, mousey: i32) {
    let shape_icons = get_shape_icon_positions(BLOCK_MENU_WIDTH, -BLOCK_MENU_HEIGHT);
    gamestate.shaders.use_program("2d");
    let shader2d = gamestate.shaders.get("2d");
    let quad = gamestate.models.bind("quad2d");

    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_float("alpha", 1.0);

    let (mousex_f32, mousey_f32) = convert_mouse_pos(mousex, mousey, w, h);
    let selected = get_selected(&shape_icons, mousex_f32, mousey_f32).unwrap_or(shape_icons.len());
    for (i, (shape, pos)) in shape_icons.iter().enumerate() {
        match shape {
            BlockMenuShape::Normal => gamestate.textures.bind("full_block_icon"),
            BlockMenuShape::Slab => gamestate.textures.bind("half_slab_icon"),
            BlockMenuShape::Stair => gamestate.textures.bind("stair_icon"),
        }

        let sz = if selected == i {
            ICON_SIZE
        } else {
            ICON_SIZE * 0.8
        };
        let transform = Matrix4::from_translation(Vector3::new(pos.x, pos.y, 0.0))
            * Matrix4::from_nonuniform_scale(sz, sz, 0.0);
        shader2d.uniform_matrix4f("transform", &transform);
        draw_elements(quad.clone());
    }
}

pub fn display_block_menu(gamestate: &Game, w: i32, h: i32, mousex: i32, mousey: i32) {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    gamestate.textures.bind("black_bg");
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

    display_shape_icons(gamestate, w, h, mousex, mousey);

    unsafe {
        gl::Enable(gl::CULL_FACE);
    }

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }

    //Display blocks
    //Probably not the most efficient way to do this but it shouldn't have
    //too much of a bad effect on performance
    let (mousex_f32, mousey_f32) = convert_mouse_pos(mousex, mousey, w, h);
    gamestate.textures.bind("blocks");
    gamestate.shaders.use_program("orthographic");
    let orthographic_shader = gamestate.shaders.get("orthographic");
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    let orthographic = cgmath::ortho(-half_w, half_w, -half_h, half_h, 0.01, 100.0);
    orthographic_shader.uniform_matrix4f("screen", &orthographic);
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

        let transform = get_block_item_transform(size, position, Block::new_id(*block));
        orthographic_shader.uniform_matrix4f("transform", &transform);
        let mut block_item = Block::new_id(*block);
        if !block_item.is_flat_item() {
            set_block_shape(&mut block_item, gamestate.get_block_menu_shape());
        }
        display_block_item(&mut chunk, block_item);
    }
}
