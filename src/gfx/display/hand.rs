use super::{
    get_sky_brightness,
    inventory::{display_block_item, display_block_item_flat3d, ITEM_TEX_SCALE, ITEM_TEX_SIZE},
};
use crate::{
    game::{assets::models::draw_elements_instanced, inventory::Item, Game},
    voxel::{chunk, coordinates::f32coord_to_int, light::LU},
};
use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};
use chunk::Chunk;

const HAND_ANIMATION_MAX_ROTATION: f32 = 90.0;

pub fn display_hand_item(gamestate: &Game) {
    let held_item = gamestate.player.hotbar.get_selected();
    let (playerx, playery, playerz) = f32coord_to_int(
        gamestate.player.position.x,
        gamestate.player.position.y,
        gamestate.player.position.z,
    );
    let light = gamestate.world.get_light(playerx, playery, playerz);
    let lu = LU::new(
        Some(light.sky()),
        Some(light.r()),
        Some(light.g()),
        Some(light.b()),
    );
    match held_item {
        Item::Block(block, _) => {
            let hand_animation = gamestate.get_hand_animation();
            let item_rotation = if hand_animation < 0.5 {
                Deg(hand_animation * 2.0 * -HAND_ANIMATION_MAX_ROTATION)
            } else {
                Deg((1.0 - hand_animation) * 2.0 * -HAND_ANIMATION_MAX_ROTATION)
            };
            let rotation_animation = Matrix4::<f32>::from_angle_x(item_rotation);

            let chunk_shader = gamestate.shaders.use_program("chunk");
            gamestate.textures.bind("blocks");

            let mut view = Matrix4::<f32>::identity();
            let y = if block.is_flat_item() { -0.6 } else { -1.0 };
            let position = Vector3::<f32>::new(1.0, y, -1.5);
            view = view * Matrix4::from_translation(position);
            view = view * rotation_animation;
            view = view * Matrix4::from_angle_y(Deg(45.0));
            view = view * Matrix4::from_scale(0.75);

            chunk_shader.uniform_matrix4f("persp", &gamestate.persp);
            chunk_shader.uniform_matrix4f("view", &view);
            chunk_shader.uniform_vec3f("chunkpos", -1.5, -1.5, -1.5);
            chunk_shader.uniform_vec3f("campos", 0.0, 0.0, 0.0);
            let mut chunk = Chunk::new(0, 0, 0);
            //Fill in the light for the chunk
            for x in 0..=2 {
                for y in 0..=2 {
                    for z in 0..=2 {
                        chunk.update_light(x, y, z, lu);
                    }
                }
            }

            if block.is_flat_item() {
                display_block_item_flat3d(&mut chunk, block);
            } else {
                display_block_item(&mut chunk, block);
            }
        }
        Item::Sprite(id, _) => {
            let hand_animation = gamestate.get_hand_animation();
            let item_rotation = if hand_animation < 0.5 {
                Deg(hand_animation * 2.75 * -HAND_ANIMATION_MAX_ROTATION)
            } else {
                Deg((1.0 - hand_animation) * 2.75 * -HAND_ANIMATION_MAX_ROTATION)
            };
            let rotation_animation = Matrix4::<f32>::from_angle_z(item_rotation);

            let quad3d = gamestate.shaders.use_program("quad3d");
            gamestate.textures.bind("items");

            let position = Vector3::new(1.0, -0.85, -1.5);
            let mut view = Matrix4::<f32>::identity();
            view = view * Matrix4::from_translation(position);
            view = view * Matrix4::from_angle_x(Deg(180.0));
            view = view * Matrix4::from_angle_y(Deg(180.0));
            view = view * Matrix4::from_scale(0.75);
            view = view * Matrix4::from_angle_y(Deg(-100.0));
            view = view * Matrix4::from_angle_z(Deg(-20.0));
            view = view * rotation_animation;
            quad3d.uniform_matrix4f("view", &view);
            quad3d.uniform_matrix4f("transform", &Matrix4::from_scale(0.75));
            quad3d.uniform_matrix4f("persp", &gamestate.persp);

            let brightness = get_sky_brightness(gamestate.world.time);
            let daylight = (light.skylight() as f32 * brightness) as u16;
            let r = light.r().max(daylight) as f32 / 15.0;
            let g = light.g().max(daylight) as f32 / 15.0;
            let b = light.b().max(daylight) as f32 / 15.0;
            quad3d.uniform_vec4f("tint", r, g, b, 1.0);
            quad3d.uniform_vec2f("texscale", ITEM_TEX_SCALE, ITEM_TEX_SCALE);
            let ix = id % ITEM_TEX_SIZE;
            let iy = id / ITEM_TEX_SIZE;
            let tx = ix as f32 * ITEM_TEX_SCALE;
            let ty = iy as f32 * ITEM_TEX_SCALE;
            quad3d.uniform_vec2f("texoffset", tx, ty);

            unsafe {
                gl::Disable(gl::CULL_FACE);
            }

            let quad = gamestate.models.bind("quad2d");
            draw_elements_instanced(quad, 8);

            unsafe {
                gl::Enable(gl::CULL_FACE);
            }
        }
        Item::Empty => {}
    }
}
