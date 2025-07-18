use super::inventory::{ITEM_TEX_SCALE, ITEM_TEX_SIZE};
use crate::{
    game::{
        assets::models::draw_elements,
        inventory::{get_item_atlas_id, Item},
        Game,
    },
    voxel::world::{get_simulation_dist, in_sim_range},
};
use cgmath::{vec3, Deg, Matrix4, SquareMatrix};

pub fn display_dropped_items(gamestate: &Game) {
    gamestate.textures.bind("items");
    let quad = gamestate.models.bind("quad2d");
    let quadshader = gamestate.shaders.use_program("quad3d");

    quadshader.uniform_matrix4f("persp", &gamestate.persp);
    quadshader.uniform_matrix4f("view", &gamestate.cam.get_view());
    quadshader.uniform_vec2f("texscale", ITEM_TEX_SCALE, ITEM_TEX_SCALE);

    unsafe {
        gl::Disable(gl::CULL_FACE);
    }

    let center = gamestate.world.get_center();
    let sim_dist = get_simulation_dist(&gamestate.world);
    //Display flat sprite items
    for (pos, list) in gamestate.entities.dropped_items.items() {
        if !in_sim_range(center, *pos, sim_dist) {
            continue;
        }

        for dropped_item in list {
            match dropped_item.item {
                Item::Block(..) | Item::Empty => continue,
                _ => {}
            }

            let pos = dropped_item.pos() + vec3(0.0, 0.2, 0.0);
            let (r, g, b) = gamestate.world.get_client_light(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            );
            quadshader.uniform_vec4f("tint", r, g, b, 1.0);

            let id = get_item_atlas_id(dropped_item.item);
            let tx = id % ITEM_TEX_SIZE;
            let ty = id / ITEM_TEX_SIZE;
            quadshader.uniform_vec2f(
                "texoffset",
                tx as f32 * ITEM_TEX_SCALE,
                ty as f32 * ITEM_TEX_SCALE,
            );

            let mut transform = Matrix4::<f32>::identity();
            transform = transform * Matrix4::from_translation(pos);
            let scale = dropped_item.scale();
            transform = transform * Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
            transform = transform * Matrix4::from_angle_y(Deg(dropped_item.rotation));
            transform = transform * Matrix4::from_angle_z(Deg(180.0));
            transform = transform * Matrix4::from_angle_y(Deg(180.0));
            quadshader.uniform_matrix4f("transform", &transform);
            draw_elements(quad.clone());
        }
    }

    gamestate.textures.bind("blocks");

    //Display flat blocks
    for (pos, list) in gamestate.entities.dropped_items.items() {
        if !in_sim_range(center, *pos, sim_dist) {
            continue;
        }

        for dropped_item in list {
            let id = match dropped_item.item {
                Item::Block(block, _) => {
                    if !block.is_flat_item() {
                        continue;
                    }
                    block.id as u16
                }
                _ => continue,
            };

            let pos = dropped_item.pos() + vec3(0.0, 0.2, 0.0);
            let (r, g, b) = gamestate.world.get_client_light(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            );
            quadshader.uniform_vec4f("tint", r, g, b, 1.0);

            let tx = id % ITEM_TEX_SIZE;
            let ty = id / ITEM_TEX_SIZE;
            quadshader.uniform_vec2f(
                "texoffset",
                tx as f32 * ITEM_TEX_SCALE,
                ty as f32 * ITEM_TEX_SCALE,
            );

            let mut transform = Matrix4::<f32>::identity();
            transform = transform * Matrix4::from_translation(pos);
            let scale = dropped_item.scale();
            transform = transform * Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
            transform = transform * Matrix4::from_angle_y(Deg(dropped_item.rotation));
            transform = transform * Matrix4::from_angle_z(Deg(180.0));
            transform = transform * Matrix4::from_angle_y(Deg(180.0));
            quadshader.uniform_matrix4f("transform", &transform);
            draw_elements(quad.clone());
        }
    }

    //Display 3D blocks

    unsafe {
        gl::Enable(gl::CULL_FACE);
    }
}
