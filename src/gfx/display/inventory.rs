use crate::{
    game::{
        assets::models::draw_elements,
        inventory::{get_item_atlas_id, Inventory, Item},
        inventory_screen::mouse_selecting_slot,
        Game, GameMode,
    },
    gfx::{
        buildchunk::{
            add_block_vertices, add_block_vertices_flat, add_block_vertices_fluid,
            add_block_vertices_transparent, get_indices,
        },
        chunktable::ChunkVao,
    },
    voxel::{Block, Chunk},
};
use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};

const DIGIT_W: f32 = 6.3;
const DIGIT_H: f32 = DIGIT_W * 16.0 / 9.0;

pub fn get_block_item_transform(size: f32, position: Vector3<f32>, block: Block) -> Matrix4<f32> {
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
    //If it's a slab that is vertical, attempt to center it
    if block.shape() == 1 && block.orientation() % 3 != 0 {
        transform = Matrix4::from_translation(Vector3::new(4.0, -4.0, 0.0)) * transform;
    }
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

    if vert_data.is_empty() {
        return;
    }

    let face_count = vert_data.len() / (7 * 4);
    let vao = ChunkVao::generate_new(&vert_data, &get_indices(face_count), 7);
    vao.draw();
    vao.delete();
}

pub fn display_block_item_flat3d(chunk: &mut Chunk, block: Block) {
    chunk.set_block_relative(1, 1, 1, block);
    let mut vert_data = vec![];
    add_block_vertices_flat(chunk, (1, 1, 1), &mut vert_data);

    if vert_data.is_empty() {
        return;
    }

    let face_count = vert_data.len() / (7 * 4);
    let vao = ChunkVao::generate_new(&vert_data, &get_indices(face_count), 7);
    vao.draw_instanced(8);
    vao.delete();
}

//Assumes that we are using the "icon2d" shader
pub fn display_u8(gamestate: &Game, x: f32, y: f32, w: f32, h: f32, num: u8) {
    let shader2d = gamestate.shaders.get("icon2d");
    let quad = gamestate.models.bind("quad2d");
    let mut digits = num;
    let mut posx = x;
    while digits > 0 {
        let digit = digits % 10;
        digits -= digit;
        digits /= 10;

        shader2d.uniform_vec2f("texoffset", 0.1 * digit as f32, 0.0);
        let mut transform = Matrix4::identity();
        transform = Matrix4::from_nonuniform_scale(w, h, 1.0) * transform;
        transform = Matrix4::from_translation(Vector3::new(posx, y, 0.0)) * transform;
        shader2d.uniform_matrix4f("transform", &transform);
        draw_elements(quad.clone());
        posx -= w * 2.0;
    }
}

const HOTBAR_SIZE: f32 = 32.0; //In pixels

fn display_hotbar_blocks(gamestate: &Game, w: i32, h: i32) {
    let hotbar_sz = gamestate.player.hotbar.items.len();
    gamestate.textures.bind("blocks");
    gamestate.shaders.use_program("orthographic");
    let orthographic_shader = gamestate.shaders.get("orthographic");
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    let orthographic = cgmath::ortho(-half_w, half_w, -half_h, half_h, 0.01, 100.0);
    orthographic_shader.uniform_matrix4f("screen", &orthographic);
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

        if let Item::Block(block, _amt) = item {
            let transform = get_block_item_transform(size, position, *block);
            orthographic_shader.uniform_matrix4f("transform", &transform);
            display_block_item(&mut chunk, *block);
        }
    }
}

fn display_hotbar_sprite_items(gamestate: &Game, w: i32, h: i32) {
    let hotbar_sz = gamestate.player.hotbar.items.len();
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    gamestate.textures.bind("items");
    gamestate.shaders.use_program("icon2d");
    let icon2d = gamestate.shaders.get("icon2d");
    icon2d.uniform_matrix4f("screen", &screen_mat);
    icon2d.uniform_float("alpha", 1.0);
    icon2d.uniform_vec2f("texscale", ITEM_TEX_SCALE, ITEM_TEX_SCALE);
    let quad = gamestate.models.bind("quad2d");
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

        let id = get_item_atlas_id(*item);
        match item {
            Item::Sprite(..) | Item::Food(..) | Item::Tool(..) | Item::Bucket(..) => {
                let ix = id % ITEM_TEX_SIZE;
                let iy = id / ITEM_TEX_SIZE;
                let tx = ix as f32 * ITEM_TEX_SCALE;
                let ty = iy as f32 * ITEM_TEX_SCALE;

                icon2d.uniform_vec2f("texoffset", tx, ty);
                let mut transform = Matrix4::identity();
                transform = Matrix4::from_scale(size * 14.0 / 16.0) * transform;
                transform = Matrix4::from_translation(position) * transform;
                icon2d.uniform_matrix4f("transform", &transform);
                draw_elements(quad.clone());
            }
            _ => {}
        }
    }
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
        gl::Enable(gl::DEPTH_TEST);
    }

    //Display blocks
    display_hotbar_blocks(gamestate, w, h);

    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    //Display items
    display_hotbar_sprite_items(gamestate, w, h);

    //Display number of items
    gamestate.shaders.use_program("icon2d");
    let shader2d = gamestate.shaders.get("icon2d");
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_vec2f("texscale", 1.0 / 10.0, 1.0);
    shader2d.uniform_float("alpha", 1.0);
    gamestate.textures.bind("digits");
    for (i, item) in gamestate.player.hotbar.items.iter().copied().enumerate() {
        let x = i as f32 * HOTBAR_SIZE * 2.0 - HOTBAR_SIZE * hotbar_sz as f32 + HOTBAR_SIZE * 1.5;
        let y = -h as f32 / 2.0 + HOTBAR_SIZE * 0.6;

        match item {
            Item::Sprite(_, amt) | Item::Block(_, amt) => {
                if amt <= 1 {
                    continue;
                }
                display_u8(gamestate, x, y, DIGIT_W, DIGIT_H, amt);
            }
            _ => {}
        }
    }

    let shader2d = gamestate.shaders.use_program("2d");
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_float("alpha", 1.0);
    gamestate.textures.bind("black_bg");
    for (i, item) in gamestate.player.hotbar.items.iter().copied().enumerate() {
        let x = i as f32 * HOTBAR_SIZE * 2.0 - HOTBAR_SIZE * hotbar_sz as f32 + HOTBAR_SIZE * 1.0;
        let y = -h as f32 / 2.0 + HOTBAR_SIZE * 0.35 + 1.0;

        if let Item::Tool(_, info) = item {
            if info.durability >= info.max_durability {
                continue;
            }
            let mut transform = Matrix4::identity();
            transform = Matrix4::from_nonuniform_scale(HOTBAR_SIZE - 10.0, 2.0, 1.0) * transform;
            transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
            shader2d.uniform_matrix4f("transform", &transform);
            draw_elements(quad.clone());
        }
    }

    let shader2d = gamestate.shaders.use_program("gradientquad");
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_vec4f("start", 1.0, 0.0, 0.0, 1.0);
    shader2d.uniform_vec4f("mid", 1.0, 1.0, 0.0, 1.0);
    shader2d.uniform_vec4f("end", 0.0, 1.0, 0.0, 1.0);

    for (i, item) in gamestate.player.hotbar.items.iter().copied().enumerate() {
        let x = i as f32 * HOTBAR_SIZE * 2.0 - HOTBAR_SIZE * hotbar_sz as f32 + HOTBAR_SIZE * 1.0;
        let y = -h as f32 / 2.0 + HOTBAR_SIZE * 0.35 + 1.0;

        if let Item::Tool(_, info) = item {
            if info.durability >= info.max_durability {
                continue;
            }
            let perc = info.durability as f32 / info.max_durability as f32;
            shader2d.uniform_float("perc", perc);
            let mut transform = Matrix4::identity();
            let width = (HOTBAR_SIZE - 10.0) * perc;
            let offset = (1.0 - perc) * (HOTBAR_SIZE - 10.0);
            let pos = Vector3::new(x - offset, y, 0.0);
            transform = Matrix4::from_nonuniform_scale(width, 2.0, 1.0) * transform;
            transform = Matrix4::from_translation(pos) * transform;
            shader2d.uniform_matrix4f("transform", &transform);
            draw_elements(quad.clone());
        }
    }

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }
}

fn get_slot_transform(x: f32, y: f32, sz: f32, mousex: f32, mousey: f32) -> Matrix4<f32> {
    let mut transform = Matrix4::identity();
    let slot_sz = if mouse_selecting_slot(x, y, sz, mousex, mousey) {
        sz * 17.0 / 15.0
    } else {
        sz
    };
    transform = Matrix4::from_scale(slot_sz) * transform;
    transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
    transform
}

fn display_inventory_slots(
    gamestate: &Game,
    inventory: &Inventory,
    topleft: (f32, f32),
    sz: f32,
    mousepos: (f32, f32),
) {
    gamestate.textures.bind("hud_icons");
    let (leftx, topy) = topleft;
    let (mousex, mousey) = mousepos;
    let quad = gamestate.models.bind("quad2d");
    let shader = gamestate.shaders.get("icon2d");
    shader.uniform_vec2f("texscale", 1.0 / 4.0, 1.0 / 4.0);
    shader.uniform_vec2f("texoffset", 0.25, 0.5);
    let step = (sz / 2.0 + 2.0) * 4.0;
    for iy in 0..inventory.h() {
        for ix in 0..inventory.w() {
            let x = leftx + ix as f32 * step;
            let y = topy - step * iy as f32;
            let transform = get_slot_transform(x, y, sz, mousex, mousey);
            shader.uniform_matrix4f("transform", &transform);
            draw_elements(quad.clone());
        }
    }
}

fn display_inventory_blocks(
    gamestate: &Game,
    inventory: &Inventory,
    topleft: (f32, f32),
    sz: f32,
    win_dimensions: (i32, i32),
) {
    gamestate.textures.bind("blocks");
    gamestate.shaders.use_program("orthographic");
    let orthographic_shader = gamestate.shaders.get("orthographic");
    let (w, h) = win_dimensions;
    let half_w = w as f32 / 2.0;
    let half_h = h as f32 / 2.0;
    let orthographic = cgmath::ortho(-half_w, half_w, -half_h, half_h, 0.01, 100.0);
    orthographic_shader.uniform_matrix4f("screen", &orthographic);
    orthographic_shader.uniform_vec3f("offset", -1.5, -1.5, -1.5);

    let (leftx, topy) = topleft;
    let step = (sz / 2.0 + 2.0) * 4.0;
    let mut chunk = Chunk::new(0, 0, 0);
    for iy in 0..inventory.h() {
        for ix in 0..inventory.w() {
            let x = leftx + ix as f32 * step;
            let y = topy - step * iy as f32;
            let position = Vector3::new(x, y, 0.0);

            if let Item::Block(block, _amt) = inventory.get_item(ix, iy) {
                let transform = get_block_item_transform(sz, position, block);
                orthographic_shader.uniform_matrix4f("transform", &transform);
                display_block_item(&mut chunk, block);
            }
        }
    }
}

fn display_inventory_numbers(
    gamestate: &Game,
    inventory: &Inventory,
    topleft: (f32, f32),
    sz: f32,
    win_dimensions: (i32, i32),
) {
    let (w, h) = win_dimensions;
    gamestate.textures.bind("digits");
    gamestate.shaders.use_program("icon2d");
    let shader2d = gamestate.shaders.get("icon2d");
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_vec2f("texscale", 1.0 / 10.0, 1.0);
    shader2d.uniform_float("alpha", 1.0);

    let (leftx, topy) = topleft;
    let step = (sz / 2.0 + 2.0) * 4.0;
    for iy in 0..inventory.h() {
        for ix in 0..inventory.w() {
            let x = leftx + ix as f32 * step + step / 4.0;
            let y = topy - step * iy as f32 - step / 4.0;

            match inventory.get_item(ix, iy) {
                Item::Block(_block, amt) => {
                    if amt <= 1 {
                        continue;
                    }
                    display_u8(gamestate, x, y, DIGIT_W, DIGIT_H, amt);
                }
                Item::Sprite(_id, amt) => {
                    if amt <= 1 {
                        continue;
                    }
                    display_u8(gamestate, x, y, DIGIT_W, DIGIT_H, amt);
                }
                _ => {}
            }
        }
    }
}

fn display_tool_durability(
    gamestate: &Game,
    inventory: &Inventory,
    topleft: (f32, f32),
    sz: f32,
    win_dimensions: (i32, i32),
) {
    let quad = gamestate.models.bind("quad2d");
    let (w, h) = win_dimensions;
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    let (leftx, topy) = topleft;
    let step = (sz / 2.0 + 2.0) * 4.0;

    let shader2d = gamestate.shaders.use_program("2d");
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_float("alpha", 1.0);
    gamestate.textures.bind("black_bg");
    for iy in 0..inventory.h() {
        for ix in 0..inventory.w() {
            let x = leftx + ix as f32 * step;
            let y = topy - step * iy as f32 - sz + 6.0;

            if let Item::Tool(_, info) = inventory.get_item(ix, iy) {
                if info.durability >= info.max_durability {
                    continue;
                }
                let mut transform = Matrix4::identity();
                transform = Matrix4::from_nonuniform_scale(sz - 6.0, 2.0, 1.0) * transform;
                transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
                shader2d.uniform_matrix4f("transform", &transform);
                draw_elements(quad.clone());
            }
        }
    }

    let shader2d = gamestate.shaders.use_program("gradientquad");
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_vec4f("start", 1.0, 0.0, 0.0, 1.0);
    shader2d.uniform_vec4f("mid", 1.0, 1.0, 0.0, 1.0);
    shader2d.uniform_vec4f("end", 0.0, 1.0, 0.0, 1.0);
    for iy in 0..inventory.h() {
        for ix in 0..inventory.w() {
            let x = leftx + ix as f32 * step;
            let y = topy - step * iy as f32 - sz + 6.0;

            if let Item::Tool(_, info) = inventory.get_item(ix, iy) {
                if info.durability >= info.max_durability {
                    continue;
                }
                let perc = info.durability as f32 / info.max_durability as f32;
                shader2d.uniform_float("perc", perc);
                let mut transform = Matrix4::identity();
                let width = (sz - 6.0) * perc;
                let offset = (1.0 - perc) * (sz - 6.0);
                let pos = Vector3::new(x - offset, y, 0.0);
                transform = Matrix4::from_nonuniform_scale(width, 2.0, 1.0) * transform;
                transform = Matrix4::from_translation(pos) * transform;
                shader2d.uniform_matrix4f("transform", &transform);
                draw_elements(quad.clone());
            }
        }
    }
}

pub const ITEM_TEX_SIZE: u16 = 16;
pub const ITEM_TEX_SCALE: f32 = 1.0 / (ITEM_TEX_SIZE as f32);

fn display_inventory_sprite_items(
    gamestate: &Game,
    inventory: &Inventory,
    topleft: (f32, f32),
    sz: f32,
    win_dimensions: (i32, i32),
) {
    let (w, h) = win_dimensions;
    gamestate.textures.bind("items");
    gamestate.shaders.use_program("icon2d");
    let shader2d = gamestate.shaders.get("icon2d");
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_vec2f("texscale", ITEM_TEX_SCALE, ITEM_TEX_SCALE);
    shader2d.uniform_float("alpha", 1.0);

    let (leftx, topy) = topleft;
    let step = (sz / 2.0 + 2.0) * 4.0;
    for iy in 0..inventory.h() {
        for ix in 0..inventory.w() {
            let x = leftx + ix as f32 * step + step / 4.0 - sz / 2.0 - 2.0;
            let y = topy - step * iy as f32 - step / 4.0 + sz / 2.0;

            let id = get_item_atlas_id(inventory.get_item(ix, iy));
            match inventory.get_item(ix, iy) {
                Item::Sprite(..) | Item::Tool(..) | Item::Food(..) | Item::Bucket(..) => {
                    let ix = id % ITEM_TEX_SIZE;
                    let iy = id / ITEM_TEX_SIZE;
                    let tx = ix as f32 * ITEM_TEX_SCALE;
                    let ty = iy as f32 * ITEM_TEX_SCALE;

                    shader2d.uniform_vec2f("texoffset", tx, ty);
                    let mut transform = Matrix4::identity();
                    transform = Matrix4::from_scale(sz * 14.0 / 16.0) * transform;
                    transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
                    shader2d.uniform_matrix4f("transform", &transform);
                    let quad = gamestate.models.bind("quad2d");
                    draw_elements(quad);
                }
                _ => {}
            }
        }
    }
}

fn display_inventory_items(
    gamestate: &Game,
    inventory: &Inventory,
    topleft: (f32, f32),
    sz: f32,
    w: i32,
    h: i32,
) {
    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }

    //Display block items
    display_inventory_blocks(gamestate, inventory, topleft, sz, (w, h));

    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::Disable(gl::DEPTH_TEST);
    }

    //Display sprite items
    display_inventory_sprite_items(gamestate, inventory, topleft, sz, (w, h));

    //Display durability bars
    display_tool_durability(gamestate, inventory, topleft, sz, (w, h));
    //Display inventory number icons
    display_inventory_numbers(gamestate, inventory, topleft, sz, (w, h));
}

//Assumes that the screen matrix has already been set
fn display_icon(gamestate: &Game, x: f32, y: f32, tcx: f32, tcy: f32) {
    let quad = gamestate.models.bind("quad2d");
    let shader2d = gamestate.shaders.use_program("icon2d");
    gamestate.textures.bind("hud_icons");
    shader2d.uniform_vec2f("texscale", 1.0 / 4.0, 1.0 / 4.0);
    shader2d.uniform_vec2f("texoffset", tcx, tcy);
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_scale(30.0) * transform;
    transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad);
}

fn display_arrow(gamestate: &Game, x: f32, y: f32) {
    display_icon(gamestate, x, y, 0.75, 0.5);
}

fn display_fire_outline(gamestate: &Game, x: f32, y: f32) {
    display_icon(gamestate, x, y, 0.25, 0.75);
}

fn display_fire(gamestate: &Game, x: f32, y: f32, perc: f32) {
    let shader2d = gamestate.shaders.use_program("icon2d");
    shader2d.uniform_vec2f("texperc", 1.0, perc);
    display_icon(gamestate, x, y, 0.5, 0.75);
}

fn display_progress(gamestate: &Game, x: f32, y: f32, perc: f32) {
    let shader2d = gamestate.shaders.use_program("icon2d");
    shader2d.uniform_vec2f("texperc", perc, 1.0);
    display_icon(gamestate, x, y, 0.0, 0.75);
}

fn display_single_slot(
    gamestate: &Game,
    pos: (f32, f32),
    mousepos: (f32, f32),
    w: i32,
    h: i32,
    item: Item,
) {
    let mut inventory = Inventory::empty_with_sz(1, 1);
    inventory.set_item(0, 0, item);
    display_inventory(gamestate, &inventory, pos, mousepos, w, h);
}

pub const SLOT_SZ: f32 = 30.0;
pub const BUFFER: f32 = 4.0;
const STEP: f32 = (SLOT_SZ + BUFFER) * 2.0;

fn display_inventory(
    gamestate: &Game,
    inventory: &Inventory,
    pos: (f32, f32),
    mousepos: (f32, f32),
    w: i32,
    h: i32,
) {
    //Display main inventory
    display_inventory_slots(gamestate, inventory, pos, SLOT_SZ, mousepos);
    display_inventory_items(gamestate, inventory, pos, SLOT_SZ, w, h);
}

const BOTTOM_Y: f32 = -230.0;
pub const MAIN_INVENTORY_POS: (f32, f32) = (-4.0 * STEP, BOTTOM_Y + 15.0 + STEP * 3.0);
pub const CHEST_INVENTORY_POS: (f32, f32) =
    (-4.0 * STEP, BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 6.0 + SLOT_SZ);
pub const HOTBAR_POS: (f32, f32) = (-4.0 * STEP, BOTTOM_Y);
pub const CRAFTING_GRID_POS: (f32, f32) =
    (-2.0 * STEP, BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 6.0 + SLOT_SZ);
pub const OUTPUT_POS: (f32, f32) = (2.0 * STEP, BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 5.0 + SLOT_SZ);
pub const DESTROY_POS: (f32, f32) = (-4.0 * STEP, BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 5.0 + SLOT_SZ);

pub const FURNACE_FUEL_POS: (f32, f32) = (-STEP, BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 4.0 + SLOT_SZ);
pub const FURNACE_INPUT_POS: (f32, f32) = (-STEP, BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 6.0 + SLOT_SZ);
pub const FURNACE_OUTPUT_POS: (f32, f32) = (STEP, BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 5.0 + SLOT_SZ);

pub fn display_inventory_screen(gamestate: &Game, w: i32, h: i32, mousepos: (f32, f32)) {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    gamestate.textures.bind("hud_icons");
    let shader2d = gamestate.shaders.use_program("icon2d");
    let quad = gamestate.models.bind("quad2d");

    //Set screen matrix
    let screen_mat = Matrix4::from_nonuniform_scale(2.0 / w as f32, 2.0 / h as f32, 1.0);
    shader2d.uniform_matrix4f("screen", &screen_mat);
    shader2d.uniform_vec2f("texscale", 1.0 / 4.0, 1.0 / 4.0);

    //Display background
    shader2d.uniform_vec2f("texoffset", 0.5, 0.5);
    shader2d.uniform_float("alpha", 1.0);
    let transform = Matrix4::from_nonuniform_scale(9.0 * 34.0 + 20.0, 280.0, 1.0);
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());

    let main_inventory = &gamestate.player.inventory;
    let hotbar = Inventory::from_hotbar(&gamestate.player.hotbar);
    let crafting_grid = &gamestate.player.crafting_grid;

    let mut output_slot = Inventory::empty_with_sz(1, 1);
    //Set output item for the output slot
    let output = gamestate
        .recipe_table
        .get_output(&gamestate.player.crafting_grid)
        .unwrap_or(Item::Empty);
    output_slot.set_item(0, 0, output);

    let mut destroy_slot = Inventory::empty_with_sz(1, 1);
    //Trash can icon
    destroy_slot.set_item(0, 0, Item::Sprite(255, 1));

    //Main inventory
    display_inventory(
        gamestate,
        main_inventory,
        MAIN_INVENTORY_POS,
        mousepos,
        w,
        h,
    );
    //Hotbar
    display_inventory(gamestate, &hotbar, HOTBAR_POS, mousepos, w, h);
    if gamestate.player.opened_block.is_none() {
        //Crafting grid
        display_inventory(gamestate, crafting_grid, CRAFTING_GRID_POS, mousepos, w, h);
        //Output slot
        display_inventory(gamestate, &output_slot, OUTPUT_POS, mousepos, w, h);

        //Destroy item slot
        if gamestate.game_mode() == GameMode::Creative {
            display_inventory(gamestate, &destroy_slot, DESTROY_POS, mousepos, w, h);
        }

        let arrow_x = STEP;
        let arrow_y = BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 5.0 + SLOT_SZ;
        display_arrow(gamestate, arrow_x, arrow_y);
    } else {
        match gamestate.player.opened_block_id {
            //Chest
            37 => {
                display_inventory(
                    gamestate,
                    &gamestate.player.open_block_data.inventory,
                    CHEST_INVENTORY_POS,
                    mousepos,
                    w,
                    h,
                );
            }
            //Furnace
            40 | 70 => {
                let arrow_y = BOTTOM_Y + SLOT_SZ / 2.0 + STEP * 5.0 + SLOT_SZ;
                display_arrow(gamestate, 0.0, arrow_y);
                display_fire_outline(gamestate, -STEP, arrow_y);

                //Input slot
                let input = gamestate.player.open_block_data.get_furnace_input();
                display_single_slot(gamestate, FURNACE_INPUT_POS, mousepos, w, h, input);
                //Fuel slot
                let fuel = gamestate.player.open_block_data.get_furnace_fuel();
                display_single_slot(gamestate, FURNACE_FUEL_POS, mousepos, w, h, fuel);
                //Output slot
                let output = gamestate.player.open_block_data.get_furnace_output();
                display_single_slot(gamestate, FURNACE_OUTPUT_POS, mousepos, w, h, output);

                let fuel = gamestate
                    .player
                    .open_block_data
                    .get_float("fuel")
                    .unwrap_or(0.0);
                let maxfuel = gamestate
                    .player
                    .open_block_data
                    .get_float("maxfuel")
                    .unwrap_or(0.0);
                let fuelperc = if maxfuel > 0.0 { fuel / maxfuel } else { 0.0 };
                let progress = gamestate
                    .player
                    .open_block_data
                    .get_float("progress")
                    .unwrap_or(0.0);
                display_fire(gamestate, -STEP, arrow_y, fuelperc);
                display_progress(gamestate, 0.0, arrow_y, progress);
                //reset texperc in icon2d to be (1.0, 1.0)
                shader2d.uniform_vec2f("texperc", 1.0, 1.0);
            }
            _ => {}
        }
    }

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }
}

pub fn display_mouse_item(gamestate: &Game, mousepos: (f32, f32), w: i32, h: i32) {
    unsafe {
        gl::Clear(gl::DEPTH_BUFFER_BIT);
    }

    if !gamestate.player.mouse_item.is_empty() {
        let mut mouse_item = Inventory::empty_with_sz(1, 1);
        mouse_item.set_item(0, 0, gamestate.player.mouse_item);
        display_inventory_items(gamestate, &mouse_item, mousepos, 30.0, w, h);
    }

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }
}
