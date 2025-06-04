use crate::{
    game::{
        assets::models::draw_elements,
        inventory::{Inventory, Item},
        inventory_screen::mouse_selecting_slot,
        Game,
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
        gl::Enable(gl::DEPTH_TEST);
    }

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
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

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
            Item::BlockItem(_block, amt) => {
                if amt <= 1 {
                    continue;
                }
                display_u8(gamestate, x, y, DIGIT_W, DIGIT_H, amt);
            }
            Item::EmptyItem => {}
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
    shaderid: &str,
) {
    let (leftx, topy) = topleft;
    let (mousex, mousey) = mousepos;
    let quad = gamestate.models.bind("quad2d");
    let shader = gamestate.shaders.get(shaderid);
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
    shaderid: &str,
) {
    gamestate.textures.bind("blocks");
    gamestate.shaders.use_program(shaderid);
    let orthographic_shader = gamestate.shaders.get(shaderid);
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

            if let Item::BlockItem(block, _amt) = inventory.get_item(ix, iy) {
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
                Item::BlockItem(_block, amt) => {
                    if amt <= 1 {
                        continue;
                    }
                    display_u8(gamestate, x, y, DIGIT_W, DIGIT_H, amt);
                }
                Item::EmptyItem => {}
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
    display_inventory_blocks(gamestate, inventory, topleft, sz, (w, h), "orthographic");

    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::Disable(gl::DEPTH_TEST);
    }

    //Display inventory number icons
    display_inventory_numbers(gamestate, inventory, topleft, sz, (w, h));
}

const BOTTOM_Y: f32 = -230.0;
pub const MAIN_INVENTORY_POS: (f32, f32) = (-4.0 * 68.0, BOTTOM_Y + 15.0 + 68.0 * 3.0);
pub const HOTBAR_POS: (f32, f32) = (-4.0 * 68.0, BOTTOM_Y);
pub const CRAFTING_GRID_POS: (f32, f32) = (-2.0 * 68.0, BOTTOM_Y + 15.0 + 68.0 * 6.0 + 30.0);
pub const OUTPUT_POS: (f32, f32) = (2.0 * 68.0, BOTTOM_Y + 15.0 + 68.0 * 5.0 + 30.0);

pub fn display_inventory_screen(gamestate: &Game, w: i32, h: i32, mousepos: (f32, f32)) {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    gamestate.textures.bind("hud_icons");
    gamestate.shaders.use_program("icon2d");
    let shader2d = gamestate.shaders.get("icon2d");
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

    //Display item slots
    shader2d.uniform_vec2f("texoffset", 0.25, 0.5);

    //Display main inventory
    display_inventory_slots(
        gamestate,
        &gamestate.player.inventory,
        MAIN_INVENTORY_POS,
        30.0,
        mousepos,
        "icon2d",
    );

    //Hotbar
    let hotbar = Inventory::from_hotbar(&gamestate.player.hotbar);
    display_inventory_slots(gamestate, &hotbar, HOTBAR_POS, 30.0, mousepos, "icon2d");

    //Draw crafting grid
    display_inventory_slots(
        gamestate,
        &gamestate.player.crafting_grid,
        CRAFTING_GRID_POS,
        30.0,
        mousepos,
        "icon2d",
    );

    //Display output slot
    let output_slot = Inventory::empty_with_sz(1, 1);
    display_inventory_slots(
        gamestate,
        &output_slot,
        OUTPUT_POS,
        30.0,
        mousepos,
        "icon2d",
    );

    shader2d.uniform_vec2f("texoffset", 0.75, 0.5);
    let mut transform = Matrix4::identity();
    transform = Matrix4::from_scale(30.0) * transform;
    let x = 68.0;
    let y = BOTTOM_Y + 15.0 + 68.0 * 5.0 + 30.0;
    transform = Matrix4::from_translation(Vector3::new(x, y, 0.0)) * transform;
    shader2d.uniform_matrix4f("transform", &transform);
    draw_elements(quad.clone());

    //Display items
    display_inventory_items(gamestate, &hotbar, HOTBAR_POS, 30.0, w, h);
    display_inventory_items(gamestate, &output_slot, OUTPUT_POS, 30.0, w, h);
    display_inventory_items(
        gamestate,
        &gamestate.player.inventory,
        MAIN_INVENTORY_POS,
        30.0,
        w,
        h,
    );
    display_inventory_items(
        gamestate,
        &gamestate.player.crafting_grid,
        CRAFTING_GRID_POS,
        30.0,
        w,
        h,
    );

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }
}

pub fn display_mouse_item(gamestate: &Game, mousepos: (f32, f32), w: i32, h: i32) {
    unsafe {
        gl::Clear(gl::DEPTH_BUFFER_BIT);
    }

    match gamestate.player.mouse_item {
        //Early return with empty item
        Item::EmptyItem => return,
        _ => {}
    }

    let mut mouse_item = Inventory::empty_with_sz(1, 1);
    mouse_item.set_item(0, 0, gamestate.player.mouse_item);
    display_inventory_items(gamestate, &mouse_item, mousepos, 30.0, w, h);

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
    }
}
