use crate::game::assets::models::draw_elements;
use crate::voxel;
use crate::{game::Game, BLOCK_REACH, EMPTY_BLOCK};
use cgmath::{Matrix4, SquareMatrix, Vector3};

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
    if gamestate.world.get_block(ix, iy, iz).id != EMPTY_BLOCK {
        let cube = gamestate.models.bind("cube");
        draw_elements(cube);
    }
}
