use super::inventory::display_block_item;
use crate::{
    game::{inventory::Item, Game},
    voxel::{chunk, coordinates::f32coord_to_int, light::LU},
};
use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};
use chunk::Chunk;

const HAND_ANIMATION_MAX_ROTATION: f32 = 90.0;

pub fn display_hand_item(gamestate: &Game) {
    let hand_animation = gamestate.get_hand_animation();
    let item_rotation_x = if hand_animation < 0.5 {
        Deg(hand_animation * 2.0 * -HAND_ANIMATION_MAX_ROTATION)
    } else {
        Deg((1.0 - hand_animation) * 2.0 * -HAND_ANIMATION_MAX_ROTATION)
    };
    let rotation_animation = Matrix4::<f32>::from_angle_x(item_rotation_x);

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
            gamestate.shaders.use_program("chunk");
            gamestate.textures.bind("blocks");
            let chunk_shader = gamestate.shaders.get("chunk");

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
            display_block_item(&mut chunk, block);
        }
        Item::Sprite(id, _) => {
            //TODO: display sprite item
        }
        Item::Empty => {}
    }
}
