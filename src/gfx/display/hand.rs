use super::{
    get_sky_brightness, get_skycolor,
    inventory::{display_block_item, display_block_item_flat3d, ITEM_TEX_SCALE, ITEM_TEX_SIZE},
};
use crate::{
    game::{
        assets::models::draw_elements_instanced,
        inventory::{get_item_atlas_id, Item},
        Game,
    },
    gfx::chunktable::set_fog,
    voxel::{
        chunk,
        coordinates::f32coord_to_int,
        light::{LightSrc, LU},
        Block,
    },
};
use cgmath::{Deg, Matrix4, SquareMatrix, Vector3};
use chunk::Chunk;
use std::f32::consts::PI;

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

    let hand_animation = gamestate.get_hand_animation();

    let t = if hand_animation < 0.5 {
        hand_animation * 2.0
    } else {
        (1.0 - hand_animation) * 2.0
    };

    let item_rotation = Deg(t * -HAND_ANIMATION_MAX_ROTATION);
    let rotation_animation = match held_item {
        Item::Food(..) => {
            let eat_t = gamestate.get_eat_animation();
            let eat_rotation = Deg((eat_t / 0.2).clamp(0.0, 1.0) * 75.0);
            let dy = (eat_t * 6.0 * PI).sin() * 0.1;
            let dx = -(eat_t / 0.2).clamp(0.0, 1.0) * 0.8;
            Matrix4::<f32>::from_translation(Vector3::new(dx, dy, 0.0))
                * Matrix4::<f32>::from_angle_y(eat_rotation)
                * Matrix4::<f32>::from_angle_x(item_rotation)
                * Matrix4::<f32>::from_translation(Vector3::new(0.0, t, 0.0))
        }
        Item::Tool(..) | Item::Sprite(..) | Item::Bucket(..) => {
            Matrix4::<f32>::from_angle_x(item_rotation)
                * Matrix4::<f32>::from_translation(Vector3::new(0.0, t, 0.0))
        }
        Item::Block(..) | Item::Empty => {
            Matrix4::<f32>::from_angle_x(item_rotation)
                * Matrix4::<f32>::from_translation(Vector3::new(0.0, t * 0.5, 0.0))
        }
    };

    let view = match held_item {
        Item::Block(block, _) => {
            let mut view = Matrix4::<f32>::identity();
            let y = if block.is_flat_item() { -0.6 } else { -1.0 };
            let position = Vector3::<f32>::new(1.0, y, -1.5);
            view = view * Matrix4::from_translation(position);
            view = view * rotation_animation;
            view = view * Matrix4::from_angle_y(Deg(45.0));
            view = view * Matrix4::from_scale(0.75);
            view
        }
        Item::Tool(..) => {
            let position = Vector3::new(1.0, -0.85, -1.5);
            let mut view = Matrix4::<f32>::identity();
            view = view * Matrix4::from_translation(position);
            view = view * rotation_animation;
            view = view * Matrix4::from_angle_x(Deg(180.0));
            view = view * Matrix4::from_angle_y(Deg(180.0));
            view = view * Matrix4::from_scale(0.75);
            view = view * Matrix4::from_angle_y(Deg(-100.0));
            view = view * Matrix4::from_angle_z(Deg(-20.0));
            view
        }
        _ => {
            let position = Vector3::new(1.0, -0.85, -1.5);
            let mut view = Matrix4::<f32>::identity();
            view = view * Matrix4::from_translation(position);
            view = view * rotation_animation;
            view = view * Matrix4::from_angle_x(Deg(180.0));
            view = view * Matrix4::from_angle_y(Deg(180.0));
            view = view * Matrix4::from_scale(0.75);
            view = view * Matrix4::from_angle_y(Deg(-100.0));
            view = view * Matrix4::from_angle_z(Deg(-20.0));
            view
        }
    };

    let (item_r, item_g, item_b) = if let Item::Bucket(blockid) = held_item {
        Block::new_id(blockid)
            .light_src()
            .unwrap_or(LightSrc::new(0, 0, 0))
            .rgb_f32()
    } else {
        (0.0, 0.0, 0.0)
    };

    let id = get_item_atlas_id(held_item);
    match held_item {
        Item::Block(block, _) => {
            let chunk_shader = gamestate.shaders.use_program("chunk");
            gamestate.textures.bind("blocks");

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
        Item::Sprite(..) | Item::Tool(..) | Item::Food(..) | Item::Bucket(..) => {
            let quad3d = gamestate.shaders.use_program("quad3d");
            gamestate.textures.bind("items");

            quad3d.uniform_matrix4f("view", &view);
            quad3d.uniform_matrix4f("transform", &Matrix4::from_scale(0.75));
            quad3d.uniform_matrix4f("persp", &gamestate.persp);
            quad3d.uniform_vec3f("campos", 0.0, 0.0, 0.0);
            set_fog(gamestate, &quad3d, get_skycolor(gamestate.world.time));

            let brightness = get_sky_brightness(gamestate.world.time);
            let daylight = (light.skylight() as f32 * brightness) as u16;
            let r = (light.r().max(daylight) as f32 / 15.0).max(item_r);
            let g = (light.g().max(daylight) as f32 / 15.0).max(item_g);
            let b = (light.b().max(daylight) as f32 / 15.0).max(item_b);
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
